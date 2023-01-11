#[derive(Default, Clone, Debug)]
pub struct ModName {
    pub author: String,
    pub name: String,
    pub version: Option<String>,
}
