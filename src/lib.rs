#![feature(local_waker)]

pub mod future;
pub mod rt;
pub mod time;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    IOError(#[from] std::io::Error),
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
