use std::{any::Any, pin::Pin, rc::Rc, task::{Context, Poll}, time::Duration};

use io_uring::{opcode, types::Timespec};

use crate::rt::Task;

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

pub fn sleep(duration: Duration) -> Sleep {
    println!("Sleep({duration:?})");

    let wait_for = Timespec::new()
        .sec(duration.as_secs());

    let op = opcode::Timeout::new(&wait_for)
        .build();

    let fut = Box::pin(async move {
        let val: Box<dyn Any> = Box::new(12);
        val
    });

    println!("Sleep() submit {op:?}");
    let task = crate::rt::RT.with(|rt| {
        rt.submit_io(op, fut, "sleep")
            .unwrap()
    });

    println!("Sleep() returning Sleep");
    Sleep { task }
}

#[cfg(test)]
mod test {
    use std::time::Duration;

    use crate::rt;


    #[test]
    fn test_sleep() {
        rt::RT.with(|rt| {
            rt.run_future(
                // async {
                //     let start = Instant::now();
                //     println!("Start sleep: {start:?}");
                //     super::sleep(Duration::from_secs(2)).await;
                //     let end = Instant::now();
                //     println!("Start: {end:?}");
                //     let diff = end - start;
                //     println!("Slept {diff:?}");
                // }

                super::sleep(Duration::from_secs(2))

            )
        });
        panic!();
    }

}
