use colored::Colorize;
use dirs::home_dir;

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
