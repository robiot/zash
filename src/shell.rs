use colored::Colorize;
use parsers::tokens::*;
use rustyline::completion::{Completer, Pair, ShellCompleter};
use rustyline::config::OutputStreamType;
use rustyline::error::ReadlineError;
use rustyline::highlight::{Highlighter, MatchingBracketHighlighter};
use rustyline::hint::{Hinter, HistoryHinter};
use rustyline::validate::{MatchingBracketValidator, Validator};
use rustyline::{CompletionType, Config, Context, EditMode, Editor};
use rustyline_derive::Helper;
use std::borrow::Cow::{self, Borrowed, Owned};
use std::process::{Child, Command, Stdio};
// use std::collections::HashMap;

use crate::builtins;
use crate::parsers;
use crate::scripting;
use crate::utils;

#[derive(Debug, Clone)]
pub struct Shell {
    // pub variables: HashMap<String, String>,
    pub status: i32,
}

impl Shell {
    pub fn new() -> Self {
        Self {
            // variables: HashMap::new()
            status: 0,
        }
    }

    pub fn run_line(&mut self, line: String) {
        let mut sep = String::new();
        for token in parsers::lexer::line_to_cmds(line.trim()) {
            if token.0 == LineToCmdTokens::Separator {
                sep = token.1.clone();
                continue;
            }
            // "&& "Don't run the other commands if the one before failed
            //
            // "||" = "Or"
            // "ls || dir"
            // If ls does not succed it runs "dir"
            // If ls succed it does not run dir
            if (sep == "&&" && self.status != 0) || (sep == "||" && self.status == 0) {
                break;
            }
            self.status = Self::exec_command(self, token);
        }
    }
    fn exec_command(&mut self, token: (LineToCmdTokens, std::string::String)) -> i32 {
        let parts = match parsers::parser::parse_cmd(token.1, self.status) {
            Ok(m) => m,
            Err(err) => {
                utils::zash_error(err);
                return 0;
            }
        };
        // println!("{:?}", parts);
        let mut prev_command = None;
        for (i, part) in parts.iter().enumerate() {
            match part.0 {
                ParseCmdTokens::Command => {
                    let mut args = part.1.clone();
                    // Probably a very hacky way of getting first arg then removing it
                    let clon = args.clone(); // So it lives longer
                    let command = match clon.get(0) {
                        Some(m) => m,
                        None => return 1,
                    };
                    args.remove(0);
                    let status = match command.as_ref() {
                        // Builtins
                        "cd" => builtins::cd::cd(args),
                        "exit" => builtins::exit::exit(args),
                        command => {
                            let stdin = prev_command.map_or(Stdio::inherit(), |output: Child| {
                                Stdio::from(output.stdout.unwrap())
                            });
                            let stdout = if parts.get(i + 1).is_some()
                                && parts.get(i + 1).unwrap().0 == ParseCmdTokens::Separator
                            {
                                Stdio::piped()
                            } else {
                                Stdio::inherit()
                            };
                            // If application does not print something with a new line at end, it would get overwritten by the shell
                            match Command::new(command)
                                .args(args.clone())
                                .stdin(stdin)
                                .stdout(stdout)
                                .spawn()
                            {
                                Ok(output) => {
                                    prev_command = Some(output);
                                }
                                Err(_) => {
                                    // prev_command = None;
                                    utils::zash_error(format!("command not found: {}", command));
                                    return 1;
                                }
                            };
                            0
                        }
                    };
                    if status != 0 {
                        return status;
                    }
                }
                ParseCmdTokens::Separator => {
                    // Maybe do something with redirects
                    if part.1.clone() != vec!["|"] {
                        utils::zash_error("this feature is currently not implemented");
                        return 1;
                    }
                }
            }
        }
        if let Some(final_command) = prev_command {
            if let Some(status_code) = final_command.wait_with_output().unwrap().status.code() {
                return status_code;
            }
        }
        127
    }
}

#[derive(Helper)]
struct ShellHelper {
    completer: ShellCompleter,
    highlighter: MatchingBracketHighlighter,
    validator: MatchingBracketValidator,
    hinter: HistoryHinter,
    prompt: String,
}

impl Completer for ShellHelper {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        ctx: &Context<'_>,
    ) -> Result<(usize, Vec<Pair>), ReadlineError> {
        self.completer.complete(line, pos, ctx)
    }
}

impl Hinter for ShellHelper {
    type Hint = String;

    fn hint(&self, line: &str, pos: usize, ctx: &Context<'_>) -> Option<String> {
        self.hinter.hint(line, pos, ctx)
    }
}

impl Highlighter for ShellHelper {
    fn highlight_prompt<'b, 's: 'b, 'p: 'b>(
        &'s self,
        prompt: &'p str,
        default: bool,
    ) -> Cow<'b, str> {
        if default {
            Borrowed(&self.prompt)
        } else {
            Borrowed(prompt)
        }
    }

    fn highlight_hint<'h>(&self, hint: &'h str) -> Cow<'h, str> {
        Owned(format!("{}", hint.dimmed()))
    }

    fn highlight<'l>(&self, line: &'l str, pos: usize) -> Cow<'l, str> {
        self.highlighter.highlight(line, pos)
    }

    fn highlight_char(&self, line: &str, pos: usize) -> bool {
        self.highlighter.highlight_char(line, pos)
    }
}

impl Validator for ShellHelper {
    // This is commented because of issue (#5)
    // fn validate(
    //     &self,
    //     ctx: &mut validate::ValidationContext,
    // ) -> rustyline::Result<validate::ValidationResult> {
    //     self.validator.validate(ctx)
    // }

    fn validate_while_typing(&self) -> bool {
        self.validator.validate_while_typing()
    }
}

pub fn shell() {
    let homedir = utils::get_home_dir();
    let config = Config::builder()
        .history_ignore_space(true)
        .completion_type(CompletionType::List)
        //.complete_path(true)
        .edit_mode(EditMode::Emacs)
        .output_stream(OutputStreamType::Stdout)
        .build();

    let helper = ShellHelper {
        completer: ShellCompleter::new(),
        highlighter: MatchingBracketHighlighter::new(),
        hinter: HistoryHinter {},
        prompt: "".to_owned(),
        validator: MatchingBracketValidator::new(),
    };

    let mut rl = Editor::with_config(config);
    rl.set_helper(Some(helper));

    scripting::load_rc(homedir.clone());
    let hispath = format!("{}/.zash_history", homedir);
    if rl.load_history(&hispath).is_err() {
        utils::zash_error("No previous history");
    }
    let mut shell = Shell::new();

    loop {
        let mut current_dir = std::env::current_dir().unwrap().display().to_string();
        if current_dir.starts_with(&homedir.to_string()) {
            current_dir = current_dir.replace(&homedir.to_string(), "~");
        }

        let p = format!(
            "{}@{} {} {}{}{} ",
            whoami::username().blue(),
            whoami::hostname().blue(),
            current_dir.cyan(),
            "•".blue(),
            "•".red(),
            "•".yellow()
        );
        rl.helper_mut().expect("No helper").prompt = p.to_string();
        let readline = rl.readline(&p);
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str());
                shell.run_line(line);
            }
            Err(ReadlineError::Interrupted) => {
                continue;
            }
            Err(ReadlineError::Eof) => {
                println!("exit");
                break;
            }
            Err(err) => {
                utils::zash_error(format!("Error: {:?}", err));
                break;
            }
        }
        rl.save_history(&hispath).unwrap();
    }
}
