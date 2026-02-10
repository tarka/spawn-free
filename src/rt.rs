
use std::{
    any::Any,
    cell::{Cell, RefCell},
    collections::HashMap,
    future::{Future, IntoFuture},
    pin::Pin,
    rc::Rc,
    sync::atomic::{AtomicU64, Ordering},
    task::{ContextBuilder, LocalWake, LocalWaker, Poll, Waker},
};

use io_uring::{IoUring, squeue::Entry};

use crate::errors::Result;

pub struct SpawnFree {
    uring: RefCell<IoUring>,
    tasks: RefCell<HashMap<u64, Rc<Task>>>,
    next_tok: AtomicU64,
    waker: Rc<IoUringWaker>,
}

type AnyFuture = Pin<Box<dyn Future<Output = Box<dyn Any>>>>;

thread_local! {
    pub(crate) static RT:  Rc<SpawnFree> = Rc::new(SpawnFree::new().unwrap());
}

impl SpawnFree {

    pub fn new() -> Result<Self> {

        Ok(Self {
            uring: RefCell::new(IoUring::new(64)?),
            tasks: RefCell::new(HashMap::new()),
            next_tok: AtomicU64::new(0),
            waker: Rc::new(IoUringWaker::new()),
        })
    }

    pub fn run_future<F: IntoFuture>(&self, fut: F) -> F::Output {
        let mut fut = core::pin::pin!(fut.into_future());

        let waker = LocalWaker::from(self.waker.clone());
        let mut context = ContextBuilder::from_waker(Waker::noop())
            .local_waker(&waker)
            .build();

        println!("Running Future");

        let retval = loop {
            match fut.as_mut().poll(&mut context) {
                Poll::Pending => {
                    self.wait_and_wake().unwrap();
                },
                Poll::Ready(item) => break item,
            }
        };
        println!("End of Future");
        retval
    }


    pub(crate) fn submit(&self, op: Entry, future: AnyFuture) -> Result<Rc<Task>> {
        let tok = self.next_tok.fetch_add(1, Ordering::SeqCst);
        let op = op.clone()
            .user_data(tok);

        println!("QUEUEING {:?}", op);
        let mut uring = self.uring.borrow_mut();
        unsafe {
            uring.submission()
                .push(&op).unwrap()
        };
        println!("SUBMITTED");

        let task = Rc::new(Task::new(op, future));

        self.tasks.borrow_mut().insert(tok, task.clone());

        Ok(task)
    }

    fn wait_and_wake(&self) -> Result<()> {
        println!("Wait & Wake");
        let mut uring = self.uring.borrow_mut();
        let _n = uring.submit_and_wait(1)?;

        let completions = uring
            .completion();

        for entry in completions {
            let tok = entry.user_data();
            let task = self.tasks.borrow_mut()
                .remove(&tok)
                .unwrap();  // FIXME
            task.set_ready(); // FIXME??

            let waker = LocalWaker::from(self.waker.clone());
            let mut context = ContextBuilder::from_waker(Waker::noop())
                .local_waker(&waker)
                .build();

            // FIXME??
            let mut future = task.future.borrow_mut();
            let _ = future.as_mut().poll(&mut context);
        }

        Ok(())
    }
}


pub(crate) struct Task {
    pub _op: Entry,
    pub state: Cell<Poll<()>>,
    pub future: RefCell<AnyFuture>,
}

impl Task {
    pub fn new(_op: Entry, future: Pin<Box<dyn Future<Output = Box<dyn Any>>>>) -> Self {
        Self {
            _op,
            future: RefCell::new(future),
            state: Cell::new(Poll::Pending),
        }
    }

    pub fn set_ready(&self) {
        self.state.set(Poll::Ready(()));
    }
}


#[derive(Clone)]
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
        let rt = SpawnFree::new()?;
        let sq = rt.run_future(async_sq(10));
        assert_eq!(100, sq);
        Ok(())
    }

}
