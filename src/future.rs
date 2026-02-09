use std::cell::Cell;


struct UringFuture<T> {
    retval: Cell<Option<T>>,
}

// impl<T> Future for UringFuture<T> {
//     type Output = T;
// }
