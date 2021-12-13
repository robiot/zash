pub type Result<T> = std::result::Result<T, SyntaxError>;

#[derive(Debug, Clone)]
pub struct SyntaxError;

impl std::fmt::Display for SyntaxError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "SyntaxError: Unexpected end of input")
    }
}