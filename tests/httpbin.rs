extern crate get_url;
extern crate rustc_serialize;

use get_url::Request;
use rustc_serialize::json::Json;

trait IntoRequest<'a, 'b> {
    fn into_request(self) -> Request<'a, 'b>;
}

impl<'a, 'b> IntoRequest<'a, 'b> for &'a str {
    fn into_request(self) -> Request<'a, 'b> {
        Request::new(self)
    }
}

impl<'a, 'b> IntoRequest<'a, 'b> for Request<'a, 'b> {
    fn into_request(self) -> Request<'a, 'b> {
        self
    }
}

macro_rules! get_url_body {
    (as Json, $url:expr) => {
        match $url.into_request() {
            req => {
                let url = String::from(req.url());
                let res = get_url_body!(as String, req);
                Json::from_str(&res).expect(&format!("non JSON response from `{}`", url))
            }
        }
    };

    (as String, $url:expr) => {
        match $url.into_request() {
            req => {
                let url = String::from(req.url());
                String::from_utf8(get_url_body!(req))
                    .expect(&format!("non UTF-8 response from `{}`", url))
            }
        }
    };

    ($url:expr) => {
        match $url.into_request() {
            req => {
                let url = String::from(req.url());
                let mut res = req.open()
                    .expect(&format!("could not get `{}`", url));
                let mut out = vec![];
                std::io::copy(&mut res, &mut out)
                    .expect(&format!("could not read response from `{}`", url));
                out
            }
        }
    };
}

// jps = "JSON path string"
macro_rules! jps {
    ($json:expr, [$($path:expr),+]) => {
        $json.find_path(&[$($path),+]).and_then(Json::as_string)
    };
}

#[test]
fn test_user_agent() {
    let res = get_url_body!(as Json, "http://httpbin.org/user-agent");
    assert_eq!(jps!(res, ["user-agent"]), Some(get_url::AGENT));
}
