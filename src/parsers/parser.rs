use super::errors::*;
use super::*;
use crate::utils;
use glob::glob;

// Todo: rustyline escape star character in filenames
// -> (Command: ["echo", "wow"]), (Separator: [">"]), (Command: ["echo", "goodbye"])
pub fn parse_cmd(token: String, status: i32) -> Result<Vec<(tokens::ParseCmdTokens, Vec<String>)>> {
    let mut combine_value = "".to_string();
    let mut result = Vec::new();
    let mut result_part: Vec<String> = Vec::new();
    let mut before_token: Option<tokens::CmdTokens> = None;
    let mut is_definition: bool = false;
    // Todo: part should give boolean if escaped/quoted or not, for wildcards, variables and ~
    for part in lexer::cmd_to_tokens(&token)?.iter().peekable() {
        before_token = match part.0 {
            tokens::CmdTokens::Pipe => {
                if before_token.is_none() || before_token == Some(tokens::CmdTokens::Pipe) {
                    return Err(SyntaxError); // Ex "> echo lol"
                }
                if !result_part.is_empty() {
                    result.push((tokens::ParseCmdTokens::Command, result_part.clone()));
                    result_part.clear();
                }
                // The separator has to be put in a vec
                result.push((tokens::ParseCmdTokens::Separator, vec![part.1.clone()]));
                Some(part.0)
            }
            _ => {
                let mut str_part = part.1.clone();
                if part.0 == tokens::CmdTokens::Definition {
                    is_definition = true;
                }

                // Replace with enviroment variable
                if part.0 == tokens::CmdTokens::Variable {
                    if part.1.clone() == "?" {
                        str_part = status.to_string();
                    } else {
                        str_part = std::env::var(part.1.clone()).unwrap_or_else(|_| "".to_string());
                    }
                // Replace ~ with home dir
                } else if part.0 == tokens::CmdTokens::Normal && str_part.starts_with('~') {
                    str_part = str_part.split_at(1).1.to_string();
                    str_part = format!("{}{}", utils::get_home_dir(), str_part);
                }

                if part.2 {
                    combine_value += &str_part;
                } else {
                    let val = format!("{}{}", combine_value, str_part);

                    if is_definition {
                        is_definition = false;
                        // For now all variables are exported / enviroment variables
                        // Todo: Add shell variables
                        let definition_parts: Vec<&str> = val.split('=').collect();
                        // Could maybe happen? thread 'main' panicked at 'index out of bounds: the len is 1 but the index is 1'
                        std::env::set_var(definition_parts[0], definition_parts[1]);
                    } else {
                        // Glob paths. ex ./*.md
                        if let Ok(globs) = glob(&val) {
                            let mut has_entry = false;
                            for entry in globs.flatten() {
                                has_entry = true;
                                result_part.push(entry.display().to_string());
                            }
                            if !has_entry {
                                result_part.push(val);
                            }
                        }
                    }
                    combine_value.clear();
                }
                Some(part.0)
            }
        };
    }
    // Ex "hello |"
    if before_token == Some(tokens::CmdTokens::Pipe) {
        return Err(SyntaxError);
    }

    if !result_part.is_empty() {
        result.push((tokens::ParseCmdTokens::Command, result_part.clone()));
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    macro_rules! string_vec {
        ($($x:expr),*) => (vec![$($x.to_string()),*]);
    }

    #[test]
    fn test_parser() {
        use super::parse_cmd;
        use super::tokens::ParseCmdTokens::*;

        let v = vec![
            (
                "echo hello world",
                vec![(Command, string_vec!["echo", "hello", "world"])],
            ), // Trim input
            (
                "echo $tesrakijds",
                vec![(Command, string_vec!["echo", "hello"])],
            ), // Test enviroment variables
            (
                "echo /home/$tesrakijds",
                vec![(Command, string_vec!["echo", "/home/hello"])],
            ), // Test combine with one before
            (
                "echo $tesrakijds/.config",
                vec![(Command, string_vec!["echo", "hello/.config"])],
            ), // Test combine with one after
            (
                "echo /home/$tesrakijds/.config",
                vec![(Command, string_vec!["echo", "/home/hello/.config"])],
            ), // Test combine with one before & one after
            (
                "echo 'hello world'",
                vec![(Command, string_vec!["echo", "hello world"])],
            ), // Single quotes
            (
                "echo \"hello world\"",
                vec![(Command, string_vec!["echo", "hello world"])],
            ), // Double Quotes
            (
                "echo hello\\ world",
                vec![(Command, string_vec!["echo", "hello world"])],
            ), // Escaped space
            ("TEST=$tesrakijds:/root/.config", vec![]), // Define variable with another variable
        ];

        std::env::set_var("tesrakijds", "hello"); // Random name, for enviroment variables test
        for (l, r) in v {
            assert_eq!(parse_cmd(l.to_string(), 0).unwrap(), r);
        }
    }
}
