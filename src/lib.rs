

use std::{collections::VecDeque, pin::{Pin, pin}, task::{Context, Poll}};
use futures::task;



type Task = Pin<Box<dyn Future<Output = ()>>>;

pub struct SpawnFree {
    _tasks: VecDeque<Task>,
}

impl SpawnFree {

    pub fn new() -> Self {
        Self {
            _tasks: VecDeque::new()
        }
    }

    pub fn run_future<F, O>(&self, future: F) -> O
        where F: Future<Output = O>,
    {
        let waker = task::noop_waker();
        let mut cx = Context::from_waker(&waker);
        let mut future = pin!(future);

        let ret = loop {
            match future.as_mut().poll(&mut cx) {
                Poll::Pending => {
                    continue;
                },
                Poll::Ready(ret) => {
                    break ret
                }
            }
        };
        ret
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn async_sq(val: u64) -> u64 {
        val * val
    }

    #[test]
    fn test_timer() {
        let ex = SpawnFree::new();
        let sq = ex.run_future(async_sq(10));
        assert_eq!(100, sq);
    }

}
