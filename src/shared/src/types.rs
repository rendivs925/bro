pub type Result<T> = anyhow::Result<T>;

#[derive(Debug, Clone, PartialEq)]
pub enum ScriptType {
    Bash,
    Python,
    JavaScript,
    Rust,
}
