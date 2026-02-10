use std::{future::Pending, pin::Pin, task::{Context, Poll}, time::{Duration, Instant}};

use io_uring::{opcode::{self, Timeout}, types::Timespec};

use crate::rt::Task;


struct Delay {
    when: Instant,
}

impl Future for Delay {
    type Output = &'static str;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>)
        -> Poll<&'static str>
    {
        if Instant::now() >= self.when {
            println!("Hello world");
            Poll::Ready("done")
        } else {
            // Get a handle to the waker for the current task
            let waker = cx.waker().clone();
            let when = self.when;

            // // Spawn a timer thread.
            // thread::spawn(move || {
            //     let now = Instant::now();

            //     if now < when {
            //         thread::sleep(when - now);
            //     }

            //     waker.wake();
            // });

            Poll::Pending
        }
    }
}

// struct Sleep {
//     task: Task,
// }

// impl Future for Sleep {
//     type Output = ();

//     fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        
//     }
// }


// pub async fn sleep(duration: Duration) -> Sleep {
    

// }
