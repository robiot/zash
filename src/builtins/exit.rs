use crate::utils;

pub fn exit(args: Vec<String>) -> i32
{
    if let Some(exit_code) = args.get(0)
    {
        utils::exit(match exit_code.to_string().parse::<i32>(){
            Ok(m) => m,
            Err(_) => {
                utils::zash_error("exit: numeric argument required");
                2
            }
        });
    } else {
        utils::exit(0);
    }
    0
}