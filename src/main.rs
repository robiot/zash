/** zash - A Zuper Awesome Shell
 * License: GPL-3.0
 * https://github.com/robiot/zash
 */

use signal_hook::{consts, iterator::Signals};
use structopt::StructOpt;

mod opts;
mod utils;
mod shell;

fn signal_handler() {
    Signals::new(&[consts::SIGINT]).unwrap();
}

fn main() {
    opts::Opts::from_args();
    signal_handler();
    shell::shell();
}
