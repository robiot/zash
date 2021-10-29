use dirs::home_dir;
use rustyline::error::ReadlineError;
use rustyline::Editor;
use signal_hook::{consts, iterator::Signals};
use std::process::Command;
use whoami;

fn zash_error<T: std::string::ToString>(error: T) {
    eprintln!("zash: {}", error.to_string());
}

fn signal_handler() {
    Signals::new(&[consts::SIGINT]).unwrap();
}

fn main() {
    signal_handler();

    println!("Welcome to zash");
    let mut rl = Editor::<()>::new();
    if home_dir().is_none() {
        zash_error(
            "Home directory could not be found. Make sure you have a folder for your user in /home",
        );
        return;
    }

    let hispath = format!("{}/.zash_history", home_dir().unwrap().display());
    if rl.load_history(&hispath).is_err() {
        zash_error("No previous history");
    }
    loop {
        let readline = rl.readline(
            format!(
                "{}@{} {} $ ",
                whoami::username().as_str(),
                whoami::hostname().as_str(),
                std::env::current_dir().unwrap().display().to_string().replace(&home_dir().unwrap().display().to_string(), "~")
            )
            .as_str(),
        );
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str());
                let mut parts = line.trim().split_whitespace();
                let command = match parts.next() {
                    Some(n) => n,
                    None => continue,
                };
                match command {
                    
                    // Builtins 
                    "cd" => {
                        let new_dir = match parts.peekable().peek() {
                            Some(&m) => m,
                            None => continue,
                        };
                        let dir = new_dir.to_string().replace("~", &home_dir().unwrap().display().to_string());
                        let root = std::path::Path::new(&dir);
                        if let Err(_) = std::env::set_current_dir(&root) {
                            println!("cd: no such file or directory: {}", dir);
                        }
                    }

                    "exit" => break,

                    command => {
                        let mut child = match Command::new(command).args(parts).spawn() {
                            Ok(m) => m,
                            Err(_) => {
                                zash_error(format!("command not found: {}", command));
                                continue;
                            }
                        };
                        child.wait().unwrap();
                    }
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
    }
    rl.save_history(&hispath).unwrap();
}
