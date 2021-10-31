use crate::shell;
use colored::Colorize;
use dirs::home_dir;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::io::{self, BufRead};
use std::path::Path;

pub fn exit(code: i32) {
    std::process::exit(code);
}

pub fn zash_error<T: std::string::ToString>(error: T) {
    eprintln!("{}: {}", "zash".red(), error.to_string());
}

fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

pub fn load_zashrc(homedir: String) {
    let rcpath = &format!("{}/.zashrc", homedir);
    if let Ok(lines) = read_lines(rcpath) {
        for line in lines {
            if let Ok(ip) = line {
                shell::run_line(ip);
            }
        }
    } else {
        let welcometext = "Welcome to zash";
        println!("{}", welcometext);
        let mut file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(rcpath)
            .unwrap();
        writeln!(file, "echo {}", welcometext).unwrap();
    }
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
