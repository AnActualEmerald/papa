use crate::model::ModName;
use regex::Regex;

pub fn validate_modnames(input: &str) -> Result<ModName, String> {
    let Ok(re) = Regex::new(r"^(\w+)\.(\w+)(?:@(\d+\.\d+\.\d+))?$")else {
        return Err("Unable to compile regex".to_string());
    };

    if let Some(captures) = re.captures(input) {
        let mut name = ModName::default();
        if let Some(author) = captures.get(1) {
            name.author = author.as_str().to_string();
        }

        if let Some(n) = captures.get(2) {
            name.name = n.as_str().to_string();
        }

        name.version = captures.get(3).map(|v| v.as_str().to_string());

        Ok(name)
    } else {
        Err(format!(
            "Mod name '{input}' should be in 'Author.ModName' format"
        ))
    }
}

pub fn to_file_size_string(size: u64) -> String {
    if size / 1_000_000 >= 1 {
        let size = size as f64 / 1_048_576f64;

        format!("{:.2} MB", size)
    } else {
        let size = size as f64 / 1024f64;
        format!("{:.2} KB", size)
    }
}
