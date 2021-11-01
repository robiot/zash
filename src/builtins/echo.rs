use crate::parser;

pub fn echo(args: parser::Parser) {
    let mut str_args: String = String::new();    
    for word in args {
        str_args += word.as_str();
    }
    println!("{}", str_args);
}
