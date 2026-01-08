#[derive(Debug, Clone)]
pub struct Args {
    pub flags: std::collections::HashMap<String, bool>,
    pub params: std::collections::HashMap<String, String>,
    pub commands: Vec<String>,
}