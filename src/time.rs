use std::{any::Any, pin::Pin, rc::Rc, task::{Context, Poll}, time::Duration};

use io_uring::{opcode, types::Timespec};

use crate::rt::Task;


// struct Delay {
//     when: Instant,
// }

// impl Future for Delay {
//     type Output = &'static str;

//     fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>)
//         -> Poll<&'static str>
//     {
//         if Instant::now() >= self.when {
//             println!("Hello world");
//             Poll::Ready("done")
//         } else {
//             // Get a handle to the waker for the current task
//             let waker = cx.waker().clone();
//             let when = self.when;

//             // // Spawn a timer thread.
//             // thread::spawn(move || {
//             //     let now = Instant::now();

//             //     if now < when {
//             //         thread::sleep(when - now);
//             //     }

//             //     waker.wake();
//             // });

//             Poll::Pending
//         }
//     }
// }

pub struct Sleep {
    task: Rc<Task>,
}

impl Future for Sleep {
    type Output = ();

    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        println!("Sleep Polled");
        let state = self.task.state.get();
        println!("State = {state:?}");
        state
    }
}


pub async fn sleep(duration: Duration) -> Sleep {
    let wait_for = Timespec::new()
        .sec(duration.as_secs());

    let op = opcode::Timeout::new(&wait_for)
        .build();

    let fut = Box::pin(async move {
        let val: Box<dyn Any> = Box::new(12);
            val
    });

    let task = crate::rt::RT.with(|rt| {
        rt.submit(op, fut)
            .unwrap()
    });

    Sleep { task }
}

#[cfg(test)]
mod test {
    use std::time::{Duration, Instant};

    use crate::rt;


    #[test]
    fn test_sleep() {
        rt::RT.with(|rt| {
            rt.run_future(
                async {
                    let start = Instant::now();
                    println!("Start sleep: {start:?}");
                    super::sleep(Duration::from_secs(2)).await;
                    let end = Instant::now();
                    println!("Start: {end:?}");
                    let diff = end - start;
                    println!("Slept {diff:?}");
                })
        });
        panic!();
    }

}
