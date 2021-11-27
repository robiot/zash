use std::iter::Peekable;
use std::str::CharIndices;

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
enum ParsingState {
    Normal,
    Escaped,
    SingleQuoted,
    DoubleQuoted,
    DoubleQuotedEscaped,
    Separator,
}

#[derive(Debug)]
pub struct Parser<'a> {
    state: ParsingState,
    cmdline: Peekable<CharIndices<'a>>,
    separator: String,
    keep_escape: bool,
}

impl<'a> Parser<'a> {
    pub fn new(cmdline: &str, sep: String, keep_escape: bool) -> Parser {
        Parser {
            state: ParsingState::Normal,
            cmdline: cmdline.char_indices().peekable(),
            separator: sep,
            keep_escape: keep_escape,
        }
    }
}

// This will be cleaned up
impl<'a> Iterator for Parser<'a> {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        use self::ParsingState::*;
        let mut arg = String::new();

        if let Some(_) = self.cmdline.peek() {
            let mut yield_value = false;
            let mut was_quoted = false;

            for (_, c) in &mut self.cmdline {
                self.state = match (self.state, c) {
                    (Normal, '#') => {
                        break;
                    } // Comment
                    (Normal, '\\') => {
                        if self.keep_escape {
                            arg.push(c);
                        }
                        Escaped
                    }
                    (Normal, '\'') => {
                        if self.keep_escape {
                            arg.push(c);
                        }
                        SingleQuoted
                    }
                    (Normal, '"') => {
                        if self.keep_escape {
                            arg.push(c);
                        }
                        DoubleQuoted
                    }
                    (Normal, ref c) if &self.separator.chars().next().unwrap() == c => {
                        // Ex &&
                        if self.separator.len() > 1 {
                            Separator
                        } else {
                            if arg.len() > 0 || was_quoted {
                                yield_value = true;
                            }
                            Normal
                        }
                    }
                    (Normal, _) | (Escaped, _) => {
                        arg.push(c);
                        Normal
                    }
                    (SingleQuoted, '\'') => {
                        was_quoted = true;
                        Normal
                    }
                    (SingleQuoted, _) => {
                        arg.push(c);
                        SingleQuoted
                    }
                    (DoubleQuoted, '"') => {
                        was_quoted = true;
                        Normal
                    }
                    (DoubleQuoted, '\\') => DoubleQuotedEscaped,
                    (DoubleQuoted, _)
                    | (DoubleQuotedEscaped, '"')
                    | (DoubleQuotedEscaped, '\\') => {
                        arg.push(c);
                        DoubleQuoted
                    }
                    (DoubleQuotedEscaped, _) => {
                        arg.push('\\');
                        arg.push(c);
                        DoubleQuoted
                    }
                    (Separator, _) => {
                        if arg.len() > 0 || was_quoted {
                            yield_value = true;
                        }
                        Normal
                    }
                };

                if yield_value {
                    return Some(arg);
                }
            }

            if arg.len() > 0 || was_quoted {
                return Some(arg);
            }
        }

        None
    }
}


#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum LineTCmdState {
    Normal,
    Escaped,
    SingleQuoted,
    DoubleQuoted,
    DoubleQuotedEscaped,
    WaitingSeparator,
}

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum LineTCmdTokens {
    Command,
    Separator,
}

// Inspired by github.com/mitnk/cicada/blob/master/src/parsers/parser_line.rs
// Splits input into multiple commands
// line_to_cmds("echo hello; echo goodbye")
// [(Command, "echo hello"), (Separator, ";"), (Command, "echo goodbye")]
pub fn line_to_cmds(line: &str) -> Vec<(LineTCmdTokens, std::string::String)> {
    use LineTCmdTokens::*;
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
                token = String::new();
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
                    token = String::new();
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