//! Implements a serde serializer for JavaScript values
//!
//! # Safety
//!
//! All JavaScript types are neither `Send` or `Sync`. Threads should be used.

use conv::ValueFrom;
use serde_crate::{ser, Serialize};

use super::*;
use crate::napi;

#[derive(Clone, Copy)]
#[repr(transparent)]
/// High level deserializer for all JavaScript values
pub(super) struct Serializer {
    env: napi::Env,
}

impl Serializer {
    pub(super) fn new(env: napi::Env) -> Self {
        Self { env }
    }
}

// Specialized serializer for writing to an `Array`
pub(super) struct ArraySerializer {
    serializer: Serializer,
    value: napi::Value,
    offset: usize,
}

impl ArraySerializer {
    fn new(serializer: Serializer, value: napi::Value) -> Self {
        Self {
            serializer,
            value,
            offset: 0,
        }
    }
}

// `Array` serializer for externally tagged enum `{ [key]: value }`
pub(super) struct WrappedArraySerializer {
    serializer: ArraySerializer,
    value: napi::Value,
}

impl WrappedArraySerializer {
    fn new(serializer: ArraySerializer, value: napi::Value) -> Self {
        Self { serializer, value }
    }
}

// Specialized serializer for writing to a generic `Object`
pub(super) struct ObjectSerializer {
    serializer: Serializer,
    value: napi::Value,
    key: Option<napi::Value>,
}

impl ObjectSerializer {
    fn new(serializer: Serializer, value: napi::Value) -> Self {
        Self {
            serializer,
            value,
            key: None,
        }
    }
}

// `Object` serializer for externally tagged enum `{ [key]: value }`
pub(super) struct WrappedObjectSerializer {
    serializer: ObjectSerializer,
    value: napi::Value,
}

impl WrappedObjectSerializer {
    fn new(serializer: ObjectSerializer, value: napi::Value) -> Self {
        Self { serializer, value }
    }
}

impl ser::Serializer for Serializer {
    type Ok = napi::Value;
    type Error = Error;

    // Limited JavaScript types require sequences and tuples to both use `Array`
    type SerializeSeq = ArraySerializer;
    type SerializeTuple = ArraySerializer;
    type SerializeTupleStruct = ArraySerializer;
    type SerializeTupleVariant = WrappedArraySerializer;
    type SerializeMap = ObjectSerializer;
    type SerializeStruct = ObjectSerializer;
    type SerializeStructVariant = WrappedObjectSerializer;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        Ok(create_bool(self.env, v)?)
    }

    // All numeric types are serialized into `f64`
    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        Ok(create_double(self.env, v)?)
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        Ok(create_double(self.env, v)?)
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        Ok(create_double(self.env, v)?)
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        let v = f64::value_from(v)?;
        Ok(create_double(self.env, v)?)
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        Ok(create_double(self.env, v)?)
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        Ok(create_double(self.env, v)?)
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        Ok(create_double(self.env, v)?)
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        let v = f64::value_from(v)?;
        Ok(create_double(self.env, v)?)
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        Ok(create_double(self.env, v)?)
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        Ok(create_double(self.env, v)?)
    }

    // `char` are serialized as single character string
    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        Ok(create_string(self.env, v.to_string())?)
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        Ok(create_string(self.env, v)?)
    }

    // Bytes are serialized as `ArrayBuffer`
    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        Ok(create_arraybuffer(self.env, v)?)
    }

    // `None` is serialized as a `null`
    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        self.serialize_unit()
    }

    fn serialize_some<T>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    // JavaScript does not have a unit type; `null` is used instead
    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Ok(get_null(self.env)?)
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        self.serialize_unit()
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        self.serialize_str(variant)
    }

    fn serialize_newtype_struct<T>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        let o = create_object(self.env)?;
        let k = create_string(self.env, variant)?;
        let v = value.serialize(self)?;

        object_set(self.env, o, k, v)?;

        Ok(o)
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        let len = len.unwrap_or_default();
        let value = create_array_with_length(self.env, len)?;

        Ok(ArraySerializer::new(self, value))
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        self.serialize_seq(Some(len))
    }

    // Externally tagged enum; `{ [key]: value }`
    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        let env = self.env;
        let wrapper = create_object(env)?;
        let arr = create_array_with_length(env, len)?;
        let k = create_string(env, variant)?;
        let serializer = ArraySerializer::new(self, arr);

        object_set(env, wrapper, k, arr)?;

        Ok(WrappedArraySerializer::new(serializer, wrapper))
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        let value = create_object(self.env)?;

        Ok(ObjectSerializer::new(self, value))
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        self.serialize_map(Some(len))
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        let env = self.env;
        let wrapper = create_object(env)?;
        let value = create_object(env)?;
        let k = create_string(env, variant)?;
        let serializer = ObjectSerializer::new(self, value);

        object_set(env, wrapper, k, value)?;

        Ok(WrappedObjectSerializer::new(serializer, wrapper))
    }
}

impl<'a> ser::SerializeSeq for ArraySerializer {
    type Ok = napi::Value;
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        let value = value.serialize(self.serializer)?;
        let k = u32::value_from(self.offset)?;

        array_set(self.serializer.env, self.value, k, value)?;
        self.offset += 1;

        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(self.value)
    }
}

impl<'a> ser::SerializeTuple for ArraySerializer {
    type Ok = napi::Value;
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        ser::SerializeSeq::end(self)
    }
}

impl<'a> ser::SerializeTupleStruct for ArraySerializer {
    type Ok = napi::Value;
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        ser::SerializeSeq::end(self)
    }
}

impl<'a> ser::SerializeTupleVariant for WrappedArraySerializer {
    type Ok = napi::Value;
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        ser::SerializeSeq::serialize_element(&mut self.serializer, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(self.value)
    }
}

impl ser::SerializeMap for ObjectSerializer {
    type Ok = napi::Value;
    type Error = Error;

    fn serialize_key<T>(&mut self, key: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        self.key = Some(key.serialize(self.serializer)?);

        Ok(())
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        let k = self.key.ok_or_else(Error::missing_key)?;
        let v = value.serialize(self.serializer)?;

        object_set(self.serializer.env, self.value, k, v)?;

        Ok(())
    }

    fn serialize_entry<K, V>(&mut self, key: &K, value: &V) -> Result<(), Self::Error>
    where
        K: ?Sized + Serialize,
        V: ?Sized + Serialize,
    {
        let k = key.serialize(self.serializer)?;
        let v = value.serialize(self.serializer)?;

        object_set(self.serializer.env, self.value, k, v)?;

        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(self.value)
    }
}

impl ser::SerializeStruct for ObjectSerializer {
    type Ok = napi::Value;
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        ser::SerializeMap::serialize_entry(self, key, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(self.value)
    }
}

impl ser::SerializeStructVariant for WrappedObjectSerializer {
    type Ok = napi::Value;
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        ser::SerializeMap::serialize_entry(&mut self.serializer, key, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(self.value)
    }
}
