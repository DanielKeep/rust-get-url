extern crate clap;
extern crate get_url;

type Error = Box<std::error::Error>;

fn main() {
    match try_main() {
        Ok(()) => (),
        Err(err) => {
            use std::io::Write;
            let mut out = std::io::stderr();
            let _ = writeln!(out, "get-url: {}", err);
        }
    }
}

fn try_main() -> Result<(), Error> {
    use clap::Arg;
    let args = clap::App::new("get-url")
        .version(env!("CARGO_PKG_VERSION"))
        .arg(Arg::with_name("URL")
            .help("URL to fetch")
            .required(true)
        )
        .get_matches();

    let url = args.value_of("URL").unwrap();

    let mut res = try!(get_url::Request::new(url)
        .open());

    let out = std::io::stdout();
    let mut out = out.lock();

    try!(std::io::copy(&mut res, &mut out));

    Ok(())
}
