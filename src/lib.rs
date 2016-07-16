#[macro_use] extern crate log;

use std::convert::AsRef;

#[cfg(windows)] mod wininet;
#[cfg(windows)] pub use wininet::{Error, Response};

#[cfg(not(windows))] mod curl;
#[cfg(not(windows))] pub use curl::{Error, Response};

pub struct Request<'a> {
    url: &'a str,
}

impl<'a> Request<'a> {
    pub fn new<S: ?Sized + AsRef<str>>(url: &'a S) -> Self {
        Request {
            url: url.as_ref(),
        }
    }

    pub fn open(self) -> Result<Response, Error> {
        Response::open(self)
    }
}
