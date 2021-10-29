use dirs::home_dir;
use rustyline::error::ReadlineError;
use rustyline::Editor;
use signal_hook::{consts, iterator::Signals};
use std::fs::File;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::io::{self, BufRead};
use std::path::Path;
use std::process::{Child, Command, Stdio};
use structopt::StructOpt;
use whoami;

mod opts;

fn zash_error<T: std::string::ToString>(error: T) {
    eprintln!("zash: {}", error.to_string());
}

const PARSE_LINE_SUCCESS: i16 = 0;
const PARSE_LINE_CONTINUE: i16 = 1;
const PARSE_LINE_BREAK: i16 = 2;

fn parse_line(homedir: String, line: String) -> i16 {
    let mut commands = line.trim().split("|").peekable();
    let mut prev_command = None;

    while let Some(command) = commands.next() {
        let mut parts = command.trim().split_whitespace();
        let command = match parts.next() {
            Some(n) => n,
            None => return 1,
        };
        match command {
            // Builtins
            "cd" => {
                let new_dir = match parts.peekable().peek() {
                    Some(&m) => m,
                    None => return PARSE_LINE_CONTINUE,
                };
                let dir = new_dir.to_string().replace("~", &homedir.to_string());
                let root = std::path::Path::new(&dir);
                if let Err(_) = std::env::set_current_dir(&root) {
                    println!("cd: no such file or directory: {}", dir);
                }
            }

            "exit" => return PARSE_LINE_BREAK,

            command => {
                let stdin = prev_command.map_or(Stdio::inherit(), |output: Child| {
                    Stdio::from(output.stdout.unwrap())
                });

                let stdout = if commands.peek().is_some() {
                    Stdio::piped()
                } else {
                    Stdio::inherit()
                };

                match Command::new(command)
                    .args(parts)
                    .stdin(stdin)
                    .stdout(stdout)
                    .spawn()
                {
                    Ok(output) => {
                        prev_command = Some(output);
                    }
                    Err(_) => {
                        prev_command = None;
                        zash_error(format!("command not found: {}", command));
                    }
                };
            }
        }
    }

    if let Some(mut final_command) = prev_command {
        final_command.wait().unwrap();
    }

    return PARSE_LINE_SUCCESS;
}

fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

fn load_zashrc(homedir: String) {
    let rcpath = &format!("{}/.zashrc", homedir);
    if let Ok(lines) = read_lines(rcpath) {
        for line in lines {
            if let Ok(ip) = line {
                parse_line(homedir.to_string(), ip);
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

fn signal_handler() {
    Signals::new(&[consts::SIGINT]).unwrap();
}

fn main() {
    opts::Opts::from_args();
    signal_handler();
    let mut rl = Editor::<()>::new();
    if home_dir().is_none() {
        zash_error(
            "Home directory could not be found. Make sure you have a folder for your user in /home",
        );
        return;
    }
    let homedir_pathbuf = home_dir().unwrap();
    let homedir = homedir_pathbuf.display();
    load_zashrc(homedir.to_string());
    let hispath = format!("{}/.zash_history", homedir);
    if rl.load_history(&hispath).is_err() {
        zash_error("No previous history");
    }
    loop {
        let readline = rl.readline(
            format!(
                "{}@{} {} $ ",
                whoami::username().as_str(),
                whoami::hostname().as_str(),
                std::env::current_dir()
                    .unwrap()
                    .display()
                    .to_string()
                    .replace(&homedir.to_string(), "~")
            )
            .as_str(),
        );
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str());
                match parse_line(homedir.to_string(), line) {
                    PARSE_LINE_SUCCESS => {}
                    PARSE_LINE_CONTINUE => continue,
                    PARSE_LINE_BREAK => break,
                    _ => break,
                }
            }
            Err(ReadlineError::Interrupted) => {
                continue;
            }
            Err(ReadlineError::Eof) => {
                println!("exit");
                break;
            }
            Err(err) => {
                zash_error(format!("Error: {:?}", err));
                break;
            }
        }
        rl.save_history(&hispath).unwrap();
    }
}
