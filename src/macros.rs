#[macro_export]
macro_rules! readln {
    ($l:expr) => {{
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
    (z, $path:expr) => {{
        use std::fs::OpenOptions;
        OpenOptions::new().read(true).write(true).open($path)
    }};
}

#[macro_export]
macro_rules! get_answer {
    ($yes:expr) => {
        get_answer!($yes, "OK? [Y/n]: ")
    };
    (yes:expr, $msg:literal) => {
        get_answer!($yes, format!($msg))
    };
    ($yes:expr, $msg:expr) => {
        if $yes {
            Ok(String::new())
        } else {
            $crate::readln!($msg)
        }
    };
    ($yes:expr, $msg:literal, $($arg:expr),*) => {
        get_answer!($yes, format!($msg, $($arg,)*))
    }
}

#[macro_export]
macro_rules! update_cfg {
    (@cmd $ctx:ident, ignore $dir:expr) => {
            $ctx.add_ignored($dir)
    };
    (@cmd $ctx:ident, unignore $dir:expr) => {
            $ctx.remove_ignored($dir)
    };
    (@cmd $ctx:ident, profile $val:expr) => {
            $ctx.set_current_profile($val)
    };
    (@cmd $ctx:ident, game_dir $dir:expr) => {
            $ctx.set_game_dir($dir)
    };
    (@cmd $ctx:ident, install_dir $dir:expr) => {
            $ctx.set_install_dir($dir)
    };
    (@cmd $ctx:ident, install_type $type:expr) => {
        $ctx.set_install_type($type)
    };
    (@cmd $ctx:ident, server $val:expr) => {
        $ctx.set_is_server($val)
    };
    ($($cmd:ident($op:tt)),*) => {{
        let mut c = $crate::config::CONFIG.clone();
        $(update_cfg!(@cmd c, $cmd $op);)*
        c.save()
    }};
}
