extern crate conv;
extern crate url;
extern crate winapi;
extern crate wininet as inet;
extern crate wio;
use std::convert::AsRef;
use std::io;
use std::ptr;
use self::conv::prelude::*;
use self::url::Url;
use self::winapi::*;
use self::wio::wide::ToWide;

macro_rules! LPCWSTR {
    ($($chs:expr),*) => {
        {
            const S: &'static [u16] = &[$($chs as u16,)* 0];
            S.as_ptr()
        }
    };
}

pub type Error = io::Error;

pub struct Response {
    inet: HINTERNET,
    conn: HINTERNET,
    req: HINTERNET,
}

impl Response {
    pub fn open(req: ::Request) -> Result<Response, Error> {
        // TODO: wrap handles in something that'll drop them.
        let url = try!(Url::parse(req.url)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e)));
        let headers = req.headers;

        let inet = try!(internet_open(::AGENT));
        let conn = try!(internet_connect(inet, &url));
        let req = try!(http_open_request(conn, "GET", &url));
        try!(http_add_request_headers(req, headers.into_iter()));
        try!(http_send_request(req, None));

        Ok(Response {
            inet: inet,
            conn: conn,
            req: req,
        })
    }
}

impl Drop for Response {
    fn drop(&mut self) {
        if let Err(err) = internet_close_handle(self.req) {
            error!("failed to close WinINet request: {}", err);
        }
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
        internet_read_file(self.req, buf)
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

fn internet_connect(handle: HINTERNET, url: &Url) -> io::Result<HINTERNET> {
    let (
        service, username, password, host, port, default_port
    ) = match (&*url.scheme, &url.scheme_data) {
        (scheme, &url::SchemeData::Relative(ref rsd)) => {
            let service = match scheme {
                // Disabled until curl backend has support as well.
                // "ftp" => wininet::INTERNET_SERVICE_FTP,
                "http" | "https" => wininet::INTERNET_SERVICE_HTTP,
                _ => return Err(io::Error::new(
                    io::ErrorKind::Other,
                    format!("unrecognised scheme `{}`", scheme)
                ))
            };
            (service, &*rsd.username, rsd.password.as_ref(), &rsd.host,
                rsd.port, rsd.default_port)
        },
        (scheme, &url::SchemeData::NonRelative(_)) => {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("unrecognised scheme `{}`", scheme)
            ));
        }
    };

    unsafe {
        let host = host.serialize().to_wide_null();
        let host = host.as_ptr();
        let port = port.unwrap_or(default_port.expect("no default port"));
        let username = if username == "" { None } else { Some(username.to_wide_null()) };
        let username = username.as_ref().map(|s| s.as_ptr()).unwrap_or(ptr::null());
        let password = password.map(|s| s.to_wide_null());
        let password = password.as_ref().map(|s| s.as_ptr()).unwrap_or(ptr::null());
        let flags = 0;
        let context = 0;
        match inet::InternetConnectW(handle, host, port, username, password, service, flags, context) {
            ptr if ptr.is_null() => Err(io::Error::last_os_error()),
            ptr => Ok(ptr),
        }
    }
}

fn http_open_request(connect: HINTERNET, verb: &str, url: &Url) -> io::Result<HINTERNET> {
    unsafe {
        let secure = (&*url.scheme) == "https";
        let secure = if secure { wininet::INTERNET_FLAG_SECURE } else { 0 };

        let verb = verb_to_lpcwstr(verb);
        let mut object_name = url.serialize_path().unwrap();
        if let Some(ref query) = url.query {
            object_name.reserve(1 + query.len());
            object_name.push('?');
            object_name.push_str(query);
        }
        let object_name = object_name.to_wide_null();
        let object_name = object_name.as_ptr();
        let version = ptr::null();
        let referer = ptr::null();
        let accept_types = ptr::null_mut();
        let flags = secure;
        let context = 0;
        match inet::HttpOpenRequestW(connect, verb, object_name, version, referer, accept_types, flags, context) {
                ptr if ptr.is_null() => Err(io::Error::last_os_error()),
                ptr => Ok(ptr),
        }
    }
}

fn http_add_request_headers<HIter, HKey, HValue>(
    request: HINTERNET,
    headers: HIter,
) -> io::Result<()>
where
    HIter: Iterator<Item=(HKey, HValue)>,
    HKey: AsRef<str>,
    HValue: AsRef<str>,
{
    unsafe {
        let headers = {
            let mut s = vec![];
            for (k, v) in headers {
                push_latin_1!(s, &k);
                s.extend(&[b':' as u16, b' ' as u16]);
                push_latin_1!(s, &v);
                s.extend(&[b'\r' as u16, b'\n' as u16]);
            }
            s.extend(&[b'\r' as u16, b'\n' as u16]);
            s
        };
        let headers_ptr = headers.as_ptr();
        let headers_len = headers.len().value_into().expect("header length overflow");
        let modifiers = wininet::HTTP_ADDREQ_FLAG_ADD | wininet::HTTP_ADDREQ_FLAG_REPLACE;
        match inet::HttpAddRequestHeadersW(request, headers_ptr, headers_len, modifiers) {
            FALSE => Err(io::Error::last_os_error()),
            _ => Ok(()),
        }
    }
}

fn http_send_request(request: HINTERNET, optional: Option<&[u8]>) -> io::Result<()> {
    /*
    OK, so... the documentation says nothing about `HttpSendRequest` mutating the `optional` data... but it's not *defined* as taking a constant pointer, so... yeah.  If you can somehow *prove* that the data pointed to is never touched, feel free to replace this with an unsafe cast.
    */
    let mut optional = optional.map(|slice| slice.to_owned());
    unsafe {
        let headers_ptr = ptr::null();
        let headers_len = 0;
        let optional_ptr = optional.as_mut().map(|a| a.as_mut_ptr()).unwrap_or(ptr::null_mut());
        let optional_ptr = optional_ptr as *mut _;
        let optional_len = optional.as_ref().map(|a| a.len().value_into()).unwrap_or(Ok(0));
        let optional_len = try!(optional_len
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e)));
        match inet::HttpSendRequestW(request, headers_ptr, headers_len, optional_ptr, optional_len) {
            TRUE => Ok(()),
            _ => Err(io::Error::last_os_error()),
        }
    }
}

fn internet_read_file(file: HINTERNET, buffer: &mut [u8]) -> io::Result<usize> {
    unsafe {
        let buffer_ = buffer.as_ptr() as *mut _;
        let nobtr = try!(buffer.len()
            .value_into()
            .map_err(|_| io::Error::new(
                io::ErrorKind::Other,
                "buffer length overflow")));
        let mut nobr = 0;
        match inet::InternetReadFile(file, buffer_, nobtr, &mut nobr) {
            TRUE => Ok(nobr as usize),
            _ => Err(io::Error::last_os_error()),
        }
    }
}

fn verb_to_lpcwstr(verb: &str) -> LPCWSTR {
    match verb {
        "GET" => LPCWSTR!['G', 'E', 'T'],
        "PUT" => LPCWSTR!['P', 'U', 'T'],
        "HEAD" => LPCWSTR!['H', 'E', 'A', 'D'],
        "POST" => LPCWSTR!['P', 'O', 'S', 'T'],
        "OPTIONS" => LPCWSTR!['O', 'P', 'T', 'I', 'O', 'N', 'S'],
        "DELETE" => LPCWSTR!['D', 'E', 'L', 'E', 'T', 'E'],
        "TRACE" => LPCWSTR!['T', 'R', 'A', 'C', 'E'],
        "CONNECT" => LPCWSTR!['C', 'O', 'N', 'N', 'E', 'C', 'T'],
        _ => panic!("unsupported verb {:?}", verb)
    }
}
