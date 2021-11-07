use crate::parser;
use crate::utils;

pub fn exit(args: parser::Parser)
{
    let mut peekable = args.peekable();
    if let Some(exit_code) = peekable.peek().as_ref()
    {
        utils::exit(match exit_code.to_string().parse::<i32>(){
            Ok(m) => m,
            Err(_) => {
                utils::zash_error("exit: numeric argument required");
                return;
            }
        });
    } else {
        utils::exit(0);
    }
}