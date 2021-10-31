/** zash - A Zuper Awesome Shell
 * License: GPL-3.0
 * https://github.com/robiot/zash
 */

use signal_hook::{consts, iterator::Signals};
use structopt::StructOpt;

mod opts;
mod shell;
mod utils;
mod parser;
mod builtins;

fn signal_handler() {
    Signals::new(&[consts::SIGINT]).unwrap();
}

fn main() {
    let opts = opts::Opts::from_args();

    if let Some(command) = opts.command {
        shell::run_line(command);
        utils::exit(0);
    }
    
    signal_handler();
    shell::shell();
}
