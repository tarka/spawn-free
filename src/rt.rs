
use std::{
    any::Any,
    cell::{Cell, RefCell},
    collections::{HashMap, VecDeque},
    future::Future,
    pin::Pin,
    rc::Rc,
    sync::atomic::{AtomicU64, Ordering},
    task::{ContextBuilder, LocalWake, LocalWaker, Poll, Waker},
};

use derivative::Derivative;
use io_uring::{IoUring, squeue::Entry};
use tracing::{info, warn};

use crate::errors::Result;

#[derive(Derivative)]
#[derivative(Debug)]
pub struct SpawnFree {
    #[derivative(Debug="ignore")]
    uring: RefCell<IoUring>,
    io_pending: RefCell<HashMap<u64, Rc<Task>>>,
    next_tok: AtomicU64,
    waker: Rc<IoUringWaker>,
}

type AnyFuture = Pin<Box<dyn Future<Output = Box<dyn Any>>>>;

thread_local! {
    static RT:  Rc<SpawnFree> = Rc::new(SpawnFree::new().unwrap());
}

impl SpawnFree {

    #[tracing::instrument]
    pub fn new() -> Result<Self> {
        info!("Creating SpawnFree");
        let uring = IoUring::builder()
            .setup_sqpoll(1000)
            .build(16)?;
        let _n = uring.submit().unwrap();

        Ok(Self {
            uring: RefCell::new(uring),
            io_pending: RefCell::new(HashMap::new()),
            next_tok: AtomicU64::new(1),
            waker: Rc::new(IoUringWaker::new()),
        })
    }

    pub fn current() -> Rc<Self> {
        RT.with(|rt| rt.clone())
    }

    #[tracing::instrument(skip(self, future))]
    pub fn run_future<F: Future<Output = O> + 'static, O>(&self, future: F) -> () {
        let mut future = core::pin::pin!(future);

        let waker = LocalWaker::from(self.waker.clone());
        let mut context = ContextBuilder::from_waker(Waker::noop())
            .local_waker(&waker)
            .build();

        let mut done = false; // FIXME

        while !done {
            info!("POLLING top-level");
            match future.as_mut().poll(&mut context) {
                Poll::Pending => {
                    info!("TOP: State is pending, yeilding to IO-URING");
                    self.wait_and_wake().unwrap();
                },
                Poll::Ready(_item) => {
                    info!("TOP: State is Ready, returning value");
                    done = true;
                    //break item
                },
            }
        }

        info!("End of Top-Level Future");
    }

    #[tracing::instrument(skip(self, future))]
    pub(crate) fn submit_io(&self, op: Entry, future: AnyFuture, name: &'static str) -> Result<Rc<Task>> {
        let tok = self.next_tok.fetch_add(1, Ordering::SeqCst);
        let op = op.clone()
            .user_data(tok);

        info!("QUEUEING {:?}", op);
        let uring = self.uring.borrow_mut();
        unsafe {
            uring.submission_shared()
                .push(&op).unwrap()
        };
        info!("SUBMITTED TO IO-URING");

        let task = Rc::new(Task::new(future, name));

        self.io_pending.borrow_mut().insert(tok, task.clone());

        Ok(task)
    }

    #[tracing::instrument(skip(self))]
    fn wait_and_wake(&self) -> Result<()> {
        info!("Wait & Wake");
        let mut uring = self.uring.borrow_mut();

        info!("URING ID: {:p}", &uring);

        let sublen = uring.submission().len();
        info!("SUB Q LEN = {sublen}");
        info!("SUB Q: {:?}", uring.submission());

        info!("COMP Q LEN = {}", uring.completion().len());

        info!("Submitting {sublen} and waiting");
        let n = uring.submit_and_wait(1)?;
        info!("Submitted {n}");

        info!("SUB Q is now: {:?}", uring.submission());
        info!("COMP Q len {:?}", uring.completion().len());

        for entry in unsafe { uring.completion_shared() } {
            info!("GOT COMPLETION {entry:?}");
            let tok = entry.user_data();
            let task = self.io_pending.borrow_mut()
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

        info!("DONE WAIT_AND_WAKE");

        Ok(())
    }
}



#[derive(Derivative)]
#[derivative(Debug)]
pub(crate) struct Task {
    pub name: &'static str,
    pub state: Cell<Poll<()>>, // FIXME: Real type?
    #[derivative(Debug="ignore")]
    pub future: RefCell<AnyFuture>,
}

impl Task {
    pub fn new(future: AnyFuture, name: &'static str) -> Self {
        Self {
            name,
            future: RefCell::new(future),
            state: Cell::new(Poll::Pending),
        }
    }

    pub fn set_ready(&self) {
        info!("{}: Setting Ready", self.name);
        // FIXME: Real data
        self.state.set(Poll::Ready(()));
    }
}


#[derive(Clone, Debug)]
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


// #[cfg(test)]
// mod tests {
//     use super::*;

//     async fn async_sq(val: u64) -> u64 {
//         val * val
//     }

//     #[test]
//     fn test_timer() -> Result<()> {
//         let rt = SpawnFree::new()?;
//         let sq = rt.run_future(async_sq(10));
//         assert_eq!(100, sq);
//         Ok(())
//     }

// }
