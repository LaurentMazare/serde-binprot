//! Deserialize bin_prot data to a Rust data structure.

use crate::error::{Error, Result};
use crate::{CODE_INT16, CODE_INT32, CODE_INT64, CODE_NEG_INT8};
use byteorder::{LittleEndian, ReadBytesExt};
use serde::de::{self, Visitor};
use std::convert::TryInto;
use std::io;

pub struct Deserializer<R> {
    read: R,
}

impl<R> Deserializer<R>
where
    R: io::Read,
{
    pub fn new(read: R) -> Self {
        Deserializer { read }
    }
}

impl<R> Deserializer<R>
where
    R: io::Read,
{
    fn read_signed(&mut self) -> Result<i64> {
        let c = self.read.read_u8()?;
        let v = match c {
            CODE_NEG_INT8 => self.read.read_i8()? as i64,
            CODE_INT16 => self.read.read_i16::<LittleEndian>()? as i64,
            CODE_INT32 => self.read.read_i32::<LittleEndian>()? as i64,
            CODE_INT64 => self.read.read_i64::<LittleEndian>()?,
            c => c as i64,
        };
        Ok(v)
    }

    fn read_nat0(&mut self) -> Result<u64> {
        let c = self.read.read_u8()?;
        let v = match c {
            CODE_INT16 => self.read.read_u16::<LittleEndian>()? as u64,
            CODE_INT32 => self.read.read_u32::<LittleEndian>()? as u64,
            CODE_INT64 => self.read.read_u64::<LittleEndian>()?,
            c => c as u64,
        };
        Ok(v)
    }

    fn read_float(&mut self) -> Result<f64> {
        let f = self.read.read_f64::<LittleEndian>()?;
        Ok(f)
    }
}

impl<'de, 'a, R> de::Deserializer<'de> for &'a mut Deserializer<R>
where
    R: io::Read,
{
    type Error = Error;

    fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        // The bin_prot format is not self describing so return an error
        // here.
        Err(Error::CannotDeserializeAny)
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let c = self.read.read_u8()?;
        let c = match c {
            0 => false,
            1 => true,
            _ => return Err(Error::ExpectedBoolean),
        };
        visitor.visit_bool(c)
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i8(self.read_signed()?.try_into()?)
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i16(self.read_signed()?.try_into()?)
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i32(self.read_signed()?.try_into()?)
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i64(self.read_signed()?)
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u8(self.read_nat0()?.try_into()?)
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u16(self.read_nat0()?.try_into()?)
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u32(self.read_nat0()?.try_into()?)
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u64(self.read_nat0()?)
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_f32(self.read_float()? as f32)
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_f64(self.read_float()?)
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let c = self.read.read_u8()?;
        visitor.visit_char(c as char)
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_string(visitor)
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let len = self.read_nat0()?;
        let mut vec = vec![0u8; len as usize];
        self.read.read_exact(&mut vec)?;
        let string = String::from_utf8(vec)?;
        visitor.visit_string(string)
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_byte_buf(visitor)
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let len = self.read_nat0()?;
        let mut vec = vec![0u8; len as usize];
        self.read.read_exact(&mut vec)?;
        visitor.visit_byte_buf(vec)
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let c = self.read.read_u8()?;
        let is_some = match c {
            0 => false,
            1 => true,
            _ => return Err(Error::ExpectedOption),
        };
        if is_some {
            visitor.visit_some(self)
        } else {
            visitor.visit_none()
        }
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let c = self.read.read_u8()?;
        if c == 0 {
            visitor.visit_unit()
        } else {
            Err(Error::ExpectedNull)
        }
    }

    fn deserialize_unit_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_unit(visitor)
    }

    fn deserialize_newtype_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let len = self.read_nat0()?;
        visitor.visit_seq(SeqWithLen::new(self, len as usize))
    }

    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_seq(SeqWithLen::new(self, len))
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        len: usize,
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_seq(SeqWithLen::new(self, len))
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let len = self.read_nat0()?;
        visitor.visit_map(SeqWithLen::new(self, len as usize))
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_seq(SeqWithLen::new(self, fields.len()))
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_enum(VariantAccess::new(self))
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        // This handles enum/variant identifiers.
        let variant_index = self.read.read_u8()?;
        visitor.visit_u32(variant_index as u32)
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }
}

struct SeqWithLen<'a, R: 'a> {
    de: &'a mut Deserializer<R>,
    len: usize,
}

impl<'a, R: 'a> SeqWithLen<'a, R> {
    fn new(de: &'a mut Deserializer<R>, len: usize) -> Self {
        SeqWithLen { de, len }
    }
}

impl<'de, 'a, R: io::Read + 'a> de::SeqAccess<'de> for SeqWithLen<'a, R> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: de::DeserializeSeed<'de>,
    {
        if self.len == 0 {
            return Ok(None);
        }
        self.len -= 1;
        seed.deserialize(&mut *self.de).map(Some)
    }
}

impl<'de, 'a, R: io::Read + 'a> de::MapAccess<'de> for SeqWithLen<'a, R> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: de::DeserializeSeed<'de>,
    {
        if self.len == 0 {
            return Ok(None);
        }
        self.len -= 1;
        seed.deserialize(&mut *self.de).map(Some)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: de::DeserializeSeed<'de>,
    {
        seed.deserialize(&mut *self.de)
    }
}

struct VariantAccess<'a, R: 'a> {
    de: &'a mut Deserializer<R>,
}

impl<'a, R: 'a> VariantAccess<'a, R> {
    fn new(de: &'a mut Deserializer<R>) -> Self {
        VariantAccess { de }
    }
}

impl<'de, 'a, R: io::Read + 'a> de::EnumAccess<'de> for VariantAccess<'a, R> {
    type Error = Error;
    type Variant = Self;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self)>
    where
        V: de::DeserializeSeed<'de>,
    {
        let val = seed.deserialize(&mut *self.de)?;
        Ok((val, self))
    }
}

impl<'de, 'a, R: io::Read + 'a> de::VariantAccess<'de> for VariantAccess<'a, R> {
    type Error = Error;

    fn unit_variant(self) -> Result<()> {
        Ok(())
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value>
    where
        T: de::DeserializeSeed<'de>,
    {
        seed.deserialize(self.de)
    }

    fn tuple_variant<V>(self, len: usize, visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_seq(SeqWithLen::new(self.de, len))
    }

    fn struct_variant<V>(self, fields: &'static [&'static str], visitor: V) -> Result<V::Value>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_seq(SeqWithLen::new(self.de, fields.len()))
    }
}

pub fn from_reader<'a, R, T>(rdr: R) -> Result<T>
where
    R: io::Read,
    T: de::Deserialize<'a>,
{
    let mut de = Deserializer::new(rdr);
    let value = de::Deserialize::deserialize(&mut de)?;
    match de.read.read_u8() {
        Ok(_) => Err(Error::TrailingCharacters),
        Err(err) => match err.kind() {
            io::ErrorKind::UnexpectedEof => Ok(value),
            _ => Err(err.into()),
        },
    }
}

pub fn from_slice<'a, T>(v: &'a [u8]) -> Result<T>
where
    T: de::Deserialize<'a>,
{
    from_reader(v)
}

pub fn from_str<'a, T>(s: &'a str) -> Result<T>
where
    T: de::Deserialize<'a>,
{
    from_reader(s.as_bytes())
}
