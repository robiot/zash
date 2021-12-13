use crate::shell;
use crate::utils;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::io::{self, BufRead};
use std::path::Path;

fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    Ok(io::BufReader::new(File::open(filename)?).lines())
}

pub fn run_file(filename: String) -> std::io::Result<()> {
    let mut shell = shell::Shell::new();
    for line in (read_lines(filename)?).flatten() {
        shell.run_line(line);
    }
    Ok(())
}

pub fn load_rc(homedir: String) {
    let rcpath = format!("{}/.zashrc", homedir);
    if !Path::new(&rcpath).exists() {
        let welcometext = "Welcome to zash";
        println!("{}", welcometext);
        let mut file = match OpenOptions::new().create_new(true).write(true).open(rcpath) {
            Ok(m) => m,
            Err(err) => {
                utils::zash_error(err);
                return;
            }
        };
        writeln!(file, "echo {}", welcometext).unwrap();
    } else if let Err(err) = run_file(rcpath.to_string()) {
        utils::zash_error(format!("{}: {}", rcpath, err));
    };
}
