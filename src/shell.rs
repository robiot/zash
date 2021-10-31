use colored::Colorize;
use rustyline::completion::{Completer, FilenameCompleter, Pair};
use rustyline::config::OutputStreamType;
use rustyline::error::ReadlineError;
use rustyline::highlight::{Highlighter, MatchingBracketHighlighter};
use rustyline::hint::{Hinter, HistoryHinter};
use rustyline::validate::{MatchingBracketValidator, Validator};
use rustyline::{CompletionType, Config, Context, EditMode, Editor};
use rustyline_derive::Helper;
use std::borrow::Cow::{self, Borrowed, Owned};
use std::process::{Child, Command, Stdio};
use whoami;

use crate::builtins;
use crate::parser;
use crate::utils;

const RUN_LINE_SUCCESS: i16 = 0;
const RUN_LINE_CONTINUE: i16 = 1;
const RUN_LINE_BREAK: i16 = 2;
pub fn run_line(line: String) -> i16 {
    let mut commands = line.trim().split("|").peekable();
    let mut prev_command = None;

    while let Some(command) = commands.next() {
        let mut args = parser::Parser::new(command.trim());
        let command = match args.next() {
            Some(n) => n,
            None => return RUN_LINE_CONTINUE,
        };
        match command.as_ref() {
            // Builtins
            "cd" => builtins::cd::cd(args),

            "exit" => return RUN_LINE_BREAK,

            command => {
                let stdin = prev_command.map_or(Stdio::inherit(), |output: Child| {
                    Stdio::from(output.stdout.unwrap())
                });

                let stdout = if commands.peek().is_some() {
                    Stdio::piped()
                } else {
                    Stdio::inherit()
                };

                match Command::new(command)
                    .args(args)
                    .stdin(stdin)
                    .stdout(stdout)
                    .spawn()
                {
                    Ok(output) => {
                        prev_command = Some(output);
                    }
                    Err(_) => {
                        prev_command = None;
                        utils::zash_error(format!("command not found: {}", command));
                    }
                };
            }
        }
    }

    if let Some(mut final_command) = prev_command {
        final_command.wait().unwrap();
    }

    return RUN_LINE_SUCCESS;
}

#[derive(Helper)]
struct ShellHelper {
    completer: FilenameCompleter,
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
        Owned(format!("{}", hint.dimmed()).to_owned())
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
        .edit_mode(EditMode::Emacs)
        .output_stream(OutputStreamType::Stdout)
        .build();

    let helper = ShellHelper {
        completer: FilenameCompleter::new(),
        highlighter: MatchingBracketHighlighter::new(),
        hinter: HistoryHinter {},
        prompt: "".to_owned(),
        validator: MatchingBracketValidator::new(),
    };

    let mut rl = Editor::with_config(config);
    rl.set_helper(Some(helper));

    utils::load_zashrc(homedir.clone());
    let hispath = format!("{}/.zash_history", homedir);
    if rl.load_history(&hispath).is_err() {
        utils::zash_error("No previous history");
    }

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
                match run_line(line) {
                    RUN_LINE_SUCCESS => {}
                    RUN_LINE_CONTINUE => continue,
                    RUN_LINE_BREAK => break,
                    _ => break,
                }
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
