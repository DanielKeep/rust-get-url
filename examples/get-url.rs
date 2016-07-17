extern crate clap;
extern crate get_url;

type Error = Box<std::error::Error>;

const HEADER_ARG_SEP: &'static [char] = &['=', ':'];

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
        .arg(Arg::with_name("header")
            .help("Specify an additional header to send.")
            .short("h")
            .long("header")
            .takes_value(true)
            .multiple(true)
            .number_of_values(1)
        )
        .get_matches();

    let url = args.value_of("URL").unwrap();

    let mut req = get_url::Request::new(url);

    if let Some(headers) = args.values_of("header") {
        for header in headers {
            let mut parts = header.splitn(2, HEADER_ARG_SEP);
            let name = parts.next().expect(&format!("got header without name: {:?}", header));
            let value = parts.next().expect(&format!("got header without value: {:?}", header));
            req.set_header(name, value);
        }
    }

    let mut res = try!(req.open());

    let out = std::io::stdout();
    let mut out = out.lock();

    try!(std::io::copy(&mut res, &mut out));

    Ok(())
}
