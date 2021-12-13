#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum LineToCmdTokens {
    Command,
    Separator,
}

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum CmdTokens {
    Normal,
    Pipe,
    Definition,
    Variable,
}

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum ParseCmdTokens {
    Command,
    Separator,
}