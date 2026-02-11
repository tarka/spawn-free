use io_uring::{IoUring, opcode, types::Timespec};


#[test]
fn test_uring_timeout() {
    let mut uring = IoUring::new(16).unwrap();

    let wait_for = Timespec::new()
        .sec(2);

    let op = opcode::Timeout::new(&wait_for)
        .build()
        .user_data(2);

    println!("QUEUEING {op:?}");
    unsafe { uring.submission().push(&op).unwrap()  };
    println!("SUBMITTING & WAITING");
    let queued = uring.submit_and_wait(1).unwrap();

    println!("Completed {queued}");

    let resp = uring.completion().next().unwrap();
    println!("RESULT {resp:?}");
}
