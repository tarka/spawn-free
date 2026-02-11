use std::{any::Any, cell::{Cell, RefCell}, pin::Pin, rc::Rc, task::{Context, Poll}, time::Duration};

use io_uring::{opcode, types::Timespec};
use tracing::{info, warn};

use crate::rt::Task;

pub struct Sleep {
    duration: Duration,
    task: RefCell<Option<Rc<Task>>>,
}

impl Sleep {
    fn new(duration: Duration) -> Self {
        Sleep {
            duration,
            task: RefCell::new(None),
        }
    }

    fn queue_timeout(&self) -> Rc<Task> {
        info!("Adding timeout to io_uring queue");
        let secs = self.duration.as_secs();
        info!("Sleep({secs:?})");

        let wait_for = Timespec::new()
            .sec(secs);
        info!("Setup timeout for {wait_for:?}");

        let op = opcode::Timeout::new(&wait_for)
            .build();

        let fut = Box::pin(async move {
            let val: Box<dyn Any> = Box::new(12);
            val
        });

        info!("Sleep() submit {op:?}");
        let task = crate::rt::RT.with(|rt| {
            rt.submit_io(op, fut, "sleep")
                .unwrap()
        });

        task
    }
}

impl Future for Sleep {
    type Output = ();

    #[tracing::instrument(skip(self))]
    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        info!("Sleep Polled");
        let task = self.task.borrow().clone();
        match task {
            Some(task) => {
                let state = task.state.get();
                info!("State = {state:?}");
                state
            },
            None => {
                info!("First run, create uring queue entry");
                let task = self.queue_timeout();
                self.task.replace(Some(task));
                Poll::Pending
            }
        }
    }
}

#[tracing::instrument]
pub fn sleep(duration: Duration) -> Sleep {
    info!("Sleep() returning Sleep future");
    Sleep::new(duration)
}

#[cfg(test)]
mod test {
    use super::*;
    use std::time::{Duration, Instant};
    use test_log::test;

    use crate::rt;


    #[test]
    fn test_sleep() {
        rt::RT.with(|rt| {
            rt.run_future(
                // async {
                //     let start = Instant::now();
                //     info!("Start sleep: {start:?}");
                //     super::sleep(Duration::from_secs(2)).await;
                //     let end = Instant::now();
                //     info!("Start: {end:?}");
                //     let diff = end - start;
                //     info!("Slept {diff:?}");
                // }

               super::sleep(Duration::from_secs(2))

            )
        });
//        panic!();
    }

}
