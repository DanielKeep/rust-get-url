#[macro_use] extern crate log;

#[cfg(windows)] #[macro_use] mod iso_8859_1;

use std::borrow::Cow;
use std::convert::AsRef;
use std::collections::HashMap;

pub const AGENT: &'static str = concat!("get-url/", env!("CARGO_PKG_VERSION"));

#[cfg(windows)] mod wininet;
#[cfg(windows)] pub use wininet::{Error, Response};

#[cfg(not(windows))] mod curl;
#[cfg(not(windows))] pub use curl::{Error, Response};

#[derive(Clone, Debug)]
pub struct Request<'a, 'b> {
    url: &'a str,
    headers: HashMap<Cow<'b, str>, Cow<'b, str>>,
}

impl<'a, 'b> Request<'a, 'b> {
    pub fn new<S: ?Sized + AsRef<str>>(url: &'a S) -> Self {
        Request {
            url: url.as_ref(),
            headers: [("Accept".into(), "*/*".into())].into_iter()
                .cloned()
                .collect(),
        }
    }

    pub fn url(&self) -> &str {
        self.url
    }

    pub fn set_header<N, V>(&mut self, name: N, value: V) -> &mut Self
    where
        N: Into<Cow<'b, str>>,
        V: Into<Cow<'b, str>>,
    {
        self.headers.insert(name.into(), value.into());
        self
    }

    pub fn with_header<N, V>(mut self, name: N, value: V) -> Self
    where
        N: Into<Cow<'b, str>>,
        V: Into<Cow<'b, str>>,
    {
        self.set_header(name, value);
        self
    }

    pub fn open(self) -> Result<Response, Error> {
        Response::open(self)
    }
}
