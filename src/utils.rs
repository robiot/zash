use colored::Colorize;
use dirs::home_dir;
use std::ffi::CStr;
use std::os::raw::{c_char, c_int};
use std::str;

// https://stackoverflow.com/questions/40710115/how-does-one-get-the-error-message-as-provided-by-the-system-without-the-os-err
// from https://github.com/rust-lang/rust/blob/1.26.2/src/libstd/sys/unix/os.rs#L87-L107
pub fn error_string(errno: i32) -> String {
    extern "C" {
        #[cfg_attr(
            any(target_os = "linux", target_env = "newlib"),
            link_name = "__xpg_strerror_r"
        )]
        fn strerror_r(errnum: c_int, buf: *mut c_char, buflen: libc::size_t) -> c_int;
    }

    let mut buf = [0 as c_char; 128];

    let p = buf.as_mut_ptr();
    unsafe {
        assert!(!(strerror_r(errno as c_int, p, buf.len()) < 0), "strerror_r failure");

        let p = p as *const _;
        str::from_utf8(CStr::from_ptr(p).to_bytes())
            .unwrap()
            .to_owned()
    }
}

pub fn exit(code: i32) {
    std::process::exit(code);
}

pub fn zash_error<T: std::string::ToString>(error: T) {
    eprintln!("{}: {}", "zash".red(), error.to_string());
}

pub fn get_home_dir() -> String {
    if home_dir().is_none() {
        zash_error(
            "Home directory could not be found. Make sure you have a folder for your user in /home",
        );
        exit(1);
    }
    let homedir_pathbuf = home_dir().unwrap();
    return homedir_pathbuf.display().to_string();
}
