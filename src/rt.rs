use std::{
    cell::Cell, collections::HashMap, future::{Future, IntoFuture}, pin::Pin, rc::Rc, sync::atomic::{AtomicU64, Ordering}, task::{ContextBuilder, LocalWake, LocalWaker, Poll, Waker}
};

use io_uring::{IoUring, squeue::Entry};

use crate::Result;

pub struct SpawnFree {
    uring: IoUring,
    tasks: HashMap<u64, Task>,
    next_tok: AtomicU64,

}

thread_local! {
    pub(crate) static RT: SpawnFree = SpawnFree::new().unwrap();
}

impl SpawnFree {

    pub fn new() -> Result<Self> {
        Ok(Self {
            uring: IoUring::new(64)?,
            tasks: HashMap::new(),
            next_tok: AtomicU64::new(0),
        })
    }

    pub fn run_future<F: IntoFuture>(&mut self, fut: F) -> F::Output {
        let mut fut = core::pin::pin!(fut.into_future());

        let uwaker = Rc::new(IoUringWaker::new());
        let waker = LocalWaker::from(uwaker.clone());
        let mut context = ContextBuilder::from_waker(Waker::noop())
            .local_waker(&waker)
            .build();

        loop {
            match fut.as_mut().poll(&mut context) {
                Poll::Pending => {
                    self.wait_and_wake().unwrap();
                },
                Poll::Ready(item) => break item,
            }
        }
    }

    pub(crate) fn submit(&mut self, task: Task) -> Result<()> {
        let tok = self.next_tok.fetch_add(1, Ordering::SeqCst);
        let op = task.op.clone()
            .user_data(tok);

        println!("QUEUEING {:?}", op);
        unsafe {
            self.uring.submission()
                .push(&op).unwrap()
        };
        println!("SUBMITTED");

        self.tasks.insert(tok, task);

        Ok(())
    }

    fn wait_and_wake(&mut self) -> Result<()> {
        let _n = self.uring.submit_and_wait(1)?;

        let completions = self.uring
            .completion();

        for entry in completions {
            let id = entry.user_data();
            let task = self.tasks.remove(&id)
                .unwrap();  // FIXME
        }

        Ok(())
    }
}


pub(crate) struct Task {
    op: Entry,
    future: Pin<Box<dyn Future<Output = ()>>>,
}


struct IoUringWaker {
}

impl IoUringWaker {
    fn new() -> Self {
        Self { }
    }
}

// impl IoUringWaker {

//     fn wait_and_wake(self: &Rc<Self>) {
//         let _n = self.uring.submit_and_wait(1)
//             .unwrap(); // FIXME

//         let mut iuw = self.clone();
//         let completions = Rc::get_mut(&mut iuw).unwrap()
//             .uring.completion();

//         for entry in completions {
//             let id = entry.user_data();
//             let task = self.tasks.get(&id).unwrap();
//         }
//     }

// }

impl LocalWake for IoUringWaker {
    fn wake(self: Rc<Self>) {
       // let mut lw = self.clone();
       //  let completions = Rc::get_mut(&mut lw).unwrap().uring.completion();
       //  for entry in completions {
       //      let id = entry.user_data();
       //     let task = self.tasks.get(&id).unwrap();
       //  }
    }

    // fn wake_by_ref(self: &Rc<Self>) {
    // }
}


#[cfg(test)]
mod tests {
    use super::*;

    async fn async_sq(val: u64) -> u64 {
        val * val
    }

    #[test]
    fn test_timer() -> Result<()> {
        let mut rt = SpawnFree::new()?;
        let sq = rt.run_future(async_sq(10));
        assert_eq!(100, sq);
        Ok(())
    }

}
