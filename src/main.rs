/** zash - A Zuper Awesome Shell
 * License: GPL-3.0
 * https://github.com/robiot/zash
 */
use signal_hook::{consts, iterator::Signals};
use structopt::StructOpt;

mod builtins;
mod opts;
mod parser;
mod scripting;
mod shell;
mod utils;

fn signal_handler() {
    Signals::new(&[consts::SIGINT]).unwrap();
}

fn main() {
    let opts = opts::Opts::from_args();

    if let Some(command) = opts.command {
        shell::run_line(command);
        utils::exit(0);
    };

    if let Some(script_file) = opts.script_file {
        if let Err(err) = scripting::run_file(script_file) {
            utils::zash_error(err);
            utils::exit(1);
        }
        utils::exit(0);
    };

    signal_handler();
    shell::shell();
}
