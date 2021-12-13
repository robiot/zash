use colored::Colorize;
use crate::utils;

fn error_cd<T: std::string::ToString>(error: T) {
    utils::zash_error(format!("{}: {}", "cd".red(), error.to_string()));
}

pub fn cd(args: Vec<String>) -> i32
{
    if args.len() > 1 {
        error_cd("too many arguments");    
        return 1;
    }
    
    if let Some(dir) = args.get(0)
    {
        if let Err(err) = std::env::set_current_dir(&std::path::Path::new(&dir)) {
            error_cd(utils::error_string(err.raw_os_error().unwrap()));
            return 1;
        }
    }
    0
}