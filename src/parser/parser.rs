use super::*;
use crate::utils;
use glob::glob;

// ^(.?/[^/ ]*)+/?$
pub fn parse_cmd(token: String) -> Vec<(tokens::CmdTokens, String)> {
    let mut combine_value = "".to_string();
    let mut result = Vec::new();
    for part in lexer::cmd_to_tokens(&token).iter().peekable() {
        let mut str_part = part.1.clone();
        // Replace with enviroment variable
        if part.0 == tokens::CmdTokens::Variable {
            str_part = std::env::var(part.1.clone()).unwrap_or_else(|_| "".to_string());
        // Replace ~ with home dir
        } else if part.0 == tokens::CmdTokens::Arg {
            if str_part.starts_with("~") {
                str_part = str_part.split_at(1).1.to_string();
                str_part = format!("{}{}", utils::get_home_dir(), str_part);
            }
            if utils::re_contains(&str_part, "^(.?/[^/ ]*)+/?$") {
                if let Ok(globs) = glob(&str_part) {
                    for entry in globs {
                        if let Ok(entry1) = entry {
                            result.push((part.0, entry1.display().to_string()));
                        }
                    }
                }
                continue;
            }
        }




        if part.2 == true {
            combine_value += &str_part;
        } else {
            result.push((part.0, format!("{}{}", combine_value, str_part)));
            combine_value.clear();
        }
    }
    result
}
