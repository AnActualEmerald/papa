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
