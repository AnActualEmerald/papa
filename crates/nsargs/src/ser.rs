use serde::ser;

use crate::error::{self, Error};

#[derive(Default, Debug)]
pub struct Serializer {
    output: String,
    last: String,
    in_struct: bool,
}

impl<'a> Serializer {
    pub fn to_string(&self) -> String {
        if self.output.len() > 0 {
            self.output.clone()
        } else {
            self.last.to_owned()
        }
        .trim()
        .to_owned()
    }
}

impl<'a> serde::ser::Serializer for &'a mut Serializer {
    type Ok = bool;

    type Error = error::Error;

    type SerializeSeq = Self;

    type SerializeTuple = Self;

    type SerializeTupleStruct = Self;

    type SerializeTupleVariant = Self;

    type SerializeMap = Self;

    type SerializeStruct = Self;

    type SerializeStructVariant = Self;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        self.last = if v { "true" } else { "false" }.into();
        Ok(!v)
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        self.serialize_i64(v.into())
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        self.serialize_i64(v.into())
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        self.serialize_i64(v.into())
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        self.last = v.to_string();
        Ok(true)
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        self.serialize_u64(v.into())
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        self.serialize_u64(v.into())
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        self.serialize_u64(v.into())
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        self.last = v.to_string();
        Ok(true)
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        self.serialize_f64(v.into())
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        self.last = v.to_string();
        Ok(true)
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        self.serialize_str(&String::from(v))
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        self.last = format!(r#""{}""#, v);
        Ok(true)
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        self.serialize_str(&match String::from_utf8(v.to_vec()) {
            Ok(it) => it,
            Err(err) => return Err(Self::Error::from(err)),
        })
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        self.last = String::new();
        Ok(false)
    }

    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: serde::Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Ok(false)
    }

    fn serialize_unit_struct(self, name: &'static str) -> Result<Self::Ok, Self::Error> {
        self.output += &format!("-{} ", name);
        Ok(false)
    }

    fn serialize_unit_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        self.last += &format!("{} ", variant);
        Ok(true)
    }

    fn serialize_newtype_struct<T: ?Sized>(
        self,
        name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: serde::Serialize,
    {
        self.output += &format!("+{} ", name);
        value.serialize(self)?;
        Ok(false)
    }

    fn serialize_newtype_variant<T: ?Sized>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: serde::Serialize,
    {
        let ret = self.in_struct;
        if !self.in_struct {
            self.output += &format!("{} ", variant);
        }
        value.serialize(self)?;
        Ok(ret)
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        self.output += "\"";
        Ok(self)
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        self.output += "\"";
        Ok(self)
    }

    fn serialize_tuple_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Ok(self)
    }

    fn serialize_tuple_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        self.output += &format!("{} \"", name);
        Ok(self)
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        self.output += &format!("\"");
        Ok(self)
    }

    fn serialize_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        self.in_struct = true;
        Ok(self)
    }

    fn serialize_struct_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Ok(self)
    }
}

impl<'a> ser::SerializeSeq for &'a mut Serializer {
    type Ok = bool;

    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: serde::Serialize,
    {
        value.serialize(&mut **self)?;
        self.output += self.last.trim_matches('"');
        self.output += " ";
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.output = self.output.trim_end().into();
        self.output += "\"";
        Ok(true)
    }
}

impl<'a> ser::SerializeTuple for &'a mut Serializer {
    type Ok = bool;

    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: serde::Serialize,
    {
        value.serialize(&mut **self)?;
        self.output += self.last.trim_matches('"').into();
        Ok(self.output += " ")
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.output = self.output.trim_end().into();
        self.output += "\"";
        Ok(true)
    }
}

impl<'a> ser::SerializeTupleStruct for &'a mut Serializer {
    type Ok = bool;

    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: serde::Serialize,
    {
        value.serialize(&mut **self)?;
        Ok(self.output += " ")
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(true)
    }
}

impl<'a> ser::SerializeTupleVariant for &'a mut Serializer {
    type Ok = bool;

    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: serde::Serialize,
    {
        value.serialize(&mut **self)?;
        Ok(self.output += " ")
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.output += "\"";
        Ok(true)
    }
}

impl<'a> ser::SerializeMap for &'a mut Serializer {
    type Ok = bool;

    type Error = Error;

    fn serialize_key<T: ?Sized>(&mut self, key: &T) -> Result<(), Self::Error>
    where
        T: serde::Serialize,
    {
        key.serialize(&mut **self)?;
        self.output += self.last.trim_matches('"').into();
        Ok(self.output += " ")
    }

    fn serialize_value<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: serde::Serialize,
    {
        value.serialize(&mut **self)?;
        self.output += self.last.trim_matches('"').into();
        Ok(self.output += " ")
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.output += "\"";
        Ok(true)
    }
}

impl<'a> ser::SerializeStruct for &'a mut Serializer {
    type Ok = bool;

    type Error = Error;

    fn serialize_field<T: ?Sized>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error>
    where
        T: serde::Serialize,
    {
        let had_value = value.serialize(&mut **self)?;
        self.output += &format!(
            "{}{} ",
            if key == "port" || !had_value {
                "-"
            } else {
                "+"
            },
            key,
        );
        if had_value {
            self.output += &format!("{} ", self.last);
        }
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.in_struct = false;
        Ok(false)
    }
}

impl<'a> ser::SerializeStructVariant for &'a mut Serializer {
    type Ok = bool;

    type Error = Error;

    fn serialize_field<T: ?Sized>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error>
    where
        T: serde::Serialize,
    {
        let had_value = value.serialize(&mut **self)?;
        self.output += &format!(
            "{}{}",
            if key == "port" || !had_value {
                "-"
            } else {
                "+"
            },
            key,
        );
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(false)
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use serde::Serialize;

    #[test]
    fn serialize_default_struct() {
        #[derive(Serialize)]
        struct Simple {
            multiple: bool,
        }

        let input = Simple { multiple: true };

        let res = crate::to_string(&input);
        assert!(res.is_ok());
        assert_eq!(String::from("-multiple"), res.unwrap());
    }

    #[test]
    fn serialize_vec() {
        let input = vec!["this", "shouldn't", "really", "happen"];
        let res = crate::to_string(&input);

        assert!(res.is_ok());
        assert_eq!(r#""this shouldn't really happen""#, res.unwrap());
    }

    #[test]
    fn serialize_tuple() {
        let input = ("please", "don't", "use", "tuples");
        let res = crate::to_string(&input);

        assert!(res.is_ok());
        assert_eq!(r#""please don't use tuples""#, res.unwrap());
    }

    #[test]
    fn serialize_tuple_mix() {
        let input = ("hi", 42, 'c', -129, 8.0, None::<usize>);
        let res = crate::to_string(&input);

        assert!(res.is_ok());
        assert_eq!(r#""hi 42 c -129 8""#, res.unwrap());
    }

    #[test]
    fn serialize_unit_struct() {
        #[derive(Serialize)]
        struct Multiple;

        let input = Multiple;
        let res = crate::to_string(&input);

        assert!(res.is_ok());
        assert_eq!(r#"-Multiple"#, res.unwrap());
    }

    #[test]
    fn serialize_complex_struct() {
        #[derive(Serialize)]
        struct Complex {
            multiple: bool,
            exec: String,
            novid: bool,
            #[serde(rename = "playlistvaroverrides")]
            playlist_overrides: HashMap<String, String>,
        }

        let mut overrides = HashMap::new();
        overrides.insert("fsu_example".into(), "some_value".into());

        let input = Complex {
            multiple: true,
            playlist_overrides: overrides,
            exec: "gecko.cfg".into(),
            novid: true,
        };

        let res = crate::to_string(&input);

        assert!(res.is_ok());
        assert_eq!(
            "-multiple +exec \"gecko.cfg\" -novid +playlistvaroverrides \"fsu_example some_value\"",
            res.unwrap()
        );
    }
}
