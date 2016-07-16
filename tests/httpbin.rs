extern crate get_url;
extern crate rustc_serialize;

use rustc_serialize::json::Json;

macro_rules! get_url_body {
    (as Json, $url:expr) => {
        match $url {
            url => {
                let res = get_url_body!(as String, &url);
                Json::from_str(&res).expect(&format!("non JSON response from `{}`", url))
            }
        }
    };

    (as String, $url:expr) => {
        match $url {
            url => {
                String::from_utf8(get_url_body!(&url))
                    .expect(&format!("non UTF-8 response from `{}`", url))
            }
        }
    };

    ($url:expr) => {
        match $url {
            url => {
                let mut res = get_url::Request::new(&url)
                    .open()
                    .expect(&format!("could not get `{}`", url));
                let mut out = vec![];
                std::io::copy(&mut res, &mut out)
                    .expect(&format!("could not read response from `{}`", url));
                out
            }
        }
    };
}

#[test]
fn test_user_agent() {
    let res = get_url_body!(as Json, "http://httpbin.org/user-agent");
    let res = res.find("user-agent")
        .and_then(Json::as_string);
    assert_eq!(res, Some(get_url::AGENT));
}
