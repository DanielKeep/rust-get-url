extern crate conv;
extern crate winapi;
extern crate wininet as inet;
extern crate wio;
use std::convert::AsRef;
use std::io;
use std::ptr;
use self::conv::prelude::*;
use self::winapi::*;
use self::wio::wide::ToWide;

pub type Error = io::Error;

pub struct Response {
    inet: HINTERNET,
    conn: HINTERNET,
}

impl Response {
    pub fn open(req: ::Request) -> Result<Response, Error> {
        let inet = try!(internet_open(::AGENT));
        let headers = req.headers.into_iter();
        let conn = try!(internet_open_url(inet, req.url, headers));

        Ok(Response {
            inet: inet,
            conn: conn,
        })
    }
}

impl Drop for Response {
    fn drop(&mut self) {
        if let Err(err) = internet_close_handle(self.conn) {
            error!("failed to close WinINet connection: {}", err);
        }
        if let Err(err) = internet_close_handle(self.inet) {
            error!("failed to close WinINet session: {}", err);
        }
    }
}

impl io::Read for Response {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        internet_read_file(self.conn, buf)
    }
}

fn internet_close_handle(handle: HINTERNET) -> io::Result<()> {
    unsafe {
        match inet::InternetCloseHandle(handle) {
            TRUE => Ok(()),
            _ => Err(io::Error::last_os_error()),
        }
    }
}

fn internet_open<Agent>(agent: Agent) -> io::Result<HINTERNET>
where Agent: ToWide {
    unsafe {
        let agent = agent.to_wide_null();
        let agent = agent.as_ptr();
        let access_type = wininet::INTERNET_OPEN_TYPE_PRECONFIG;
        let proxy_name = ptr::null();
        let proxy_bypass = ptr::null();
        let flags = 0;
        match inet::InternetOpenW(agent, access_type, proxy_name, proxy_bypass, flags) {
            ptr if ptr.is_null() => Err(io::Error::last_os_error()),
            ptr => Ok(ptr),
        }
    }
}

fn internet_open_url<Url, HIter, HKey, HValue>(
    internet: HINTERNET,
    url: Url,
    headers: HIter,
) -> io::Result<HINTERNET>
where
    Url: ToWide,
    HIter: Iterator<Item=(HKey, HValue)>,
    HKey: AsRef<str>,
    HValue: AsRef<str>,
{
    unsafe {
        let url = url.to_wide_null();
        let url = url.as_ptr();
        let headers = {
            if headers.size_hint().1 == Some(0) {
                None
            } else {
                let mut s = vec![];
                for (k, v) in headers {
                    macro_rules! push_latin_1 {
                        ($s:expr) => {
                            stream_latin_1! {
                                $s,
                                |b| s.push(b as u16),
                                |s| return Err(io::Error::new(
                                    io::ErrorKind::Other,
                                    format!("non ISO 8859-1 character \
                                        in header: {:?}", s)))
                            }
                        }
                    }
                    push_latin_1!(&k);
                    s.extend(&[b':' as u16, b' ' as u16]);
                    push_latin_1!(&v);
                    s.extend(&[b'\r' as u16, b'\n' as u16]);
                }
                s.extend(&[b'\r' as u16, b'\n' as u16]);
                Some(s)
            }
        };
        let headers_ptr = headers.as_ref().map(|s| s.as_ptr()).unwrap_or(ptr::null());
        let headers_length = headers.as_ref().map(|s| s.len() as u32).unwrap_or(0);
        let flags = 0;
        let context = 0;
        match inet::InternetOpenUrlW(internet, url, headers_ptr, headers_length, flags, context) {
            ptr if ptr.is_null() => Err(io::Error::last_os_error()),
            ptr => Ok(ptr),
        }
    }
}

fn internet_read_file(file: HINTERNET, buffer: &mut [u8]) -> io::Result<usize> {
    unsafe {
        let buffer_ = buffer.as_ptr() as *mut _;
        let nobtr = buffer.len().value_into().unwrap_ok();
        let mut nobr = 0;
        match inet::InternetReadFile(file, buffer_, nobtr, &mut nobr) {
            TRUE => Ok(nobr as usize),
            _ => Err(io::Error::last_os_error()),
        }
    }
}
