#![feature(local_waker)]

pub mod rt;

#[derive(thiserror::Error, Debug)]
enum Error {
    #[error(transparent)]
    IOError(#[from] std::io::Error),
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
