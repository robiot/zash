use super::tokens;

// Line to commands
#[derive(Clone, Copy, Eq, PartialEq, Debug)]
enum LineTCmdState {
    Normal,
    Escaped,
    SingleQuoted,
    DoubleQuoted,
    DoubleQuotedEscaped,
    WaitingSeparator,
}

// Inspired by github.com/mitnk/cicada/blob/master/src/parsers/parser_line.rs
// Splits input into multiple commands
// line_to_cmds("echo hello; echo goodbye")
// [(Command, "echo hello"), (Separator, ";"), (Command, "echo goodbye")]
pub fn line_to_cmds(line: &str) -> Vec<(tokens::LineToCmdTokens, std::string::String)> {
    use LineTCmdState::*;
    use tokens::LineToCmdTokens::*;
    let mut result = Vec::new();
    let mut token = String::new();
    let mut sep_before: char = '\0';
    let mut state: LineTCmdState = Normal;
    for (_, c) in line.chars().enumerate() {
        state = match (state, c) {
            (Normal, '#') => break,
            (Normal, '\\') => {
                token.push(c);
                Escaped
            }
            (Normal, '\'') => {
                token.push(c);
                SingleQuoted
            }
            (Normal, '"') => {
                token.push(c);
                DoubleQuoted
            }
            (Normal, ';') => {
                if !token.is_empty() {
                    result.push((Command, token.trim().to_string()));
                }
                result.push((Separator, c.to_string()));
                token.clear();
                Normal
            }
            (Normal, _) if c == '&' || c == '|' => {
                sep_before = c;
                WaitingSeparator
            }
            (WaitingSeparator, _) => {
                if sep_before == c {
                    if !token.is_empty() {
                        result.push((Command, token.trim().to_string()));
                    }
                    result.push((Separator, format!("{}{}", c.to_string(), c.to_string())));
                    token.clear();
                } else {
                    token.push(sep_before);
                    token.push(c);
                }
                Normal
            }
            (Normal, _) | (Escaped, _) => {
                token.push(c);
                Normal
            }
            (SingleQuoted, '\'') => {
                token.push(c);
                Normal
            }
            (SingleQuoted, _) => {
                token.push(c);
                SingleQuoted
            }
            (DoubleQuoted, '"') => {
                token.push(c);
                Normal
            }
            (DoubleQuoted, '\\') => DoubleQuotedEscaped,
            (DoubleQuoted, _) | (DoubleQuotedEscaped, '"') | (DoubleQuotedEscaped, '\\') => {
                token.push(c);
                DoubleQuoted
            }
            (DoubleQuotedEscaped, _) => {
                token.push('\\');
                token.push(c);
                DoubleQuoted
            }
        };
    }
    if !token.is_empty() {
        result.push((Command, token.trim().to_string()));
    }
    result
}

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
enum CmdTTokenState {
    Normal,
    Escaped,
    SingleQuoted,
    DoubleQuoted,
    DoubleQuotedEscaped,
    DollarVariable,
}

fn valid_name_check(c: char) -> bool {
    if !c.is_alphanumeric() && c != '_' && c != '$' {
        false
    } else {
        true
    }
}

pub fn is_valid_variable_name(name: String) -> bool {
    for char in name.chars() {
        if !valid_name_check(char) {
            return false;
        }
    }
    true
}

// pub fn split_variable_name(name: String) -> Vec<String> {
//     let mut return_val = vec![String::new(), String::new()];
//     let mut is_valid = true;

//     for char in name.chars() {
//         if is_valid {
//             if !valid_name_check(char) {
//                 is_valid = false;
//                 return_val[1].push(char);
//             } else {
//                 return_val[0].push(char);
//             }
//         } else {
//             return_val[1].push(char);
//         }
//     }
//     return_val
// }

// Not result if not used... fix

type CmdToTokensReturn = (tokens::CmdTokens, std::string::String, bool);

pub fn cmd_to_tokens(line: &str) -> Vec<CmdToTokensReturn> {
    use CmdTTokenState::*;
    use tokens::CmdTokens::*;
    let mut result: Vec<CmdToTokensReturn> = Vec::new();
    let mut token = String::new();
    let mut state: CmdTTokenState = Normal;
    let mut has_command: bool = false;
    let mut is_definition: bool = false;
    for (_, c) in line.chars().enumerate() {
        // println!("state: {:?} -- {}", state, token);
        state = match (state, c) {
            (Normal, '\\') => Escaped,
            (Normal, '\'') => SingleQuoted,
            (Normal, '"') => DoubleQuoted,
            (Normal, c) if c == '>' || c == '<' || c == '|' => {
                if !token.is_empty() {
                    result.push((Command, token.trim().to_string(), false));
                }
                result.push((Pipe, c.to_string(), false));
                token.clear();
                Normal
            }
            (Normal, '=') => {
                // if has_dollar {
                //     return Err(std::io::Error::new(
                //         std::io::ErrorKind::Other,
                //         "Redeclaration of variable can't be done with $",
                //     ));
                // }
                // The value stored in token, should now be the variable name
                if is_valid_variable_name(token.clone()) {
                    // ex TEST? is not valid because of the question mark
                    is_definition = true;
                    token.push(c);
                }
                Normal
            }
            (Normal, '$') => {
                if !token.is_empty() {
                    result.push((Arg, token.trim().to_string(), true));
                }
                token.clear();
                DollarVariable
            }
            (Normal, ' ') => {
                if !token.is_empty() {
                    let token_type: tokens::CmdTokens;
                    if is_definition {
                        is_definition = false;
                        token_type = Definition;
                    } else {
                        if has_command == false {
                            has_command = true;
                            token_type = Command;
                        } else {
                            token_type = Arg;
                        }
                    }
                    result.push((token_type, token.trim().to_string(), false));
                }
                token.clear();

                Normal
            }
            (Normal, _) | (Escaped, _) => {
                token.push(c);
                Normal
            }
            (DollarVariable, ' ') => {
                token.push(c);
                result.push((Variable, token.trim().to_string(), false));
                token.clear();
                Normal
            }
            (DollarVariable, c) if !valid_name_check(c) => {
                result.push((Variable, token.trim().to_string(), true));
                token.clear();
                token.push(c);
                Normal
            }
            (DollarVariable, _) => {
                token.push(c);
                DollarVariable
            }
            (SingleQuoted, '\'') => Normal,
            (SingleQuoted, _) => {
                token.push(c);
                SingleQuoted
            }
            (DoubleQuoted, '"') => Normal,
            (DoubleQuoted, '\\') => DoubleQuotedEscaped,
            (DoubleQuoted, _) | (DoubleQuotedEscaped, '"') | (DoubleQuotedEscaped, '\\') => {
                token.push(c);
                DoubleQuoted
            }
            (DoubleQuotedEscaped, _) => {
                token.push('\\');
                token.push(c);
                DoubleQuoted
            }
        };
    }

    // yes this is duplicated code, I will have to figure out a way to
    // make it in a better way
    if state == Normal {
        if !token.is_empty() {
            let token_type: tokens::CmdTokens;
            if is_definition {
                token_type = Definition;
            } else {
                if has_command == false {
                    token_type = Command;
                } else {
                    token_type = Arg;
                }
            }
            result.push((token_type, token.trim().to_string(), false));
        }
    }
    else if state == DollarVariable {
        if !token.is_empty() {
            result.push((Variable, token.trim().to_string(), false));
        }
    }
    result
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_line_to_cmds() {
        fn s(input: &str) -> String {
            input.to_string()
        }
        use super::line_to_cmds;
        use super::tokens::LineToCmdTokens::*;

        let v = vec![
            ("   ls  ", vec![(Command, s("ls"))]), // Trim input
            ("ls", vec![(Command, s("ls"))]),
            (
                "echo morning & echo night",
                vec![(Command, s("echo morning & echo night"))],
            ),
            (
                "echo morning && echo night",
                vec![
                    (Command, s("echo morning")),
                    (Separator, s("&&")),
                    (Command, s("echo night")),
                ],
            ),
            ("ls | grep .bashrc", vec![(Command, s("ls | grep .bashrc"))]),
            // Quotes
            (
                r#"echo "What an awesome day && nice weather""#,
                vec![(Command, s(r#"echo "What an awesome day && nice weather""#))],
            ),
            (
                r#"echo 'What an awesome day && nice weather'"#,
                vec![(Command, s(r#"echo 'What an awesome day && nice weather'"#))],
            ),
            // Escape
            (
                r#"echo \"What an awesome day && nice weather\""#,
                vec![
                    (Command, s(r#"echo \"What an awesome day"#)),
                    (Separator, s("&&")),
                    (Command, s(r#"nice weather\""#)),
                ],
            ),
            (
                r#"echo What an awesome day \&\& nice weather"#,
                vec![(Command, s(r#"echo What an awesome day \&\& nice weather"#))],
            ),
            (";", vec![(Separator, s(";"))]),
        ];

        for (l, r) in v {
            assert_eq!(line_to_cmds(l), r);
        }
    }
}
