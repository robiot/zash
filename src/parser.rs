use std::collections::HashSet;
use std::iter::Peekable;
use std::str::CharIndices;

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
enum ParsingState {
    Normal,
    Escaped,
    SingleQuoted,
    DoubleQuoted,
    DoubleQuotedEscaped,
}

#[derive(Debug)]
pub struct Parser<'a> {
    state: ParsingState,
    cmdline: Peekable<CharIndices<'a>>,
    separators: HashSet<char>,
}

impl<'a> Parser<'a> {
    pub fn new(cmdline: &str) -> Parser {
        Parser {
            state: ParsingState::Normal,
            cmdline: cmdline.char_indices().peekable(),
            separators: [' '].iter().cloned().collect(),
        }
    }
}


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
                    (Normal, '\\') => Escaped,
                    (Normal, '\'') => SingleQuoted,
                    (Normal, '"') => DoubleQuoted,
                    (Normal, ref c) if self.separators.contains(c) => {
                        if arg.len() > 0 || was_quoted {
                            yield_value = true;
                        }
                        Normal
                    },
                    (Normal, _) |
                    (Escaped, _) => { arg.push(c); Normal },
                    (SingleQuoted, '\'') => { was_quoted = true; Normal },
                    (SingleQuoted, _) => { arg.push(c); SingleQuoted },
                    (DoubleQuoted, '"') => { was_quoted = true; Normal },
                    (DoubleQuoted, '\\') => DoubleQuotedEscaped,
                    (DoubleQuoted, _) |
                    (DoubleQuotedEscaped, '"') |
                    (DoubleQuotedEscaped, '\\') => { arg.push(c); DoubleQuoted },
                    (DoubleQuotedEscaped, _) => {
                        arg.push('\\');
                        arg.push(c);
                        DoubleQuoted
                    },
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