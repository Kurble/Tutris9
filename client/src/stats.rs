use serde::ser::{Serialize, Serializer, SerializeStruct, Impossible};
use std::fmt::Display;

pub struct StatsPopulator<F: FnMut(String)> {
    value: String,
    factory: F,
}

impl<F: FnMut(String)> StatsPopulator<F> {
    pub fn populate<T: Serialize>(v: &T, f: F) -> Result<(), StatsError> {
        v.serialize(&mut Self {
            value: "".to_string(),
            factory: f,
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum StatsError {
    Message(String),
    Impossible,
}

impl serde::ser::Error for StatsError {
    fn custom<T: Display>(msg: T) -> Self {
        StatsError::Message(msg.to_string())
    }
}

impl Display for StatsError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str(std::error::Error::description(self))
    }
}

impl std::error::Error for StatsError {
    fn description(&self) -> &str {
        match *self {
            StatsError::Message(ref msg) => msg,
            StatsError::Impossible => "unsupported data",
        }
    }
}

impl<'a, F: FnMut(String)> SerializeStruct for &'a mut StatsPopulator<F> {
    type Ok = ();
    type Error = StatsError;

    fn serialize_field<T: ?Sized>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error> where T: Serialize {

        value.serialize(&mut**self).unwrap();

        let text = format!("{}: {}", key, self.value);
        self.value.clear();
        (self.factory)(text);

        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> { Ok(()) }
}

impl<'a, F: FnMut(String)> Serializer for &'a mut StatsPopulator<F> {
    type Ok = ();
    type Error = StatsError;

    type SerializeSeq = Impossible<Self::Ok, Self::Error>;
    type SerializeTuple = Impossible<Self::Ok, Self::Error>;
    type SerializeTupleStruct = Impossible<Self::Ok, Self::Error>;
    type SerializeTupleVariant = Impossible<Self::Ok, Self::Error>;
    type SerializeMap = Impossible<Self::Ok, Self::Error>;
    type SerializeStruct = Self;
    type SerializeStructVariant = Impossible<Self::Ok, Self::Error>;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        self.value = format!("{}{}", self.value, v);
        Ok(())
    }
    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        self.value = format!("{}{}", self.value, v);
        Ok(())
    }
    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        self.value = format!("{}{}", self.value, v);
        Ok(())
    }
    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        self.value = format!("{}{}", self.value, v);
        Ok(())
    }
    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        self.value = format!("{}{}", self.value, v);
        Ok(())
    }
    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        self.value = format!("{}{}", self.value, v);
        Ok(())
    }
    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        self.value = format!("{}{}", self.value, v);
        Ok(())
    }
    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        self.value = format!("{}{}", self.value, v);
        Ok(())
    }
    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        self.value = format!("{}{}", self.value, v);
        Ok(())
    }
    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        self.value = format!("{}{}", self.value, v);
        Ok(())
    }
    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        self.value = format!("{}{}", self.value, v);
        Ok(())
    }
    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        self.value = format!("{}{}", self.value, v);
        Ok(())
    }
    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        self.value = format!("{}{}", self.value, v);
        Ok(())
    }
    fn serialize_bytes(self, _: &[u8]) -> Result<Self::Ok, Self::Error> { Ok(()) }
    fn serialize_none(self) -> Result<Self::Ok, Self::Error> { Ok(()) }
    fn serialize_some<T: ?Sized>(self, _: &T) -> Result<Self::Ok, Self::Error> where T: Serialize { Ok(()) }
    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> { Ok(()) }
    fn serialize_unit_struct(self, _: &'static str) -> Result<Self::Ok, Self::Error> { Ok(()) }
    fn serialize_unit_variant(self, _: &'static str, _: u32, _: &'static str) -> Result<Self::Ok, Self::Error> { Ok (()) }
    fn serialize_newtype_struct<T: ?Sized>(self, _: &'static str, _: &T) -> Result<Self::Ok, Self::Error> where T: Serialize { Ok(()) }
    fn serialize_newtype_variant<T: ?Sized>(self, _: &'static str, _: u32, _: &'static str, _: &T) -> Result<Self::Ok, Self::Error> where T: Serialize { Ok(()) }
    fn serialize_seq(self, _: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> { Err(StatsError::Impossible) }
    fn serialize_tuple(self, _: usize) -> Result<Self::SerializeTuple, Self::Error> { Err(StatsError::Impossible) }
    fn serialize_tuple_struct(self, _: &'static str, _: usize) -> Result<Self::SerializeTupleStruct, Self::Error> { Err(StatsError::Impossible) }
    fn serialize_tuple_variant(self, _: &'static str, _: u32, _: &'static str, _: usize) -> Result<Self::SerializeTupleVariant, Self::Error> { Err(StatsError::Impossible) }
    fn serialize_map(self, _: Option<usize>) -> Result<Self::SerializeMap, Self::Error> { Err(StatsError::Impossible) }
    fn serialize_struct(self, _: &'static str, _: usize) -> Result<Self::SerializeStruct, Self::Error> { Ok(self) }
    fn serialize_struct_variant(self, _: &'static str, _: u32, _: &'static str, _: usize) -> Result<Self::SerializeStructVariant, Self::Error> { Err(StatsError::Impossible) }
    #[cfg(not(any(feature = "std", feature = "alloc")))]
    fn collect_str<T: ?Sized>(self, _: &T) -> Result<Self::Ok, Self::Error> where T: Display { Ok(()) }
}
