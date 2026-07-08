use pparse::parse_bytes;

fn main() {
    let mut debug = false;
    let mut path: Option<String> = None;

    for arg in std::env::args().skip(1) {
        match arg.as_str() {
            "--debug" | "-d" => debug = true,
            _ if path.is_none() => path = Some(arg),
            _ => {
                eprintln!("usage: pparse [--debug|-d] <path>");
                std::process::exit(2);
            }
        }
    }

    let path = match path {
        Some(path) => path,
        None => {
            eprintln!("usage: pparse [--debug|-d] <path>");
            std::process::exit(2);
        }
    };

    let bytes = std::fs::read(&path).unwrap_or_else(|e| {
        eprintln!("error reading file: {e:?}");
        std::process::exit(1);
    });

    parse_bytes(&bytes, debug);
}
