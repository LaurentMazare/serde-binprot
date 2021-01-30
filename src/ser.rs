// The spec can be found on https://github.com/janestreet/bin_prot
use crate::error::{Error, Result};
use crate::{CODE_INT16, CODE_INT32, CODE_INT64, CODE_NEG_INT8};
use serde::ser::{self, Serialize};
use std::io;

pub struct Serializer<W>
where
    W: io::Write,
{
    writer: W,
}

impl<W> Serializer<W>
where
    W: io::Write,
{
    pub fn new(writer: W) -> Self {
        Serializer { writer }
    }

    fn serialize_nat0(&mut self, v: u64) -> Result<()> {
        if v < 0x000000080 {
            self.writer.write_all(&[v as u8])?;
        } else if v < 0x000010000 {
            self.writer.write_all(&[CODE_INT16])?;
            self.writer.write_all(&(v as u16).to_le_bytes())?;
        } else if v < 0x100000000 {
            self.writer.write_all(&[CODE_INT32])?;
            self.writer.write_all(&(v as u32).to_le_bytes())?;
        } else {
            self.writer.write_all(&[CODE_INT64])?;
            self.writer.write_all(&v.to_le_bytes())?;
        }
        Ok(())
    }

    fn serialize_as_u8(&mut self, v: u32) -> Result<()> {
        if v < 256 {
            self.writer.write_all(&[v as u8])?;
            Ok(())
        } else {
            Err(Error::ExpectedU8)
        }
    }
}

impl<'a, W> ser::Serializer for &'a mut Serializer<W>
where
    W: io::Write,
{
    type Ok = ();

    type Error = Error;

    type SerializeMap = Self;
    type SerializeSeq = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;

    fn serialize_bool(self, b: bool) -> Result<()> {
        let b = if b { 1 } else { 0 };
        self.writer.write_all(&[b])?;
        Ok(())
    }

    fn serialize_i8(self, v: i8) -> Result<()> {
        self.serialize_i64(i64::from(v))
    }

    fn serialize_i16(self, v: i16) -> Result<()> {
        self.serialize_i64(i64::from(v))
    }

    fn serialize_i32(self, v: i32) -> Result<()> {
        self.serialize_i64(i64::from(v))
    }

    fn serialize_i64(self, v: i64) -> Result<()> {
        if 0 <= v {
            if v < 0x000000080 {
                self.writer.write_all(&[v as u8])?;
            } else if v < 0x00008000 {
                self.writer.write_all(&[CODE_INT16])?;
                self.writer.write_all(&(v as u16).to_le_bytes())?;
            } else if v < 0x80000000 {
                self.writer.write_all(&[CODE_INT32])?;
                self.writer.write_all(&(v as u32).to_le_bytes())?;
            } else {
                self.writer.write_all(&[CODE_INT64])?;
                self.writer.write_all(&v.to_le_bytes())?;
            }
        } else if v >= -0x00000080 {
            self.writer.write_all(&[CODE_NEG_INT8])?;
            self.writer.write_all(&v.to_le_bytes()[..1])?;
        } else if v >= -0x00008000 {
            self.writer.write_all(&[CODE_INT16])?;
            self.writer.write_all(&v.to_le_bytes()[..2])?;
        } else if v >= -0x80000000 {
            self.writer.write_all(&[CODE_INT32])?;
            self.writer.write_all(&v.to_le_bytes()[..4])?;
        } else if v < -0x80000000 {
            self.writer.write_all(&[CODE_INT64])?;
            self.writer.write_all(&v.to_le_bytes())?;
        }
        Ok(())
    }

    // For unsigned int, we use the Nat0.t representation.
    // This is *not* compatible with an ocaml int/i32/...
    fn serialize_u8(self, v: u8) -> Result<()> {
        self.serialize_nat0(u64::from(v))
    }

    fn serialize_u16(self, v: u16) -> Result<()> {
        self.serialize_nat0(u64::from(v))
    }

    fn serialize_u32(self, v: u32) -> Result<()> {
        self.serialize_nat0(u64::from(v))
    }

    fn serialize_u64(self, v: u64) -> Result<()> {
        self.serialize_nat0(v)
    }

    fn serialize_f32(self, v: f32) -> Result<()> {
        self.serialize_f64(f64::from(v))
    }

    fn serialize_f64(self, v: f64) -> Result<()> {
        self.writer.write_all(&v.to_le_bytes())?;
        Ok(())
    }

    fn serialize_char(self, v: char) -> Result<()> {
        self.serialize_as_u8(v as u32)
    }

    fn serialize_str(self, v: &str) -> Result<()> {
        self.serialize_nat0(v.len() as u64)?;
        self.writer.write_all(&v.as_bytes())?;
        Ok(())
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<()> {
        self.serialize_nat0(v.len() as u64)?;
        self.writer.write_all(&v)?;
        Ok(())
    }

    fn serialize_none(self) -> Result<()> {
        self.writer.write_all(&[0])?;
        Ok(())
    }

    fn serialize_some<T>(self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.writer.write_all(&[1])?;
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<()> {
        self.writer.write_all(&[0])?;
        Ok(())
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<()> {
        self.serialize_unit()
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        variant_index: u32,
        _variant: &'static str,
    ) -> Result<()> {
        // Use a single byte for the encoding as there is no way to know
        // how many variants are available. This is only correct for types with
        // less than 256 variants.
        // https://github.com/serde-rs/serde/issues/663
        self.serialize_as_u8(variant_index as u32)
    }

    fn serialize_newtype_struct<T>(self, _name: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        variant_index: u32,
        _variant: &'static str,
        value: &T,
    ) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.serialize_as_u8(variant_index as u32)?;
        value.serialize(&mut *self)
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq> {
        match len {
            Some(len) => {
                self.serialize_nat0(len as u64)?;
                Ok(self)
            }
            None => Err(Error::UnknownSeqLength),
        }
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple> {
        Ok(self)
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        Ok(self)
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        self.serialize_as_u8(variant_index as u32)?;
        Ok(self)
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap> {
        match len {
            Some(len) => {
                self.serialize_nat0(len as u64)?;
                Ok(self)
            }
            None => Err(Error::UnknownSeqLength),
        }
    }

    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeStruct> {
        Ok(self)
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        self.serialize_as_u8(variant_index as u32)?;
        Ok(self)
    }
}

impl<'a, W> ser::SerializeSeq for &'a mut Serializer<W>
where
    W: io::Write,
{
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a, W> ser::SerializeTuple for &'a mut Serializer<W>
where
    W: io::Write,
{
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a, W> ser::SerializeTupleStruct for &'a mut Serializer<W>
where
    W: io::Write,
{
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a, W> ser::SerializeTupleVariant for &'a mut Serializer<W>
where
    W: io::Write,
{
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a, W> ser::SerializeMap for &'a mut Serializer<W>
where
    W: io::Write,
{
    type Ok = ();
    type Error = Error;

    fn serialize_key<T>(&mut self, key: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        key.serialize(&mut **self)
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a, W> ser::SerializeStruct for &'a mut Serializer<W>
where
    W: io::Write,
{
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, _key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a, W> ser::SerializeStructVariant for &'a mut Serializer<W>
where
    W: io::Write,
{
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, _key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        Ok(())
    }
}

pub fn to_writer<W, T: ?Sized>(writer: W, value: &T) -> Result<()>
where
    W: io::Write,
    T: Serialize,
{
    let mut ser = Serializer::new(writer);
    value.serialize(&mut ser)?;
    Ok(())
}

pub fn to_vec<T: ?Sized>(value: &T) -> Result<Vec<u8>>
where
    T: Serialize,
{
    let mut writer = Vec::with_capacity(128);
    to_writer(&mut writer, value)?;
    Ok(writer)
}

#[cfg(test)]
mod tests {
    use super::to_vec;
    use serde_derive::Serialize;

    #[test]
    fn test_all() {
        #[derive(Serialize, Clone)]
        struct Bar {
            head: (&'static str, i32),
            tail: Option<Box<Bar>>,
        }

        #[derive(Serialize)]
        struct Foo {
            foo_i32: i32,
            foo_i64: i64,
            foo_bool: bool,
            foo_tuple: (bool, i32, f64, f32),
            str: String,
            seq_f64: Vec<f64>,
            seq_bar: Vec<Bar>,
        }

        let bar1 = Bar {
            head: ("b1", 1234),
            tail: None,
        };
        let bar2 = Bar {
            head: ("b2", 5678),
            tail: Some(Box::new(bar1.clone())),
        };
        let foo = Foo {
            foo_i32: -42,
            foo_i64: 1337133713371337,
            foo_bool: false,
            foo_tuple: (true, 42, 3.14159, 2.71828),
            str: "foobar".to_owned(),
            seq_f64: vec![3.14, 2.718, 1.337],
            seq_bar: vec![bar1, bar2],
        };
        let expected = [
            255, 214, 252, 201, 176, 0, 180, 29, 192, 4, 0, 0, 1, 42, 110, 134, 27, 240, 249, 33,
            9, 64, 0, 0, 0, 160, 9, 191, 5, 64, 6, 102, 111, 111, 98, 97, 114, 3, 31, 133, 235, 81,
            184, 30, 9, 64, 88, 57, 180, 200, 118, 190, 5, 64, 49, 8, 172, 28, 90, 100, 245, 63, 2,
            2, 98, 49, 254, 210, 4, 0, 2, 98, 50, 254, 46, 22, 1, 2, 98, 49, 254, 210, 4, 0,
        ];
        assert_eq!(to_vec(&foo).unwrap(), expected);
    }
}
