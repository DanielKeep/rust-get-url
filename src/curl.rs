extern crate curl;
use std::io;
use self::curl::http;

pub type Error = curl::ErrCode;

pub struct Response {
    body: io::Cursor<Vec<u8>>,
}

impl Response {
    pub fn open(req: ::Request) -> Result<Response, Error> {
        if req.headers.len() != 0 {
            panic!("NYI: custom headers with curl backend");
        }
        let res = try!(http::handle()
            .get(req.url)
            .header("User-Agent", ::AGENT)
            .exec());
        let body = io::Cursor::new(res.move_body());
        Ok(Response {
            body: body,
        })
    }
}

impl io::Read for Response {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.body.read(buf)
    }
}
