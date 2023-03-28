pub mod error;
mod ser;

use serde::Serialize;

use error::Result;

pub fn to_string<T>(value: &T) -> Result<String>
where
    T: Serialize,
{
    let mut serializer = ser::Serializer::default();
    {
        value.serialize(&mut serializer)?;
    }
    Ok(serializer.to_string())
}