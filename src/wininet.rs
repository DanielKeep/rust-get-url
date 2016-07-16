extern crate conv;
extern crate winapi;
extern crate wininet as inet;
extern crate wio;
use std::io;
use std::ptr;
use self::conv::prelude::*;
use self::winapi::*;
use self::wio::wide::ToWide;

pub const AGENT: &'static str = concat!("get-url/", env!("CARGO_PKG_VERSION"));
// pub const SCHEMES: &'static [&'static str] = &["http", "https", "ftp"];

pub type Error = io::Error;

pub struct Response {
    inet: HINTERNET,
    conn: HINTERNET,
}

impl Response {
    pub fn open(req: ::Request) -> Result<Response, Error> {
        let inet = try!(internet_open(AGENT));
        let conn = try!(internet_open_url(inet, req.url));

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

const INTERNET_OPEN_TYPE_PRECONFIG: DWORD = 0;
// const INTERNET_OPEN_TYPE_DIRECT: DWORD = 1;
// const INTERNET_OPEN_TYPE_PROXY: DWORD = 3;
// const INTERNET_OPEN_TYPE_PRECONFIG_WITH_NO_AUTOPROXY: DWORD = 4;

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
        let access_type = INTERNET_OPEN_TYPE_PRECONFIG;
        let proxy_name = ptr::null();
        let proxy_bypass = ptr::null();
        let flags = 0;
        match inet::InternetOpenW(agent, access_type, proxy_name, proxy_bypass, flags) {
            ptr if ptr.is_null() => Err(io::Error::last_os_error()),
            ptr => Ok(ptr),
        }
    }
}

fn internet_open_url<Url>(internet: HINTERNET, url: Url) -> io::Result<HINTERNET>
where Url: ToWide {
    unsafe {
        let url = url.to_wide_null();
        let url = url.as_ptr();
        let headers = ptr::null();
        let headers_length = 0;
        let flags = 0;
        let context = 0;
        match inet::InternetOpenUrlW(internet, url, headers, headers_length, flags, context) {
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
