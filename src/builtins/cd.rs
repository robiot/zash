use colored::Colorize;
use crate::parser;
use crate::utils;

pub fn cd(args: parser::Parser) -> i32
{
    let homedir = utils::get_home_dir();
    let mut peekable = args.peekable();
    if let Some(new_dir) = peekable.peek().as_ref()
    {
        let dir = new_dir.to_string().replace("~", &homedir.to_string());
        let root = std::path::Path::new(&dir);
        if let Err(_) = std::env::set_current_dir(&root) {
            println!("{}: no such file or directory: {}", "cd".red(), dir);
            return 1;
        }
    }
    0
}