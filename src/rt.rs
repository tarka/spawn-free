
use std::{
    collections::HashMap, future::{Future, IntoFuture}, pin::Pin, rc::Rc, sync::Arc, task::{Context, ContextBuilder, LocalWake, LocalWaker, Poll, Wake, Waker}, thread
};

use io_uring::IoUring;

use crate::Result;

// thread_local! {
//     static LOCAL_WAKER: Waker = {
//         let uw = Arc::new(SpawnFree {
//             uring: IoUring::new(64).unwrap(),
//             tasks: HashMap::new(),
//         });
//         Waker::from(uw)
//     };
// }

struct SpawnFree {
    uring: IoUring,
    tasks: HashMap<u64, Pin<Box<dyn Future<Output = ()>>>>,
}

impl SpawnFree {

    pub fn new() -> Result<Self> {
        Ok(Self {
            uring: IoUring::new(64)?,
            tasks: HashMap::new(),
        })
    }

    pub fn run_future<F: IntoFuture>(self, fut: F) -> F::Output {
        let mut fut = core::pin::pin!(fut.into_future());

        let lw = Rc::new(self);
        let waker = LocalWaker::from(lw);
        let mut context = ContextBuilder::from_waker(Waker::noop())
            .local_waker(&waker)
            .build();

        loop {
            match fut.as_mut().poll(&mut context) {
                Poll::Pending => thread::park(),
                Poll::Ready(item) => break item,
            }
        }
    }
}


//struct LW {}

//impl LocalWake for LW {
impl LocalWake for SpawnFree {
    fn wake(self: Rc<Self>) {
        let mut lw = self.clone();
        // let completions = Rc::get_mut(&mut lw).unwrap().uring.completion();
        // for entry in completions {
        //     let id = entry.user_data();
        //    let task = self.tasks.get(&id).unwrap();
        // }
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
    fn test_timer() {
        let rt = SpawnFree::new().unwrap();
        let sq = rt.run_future(async_sq(10));
        assert_eq!(100, sq);
    }

}
