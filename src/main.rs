/** zash - A Zuper Awesome Shell
 * License: GPL-3.0
 * https://github.com/robiot/zash
 */

use signal_hook::{consts, iterator::Signals};
use structopt::StructOpt;
use dirs::home_dir;

mod opts;
mod shell;
mod utils;
mod parser;

fn signal_handler() {
    Signals::new(&[consts::SIGINT]).unwrap();
}

fn exit(code: i32)
{
    std::process::exit(code);
}

fn main() {
    let opts = opts::Opts::from_args();

    if home_dir().is_none() {
        utils::zash_error(
            "Home directory could not be found. Make sure you have a folder for your user in /home",
        );
        exit(1);
    }
    let homedir_pathbuf = home_dir().unwrap();
    let homedir = homedir_pathbuf.display();

    if let Some(command) = opts.command {
        shell::parse_line(homedir.to_string(), command);
        exit(0);
    }
    
    signal_handler();
    shell::shell(homedir);
}
