
use std::fs::File;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::io::{self, BufRead};
use std::path::Path;
use colored::Colorize;
use crate::shell;

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
                shell::parse_line(homedir.to_string(), ip);
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