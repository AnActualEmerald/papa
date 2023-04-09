#[macro_export]
macro_rules! readln {
    ($l:literal) => {{
        use std::io;
        use std::io::Write;
        let mut input = String::new();

        let stdin = io::stdin();
        print!("{}", $l);
        if let Err(e) = io::stdout().flush() {
            Err(e)
        } else {
            match stdin.read_line(&mut input) {
                Ok(_) => Ok(input),
                Err(e) => Err(e),
            }
        }
    }};
}

#[macro_export]
macro_rules! flush {
    () => {{
        use std::io::Write;
        std::io::stdout().flush()
    }};
}

#[macro_export]
macro_rules! modfile {
    ($path:expr) => {{
        use std::fs::OpenOptions;
        OpenOptions::new()
            .write(true)
            .create(true)
            .read(true)
            .truncate(true)
            .open($path)
    }};
    (wo, $path:expr) => {{
        use std::fs::OpenOptions;
        OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open($path)
    }};
    (ro, $path:expr) => {{
        use std::fs::OpenOptions;
        OpenOptions::new().create(true).read(true).open($path)
    }};
    (o, $path:expr) => {{
        use std::fs::OpenOptions;
        OpenOptions::new().read(true).write(true).open($path)
    }};
}

#[macro_export]
macro_rules! get_answer {
    ($yes:expr) => {
        get_answer!($yes, "OK? [Y/n]: ")
    };
    ($yes:expr, $msg: literal) => {
        if $yes {
            Ok(String::new())
        } else {
            $crate::readln!($msg)
        }
    };
}
