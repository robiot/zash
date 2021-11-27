use colored::Colorize;
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

use crate::builtins;
use crate::parser;
use crate::utils;

pub fn run_line(line: String) {
    let mut status = 0;
    let mut sep = String::new();
    for token in parser::line_to_cmds(line.trim()) {
        if token.0 == parser::LineTCmdTokens::Separator {
            sep = token.1.clone();
            continue;
        }

        // "&& "Don't run the other commands if the one before failed
        //
        // "||" = "Or"
        // "ls || dir"
        // If ls does not succed it runs "dir"
        // If ls succed it does not run dir
        if (sep == "&&" && status != 0) || (sep == "||" && status == 0) {
            break;
        }

        status = exec_command(token);
    }
}

fn exec_command(token: (parser::LineTCmdTokens, std::string::String)) -> i32 {
    let mut pipe_commands = parser::Parser::new(token.1.trim(), "|".to_string(), true).peekable();
    let mut prev_command = None;
    while let Some(pipe_command) = pipe_commands.next() {
        let mut args = parser::Parser::new(pipe_command.trim(), " ".to_string(), false);
        let command = match args.next() {
            Some(n) => n,
            None => return 1,
        };
        match command.as_ref() {
            // Builtins
            "cd" => builtins::cd::cd(args),
            "exit" => builtins::exit::exit(args),
            command => {
                let stdin = prev_command.map_or(Stdio::inherit(), |output: Child| {
                    Stdio::from(output.stdout.unwrap())
                });
                let stdout = if pipe_commands.peek().is_some() {
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
                        // prev_command = None;
                        utils::zash_error(format!("command not found: {}", command));
                        return 1;
                    }
                };
            }
        }
    }
    if let Some(mut final_command) = prev_command {
        final_command.wait().unwrap();
    }
    0
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
                run_line(line);
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
