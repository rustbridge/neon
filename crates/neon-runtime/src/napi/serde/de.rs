//! Implements a serde deserializer for JavaScript values
//!
//! # Safety
//!
//! All JavaScript types are neither `Send` or `Sync`. Threads should be used.

use conv::{ApproxFrom, DefaultApprox};
use serde_crate::de::{
    self, DeserializeSeed, EnumAccess, IntoDeserializer, MapAccess, SeqAccess, VariantAccess,
    Visitor,
};

use super::*;
use crate::napi;

/// High level deserializer for all JavaScript values
pub(super) struct Deserializer {
    env: napi::Env,
    value: napi::Value,
}

impl Deserializer {
    pub(super) fn new(env: napi::Env, value: napi::Value) -> Self {
        Deserializer { env, value }
    }
}

/// Specialized deserializer for `Array`
struct ArrayAccessor {
    env: napi::Env,
    array: napi::Value,
    len: u32,
    index: u32,
}

impl ArrayAccessor {
    fn new(env: napi::Env, array: napi::Value) -> Result<Self, Error> {
        Ok(Self {
            env,
            array,
            len: get_array_len(env, array)?,
            index: 0,
        })
    }

    fn next(&mut self) -> Result<Option<napi::Value>, Error> {
        if self.index >= self.len {
            return Ok(None);
        }

        let element = get_array_element(self.env, self.array, self.index)?;

        self.index += 1;

        Ok(Some(element))
    }
}

/// Specialized deserializer for generic `Object`
/// Only enumerable keys are read
struct ObjectAccessor {
    env: napi::Env,
    object: napi::Value,
    keys: ArrayAccessor,
    // Cache the most recent key for reading the next value
    next: Option<napi::Value>,
}

impl ObjectAccessor {
    fn new(env: napi::Env, object: napi::Value) -> Result<Self, Error> {
        let keys = get_property_names(env, object)?;
        let keys = ArrayAccessor::new(env, keys)?;

        Ok(Self {
            env,
            object,
            keys,
            next: None,
        })
    }
}

impl de::Deserializer<'static> for Deserializer {
    type Error = Error;

    // JavaScript is a self describing format, allowing us to provide a deserialization
    // implementation without prior knowledge of the schema. This is useful for types
    // like `serde_json::Value`.
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'static>,
    {
        match typeof_value(self.env, self.value)? {
            napi::ValueType::Undefined | napi::ValueType::Null => self.deserialize_unit(visitor),
            napi::ValueType::Boolean => self.deserialize_bool(visitor),
            napi::ValueType::Number => self.deserialize_f64(visitor),
            napi::ValueType::String => self.deserialize_string(visitor),
            napi::ValueType::Object => self.deserialize_map(visitor),
            typ => Err(Error::unsupported_type(typ)),
        }
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'static>,
    {
        visitor.visit_bool(get_value_bool(self.env, self.value)?)
    }

    // JavaScript only provides an `f64` number type. All integer types require
    // a lossy approximation. Out of range values will error.
    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'static>,
    {
        let n = get_value_double(self.env, self.value)?;
        let n = <i8 as ApproxFrom<_, DefaultApprox>>::approx_from(n)?;

        visitor.visit_i8(n)
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'static>,
    {
        let n = get_value_double(self.env, self.value)?;
        let n = <i16 as ApproxFrom<_, DefaultApprox>>::approx_from(n)?;

        visitor.visit_i16(n)
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'static>,
    {
        let n = get_value_double(self.env, self.value)?;
        let n = <i32 as ApproxFrom<_, DefaultApprox>>::approx_from(n)?;

        visitor.visit_i32(n)
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'static>,
    {
        let n = get_value_double(self.env, self.value)?;
        let n = <i64 as ApproxFrom<_, DefaultApprox>>::approx_from(n)?;

        visitor.visit_i64(n)
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'static>,
    {
        let n = get_value_double(self.env, self.value)?;
        let n = <u8 as ApproxFrom<_, DefaultApprox>>::approx_from(n)?;

        visitor.visit_u8(n)
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'static>,
    {
        let n = get_value_double(self.env, self.value)?;
        let n = <u16 as ApproxFrom<_, DefaultApprox>>::approx_from(n)?;

        visitor.visit_u16(n)
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'static>,
    {
        let n = get_value_double(self.env, self.value)?;
        let n = <u32 as ApproxFrom<_, DefaultApprox>>::approx_from(n)?;

        visitor.visit_u32(n)
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'static>,
    {
        let n = get_value_double(self.env, self.value)?;
        let n = <u64 as ApproxFrom<_, DefaultApprox>>::approx_from(n)?;

        visitor.visit_u64(n)
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'static>,
    {
        let n = get_value_double(self.env, self.value)?;
        let n = <f32 as ApproxFrom<_, DefaultApprox>>::approx_from(n)?;

        visitor.visit_f32(n)
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'static>,
    {
        let n = get_value_double(self.env, self.value)?;

        visitor.visit_f64(n)
    }

    // `char` are serialized as a single character `string`
    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'static>,
    {
        self.deserialize_string(visitor)
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'static>,
    {
        self.deserialize_string(visitor)
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'static>,
    {
        visitor.visit_string(get_value_string(self.env, self.value)?)
    }

    // This could be optimized to borrow the bytes from the JavaScript value
    // However, since JavaScript values are neither `Send` or `Sync` it is not
    // generally useful
    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'static>,
    {
        self.deserialize_byte_buf(visitor)
    }

    // Bytes are serialized as the idiomatic `ArrayBuffer` JavaScript type
    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'static>,
    {
        visitor.visit_byte_buf(get_value_arraybuffer(self.env, self.value)?)
    }

    // `None` are serialized as `null`, but when deserializing `undefined` is
    // also accepted.
    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'static>,
    {
        match typeof_value(self.env, self.value)? {
            napi::ValueType::Null | napi::ValueType::Undefined => visitor.visit_none(),
            _ => visitor.visit_some(self),
        }
    }

    // JavaScript does not have a concept of unit; `null` or `undefined` is accepted
    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'static>,
    {
        match typeof_value(self.env, self.value)? {
            napi::ValueType::Null | napi::ValueType::Undefined => visitor.visit_unit(),
            _ => Err(Error::expected_null()),
        }
    }

    fn deserialize_unit_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'static>,
    {
        self.deserialize_unit(visitor)
    }

    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'static>,
    {
        visitor.visit_newtype_struct(self)
    }

    // `Array` is used since it is the only sequence type in JavaScript
    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'static>,
    {
        visitor.visit_seq(ArrayAccessor::new(self.env, self.value)?)
    }

    // `Array` are used to serialize tuples; this is a common pattern, especially in TypeScript
    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'static>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'static>,
    {
        self.deserialize_seq(visitor)
    }

    // Generic `Object` are used to serialize map
    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'static>,
    {
        visitor.visit_map(ObjectAccessor::new(self.env, self.value)?)
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'static>,
    {
        self.deserialize_map(visitor)
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'static>,
    {
        // No-value enums are serialized as `string`
        if typeof_value(self.env, self.value)? == napi::ValueType::String {
            let s = get_value_string(self.env, self.value)?;

            visitor.visit_enum(s.into_deserializer())
        } else {
            visitor.visit_enum(self)
        }
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'static>,
    {
        self.deserialize_string(visitor)
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'static>,
    {
        visitor.visit_unit()
    }
}

impl SeqAccess<'static> for ArrayAccessor {
    type Error = Error;

    // This will have unpredictable results if the `Array` has a getter that mutates
    // the object. It should be _safe_ and return an `Error`, but hopefully users
    // don't do this.
    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: DeserializeSeed<'static>,
    {
        self.next()?
            .map(|v| seed.deserialize(Deserializer::new(self.env, v)))
            .transpose()
    }

    // We can efficiently provide a size hint since `Array` have known length
    fn size_hint(&self) -> Option<usize> {
        Some((self.len - self.index) as usize)
    }
}

impl MapAccess<'static> for ObjectAccessor {
    type Error = Error;

    // This will have unpredictable results if the `Object` has a getter that mutates
    // the object. It should be _safe_ and return an `Error`, but hopefully users
    // don't do this on serializable types.
    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: DeserializeSeed<'static>,
    {
        // Store the next `key` for deserializing the value in `next_value_seed`
        self.next = self.keys.next()?;
        self.next
            .map(|v| seed.deserialize(Deserializer::new(self.env, v)))
            .transpose()
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: DeserializeSeed<'static>,
    {
        // `Error::missing_key` should only happen in a buggy serde implementation
        let key = self.next.ok_or_else(Error::missing_key)?;
        let value = get_property(self.env, self.object, key)?;

        seed.deserialize(Deserializer::new(self.env, value))
    }

    // We can efficiently provide a size hint since we fetch all keys ahead of time
    fn size_hint(&self) -> Option<usize> {
        self.keys.size_hint()
    }
}

impl EnumAccess<'static> for Deserializer {
    type Error = Error;
    type Variant = Self;

    // Enums are serialized as `{ [type]: value }`
    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), Self::Error>
    where
        V: DeserializeSeed<'static>,
    {
        let keys = get_property_names(self.env, self.value)?;
        let key = get_array_element(self.env, keys, 0)?;
        let value = get_property(self.env, self.value, key)?;
        let deserializer = Deserializer::new(self.env, value);
        let key = seed.deserialize(self)?;

        Ok((key, deserializer))
    }
}

// Externally tagged enum can be treated equivalent to the enclosed type
impl VariantAccess<'static> for Deserializer {
    type Error = Error;

    fn unit_variant(self) -> Result<(), Self::Error> {
        Err(Error::expected_string())
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, Self::Error>
    where
        T: DeserializeSeed<'static>,
    {
        seed.deserialize(self)
    }

    fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'static>,
    {
        de::Deserializer::deserialize_seq(self, visitor)
    }

    fn struct_variant<V>(
        self,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'static>,
    {
        de::Deserializer::deserialize_map(self, visitor)
    }
}
