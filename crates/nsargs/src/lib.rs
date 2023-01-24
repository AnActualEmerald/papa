pub mod error;
mod ser;

use serde::Serialize;

use error::Result;

pub fn to_string<T>(value: &T) -> Result<String>
where
    T: Serialize,
{
    let mut serializer = ser::Serializer {
        output: String::new(),
    };
    value.serialize(serializer)?;
    Ok(serializer.into_string())
}
