extern crate curl;
use std::io;
use self::curl::http;

pub type Error = curl::ErrCode;

pub struct Response {
    body: io::Cursor<Vec<u8>>,
}

impl Response {
    pub fn open(req: ::Request) -> Result<Response, Error> {
        let mut handle = http::handle();
        let mut curl_req = handle.get(req.url);

        let mut found_user_agent = false;
        for (name, value) in &req.headers {
            if &name[..] == "User-Agent" {
                found_user_agent = true;
            }
            curl_req = curl_req.header(name, value);
        }
        if !found_user_agent {
            // We have to do this because `curl` *apparently ignores* headers you've already set.
            curl_req = curl_req.header("User-Agent", ::AGENT);
        }

        let res = try!(curl_req.exec());
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
