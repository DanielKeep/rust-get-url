macro_rules! stream_latin_1 {
    ($s:expr, |$b_arg:ident| $b_body:expr, |$e_arg:ident| $e_body:expr) => {
        {
            use std::convert::AsRef;
            match $s {
                ref s => {
                    let s = AsRef::<str>::as_ref(s);
                    for c in s.chars() {
                        if c as u32 >= 0x100 {
                            let $e_arg = s;
                            $e_body;
                        } else {
                            let $b_arg = c as u8;
                            $b_body;
                        }
                    }
                }
            }
        }
    };
}

macro_rules! push_latin_1 {
    ($v:expr, $s:expr) => {
        stream_latin_1! {
            $s,
            |b| $v.push(b as u16),
            |s| return Err(::std::io::Error::new(
                ::std::io::ErrorKind::Other,
                format!("non ISO 8859-1 character in header: {:?}", s)))
        }
    }
}
