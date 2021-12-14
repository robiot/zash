// Not really a lexer lexer but it tokenizes the input.
use super::errors::*;
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
    use tokens::LineToCmdTokens::*;
    use LineTCmdState::*;
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
                token.push(c); // If its eoi
                WaitingSeparator
            }
            (WaitingSeparator, _) => {
                if sep_before == c {
                    token.pop();
                    if !token.is_empty() {
                        result.push((Command, token.trim().to_string()));
                    }
                    result.push((Separator, format!("{}{}", c.to_string(), c.to_string())));
                    token.clear();
                } else {
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
    (c.is_alphanumeric() || c == '_' || c == '?') && c != '$'
}

pub fn is_valid_variable_name(name: String) -> bool {
    for char in name.chars() {
        if !valid_name_check(char) {
            return false;
        }
    }
    true
}

type CmdToTokensReturn = (tokens::CmdTokens, std::string::String, bool);

pub fn cmd_to_tokens(line: &str) -> Result<Vec<CmdToTokensReturn>> {
    use tokens::CmdTokens;
    use CmdTTokenState::*;
    let mut result: Vec<CmdToTokensReturn> = Vec::new();
    let mut token = String::new();
    let mut state: CmdTTokenState = Normal;
    let mut is_definition: bool = false;
    for (_, c) in line.chars().enumerate() {
        state = match (state, c) {
            (Normal, '\\') | (DollarVariable, '\\') => Escaped,
            (Normal, '\'') | (DollarVariable, '\'') => SingleQuoted,
            (Normal, '"') | (DollarVariable, '"') => DoubleQuoted,
            (Normal, c) if c == '>' || c == '<' || c == '|' => {
                if !token.is_empty() {
                    result.push((CmdTokens::Normal, token.trim().to_string(), false));
                }
                result.push((CmdTokens::Pipe, c.to_string(), false));
                token.clear();
                Normal
            }
            (Normal, '=') => {
                // The value stored in token, should now be the variable name
                if is_valid_variable_name(token.clone()) {
                    // ex TEST? is not valid because of the question mark
                    is_definition = true;
                    token.push(c);
                }
                Normal
            }
            // Todo: Variables should be supported everywhere
            // echo ${PATH}a
            // echo $USER$PWD
            // (Normal, "echo", false)
            // (Variable, "USER", true)
            // (Normal, "$PWD", false)
            (Normal, ' ') | (Normal, '$') => {
                if !token.is_empty() {
                    let token_type: tokens::CmdTokens;
                    if is_definition {
                        is_definition = false;
                        token_type = CmdTokens::Definition;
                    } else {
                        token_type = CmdTokens::Normal;
                    }
                    result.push((token_type, token.trim().to_string(), c == '$'));
                }
                token.clear();

                if c == ' ' {
                    Normal
                } else {
                    DollarVariable
                }
            }
            (Normal, _) | (Escaped, _) => {
                token.push(c);
                Normal
            }
            (DollarVariable, ' ') => {
                token.push(c);
                result.push((CmdTokens::Variable, token.trim().to_string(), false));
                token.clear();
                Normal
            }
            
            (DollarVariable, c) if !valid_name_check(c) => {
                result.push((CmdTokens::Variable, token.trim().to_string(), true));
                token.clear();
                token.push(c); // Todo: See if variable
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

    match state {
        Normal => {
            if !token.is_empty() {
                let token_type: tokens::CmdTokens;
                if is_definition {
                    token_type = CmdTokens::Definition;
                } else {
                    token_type = CmdTokens::Normal;
                }
                result.push((token_type, token.trim().to_string(), false));
            }
        }
        DollarVariable => {
            if !token.is_empty() {
                result.push((CmdTokens::Variable, token.trim().to_string(), false));
            }
        }
        _ => {
            // println!("{:?}", state);
            // Todo: Add more information on error, SyntaxError near token Pipe
            return Err(SyntaxError);
        }
    }
    Ok(result)
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
