use std::{cell::RefCell, rc::Rc, thread::{self, sleep}, time::Duration};

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

#[test]
fn test_uring_timeout_refcell() {
    let uring_rc = Rc::new(RefCell::new(IoUring::new(16).unwrap()));
    let uring = uring_rc.clone();
    let mut uring = uring.borrow_mut();

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

#[test]
fn test_uring_sqpoll() {
    let mut uring: IoUring = IoUring::builder()
        .setup_sqpoll(1000)
        .build(16)
        .unwrap();

    assert!(uring.params().is_setup_sqpoll());

    let _n = uring.submit().unwrap();

    let wait_for = Timespec::new()
        .sec(2);

    let op = opcode::Timeout::new(&wait_for)
        .build()
        .user_data(2);

    println!("Submitting {op:?}");
    unsafe { uring.submission().push(&op).unwrap()  };


    println!("Waiting...");
    for _ in 0..10 {
        if let Some(resp) = uring.completion().next() {
            println!("Found {resp:?}");
            assert_eq!(2, resp.user_data());
            return;
        }
        thread::sleep(Duration::from_secs(1));
    }


    panic!("Timeout didn't fire!");
}
