#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum LineToCmdTokens {
    Command,
    Separator,
}

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum CmdTokens {
    Command,
    Arg,
    Pipe,
    Definition,
    Variable,
}
