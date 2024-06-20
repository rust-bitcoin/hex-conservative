//! Hex encoding with `serde`.
//!
//! The functions in this module de/serialize as hex _only_ when the serializer is human readable.
//!
//! # Examples
//!
//! ```
//! # #[cfg(feature = "std")] {
//! use hex_conservative as hex;
//! use serde::{Serialize, Deserialize};
//!
//! #[derive(Debug, Serialize, Deserialize)]
//! struct Foo {
//!     #[serde(with = "hex::serde")]
//!     bar: Vec<u8>,
//! }
//! # }
//! ```

use core::fmt;
use core::marker::PhantomData;

use serde::de::{Error, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::prelude::*;

/// Serializes `data` as a hex string using lowercase characters.
///
/// We only serialize as hex if the serializer is human readable, if not we call through to the
/// `Serialize` implementation for `data`.
pub fn serialize<S, T>(data: T, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    T: Serialize + DisplayHex,
{
    serialize_lower(data, s)
}

/// Serializes `data` as a hex string using lowercase characters.
///
/// We only serialize as hex if the serializer is human readable, if not we call through to the
/// `Serialize` implementation for `data`.
pub fn serialize_lower<S, T>(data: T, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    T: Serialize + DisplayHex,
{
    // Don't do anything special when not human readable.
    if !serializer.is_human_readable() {
        serde::Serialize::serialize(&data, serializer)
    } else {
        serializer.collect_str(&format_args!("{:x}", data.as_hex()))
    }
}

/// Serializes `data` as hex string using uppercase characters.
///
/// We only serialize as hex if the serializer is human readable, if not we call through to the
/// `Serialize` implementation for `data`.
pub fn serialize_upper<S, T>(data: T, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    T: Serialize + DisplayHex,
{
    // Don't do anything special when not human readable.
    if !serializer.is_human_readable() {
        serde::Serialize::serialize(&data, serializer)
    } else {
        serializer.collect_str(&format_args!("{:X}", data.as_hex()))
    }
}

/// Deserializes a hex string into raw bytes.
///
/// Allows upper, lower, and mixed case characters (e.g. `a5b3c1`, `A5B3C1` and `A5b3C1`).
///
/// We only deserialize from hex if the serializer is human readable, if not we call through to the
/// `Deserialize` implementation for `T`.
pub fn deserialize<'de, D, T>(d: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de> + FromHex,
{
    struct HexVisitor<T>(PhantomData<T>);

    impl<'de, T> Visitor<'de> for HexVisitor<T>
    where
        T: FromHex,
    {
        type Value = T;

        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.write_str("an ASCII hex string")
        }

        fn visit_str<E: Error>(self, data: &str) -> Result<Self::Value, E> {
            FromHex::from_hex(data).map_err(Error::custom)
        }
    }

    // Don't do anything special when not human readable.
    if !d.is_human_readable() {
        serde::Deserialize::deserialize(d)
    } else {
        d.deserialize_map(HexVisitor(PhantomData))
    }
}

pub(crate) struct SerializeBytesAsHex<'a>(pub(crate) &'a [u8]);

impl<'a> serde::Serialize for SerializeBytesAsHex<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use crate::DisplayHex;

        serializer.collect_str(&format_args!("{:x}", self.0.as_hex()))
    }
}

#[cfg(feature = "alloc")]
pub mod btreemap_byte_values {
    //! Module for serialization of BTreeMaps with hex byte values.
    #![allow(missing_docs)]

    // NOTE: This module can be exactly copied to use with HashMap.

    use crate::alloc::collections::BTreeMap;
    #[cfg(all(feature = "alloc", not(feature = "std")))]
    use crate::alloc::vec::Vec;
    use crate::FromHex;

    pub fn serialize<S, T>(v: &BTreeMap<T, Vec<u8>>, s: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
        T: serde::Serialize + core::hash::Hash + Eq + Ord,
    {
        use serde::ser::SerializeMap;

        // Don't do anything special when not human readable.
        if !s.is_human_readable() {
            serde::Serialize::serialize(v, s)
        } else {
            let mut map = s.serialize_map(Some(v.len()))?;
            for (key, value) in v.iter() {
                map.serialize_entry(key, &super::SerializeBytesAsHex(value))?;
            }
            map.end()
        }
    }

    pub fn deserialize<'de, D, T>(d: D) -> Result<BTreeMap<T, Vec<u8>>, D::Error>
    where
        D: serde::Deserializer<'de>,
        T: serde::Deserialize<'de> + core::hash::Hash + Eq + Ord,
    {
        use core::marker::PhantomData;

        struct Visitor<T>(PhantomData<T>);
        impl<'de, T> serde::de::Visitor<'de> for Visitor<T>
        where
            T: serde::Deserialize<'de> + core::hash::Hash + Eq + Ord,
        {
            type Value = BTreeMap<T, Vec<u8>>;

            fn expecting(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
                write!(f, "a map with hexadecimal values")
            }

            fn visit_map<A: serde::de::MapAccess<'de>>(
                self,
                mut a: A,
            ) -> Result<Self::Value, A::Error> {
                let mut ret = BTreeMap::new();
                while let Some((key, value)) = a.next_entry()? {
                    ret.insert(key, FromHex::from_hex(value).map_err(serde::de::Error::custom)?);
                }
                Ok(ret)
            }
        }

        // Don't do anything special when not human readable.
        if !d.is_human_readable() {
            serde::Deserialize::deserialize(d)
        } else {
            d.deserialize_map(Visitor(PhantomData))
        }
    }
}

#[cfg(feature = "alloc")]
pub mod btreemap_as_seq {
    //! Module for serialization of BTreeMaps as lists of sequences because
    //! serde_json will not serialize hashmaps with non-string keys be default.
    #![allow(missing_docs)]

    // NOTE: This module can be exactly copied to use with HashMap.

    use alloc::collections::BTreeMap;

    pub fn serialize<S, T, U>(v: &BTreeMap<T, U>, s: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
        T: serde::Serialize + core::hash::Hash + Eq + Ord,
        U: serde::Serialize,
    {
        use serde::ser::SerializeSeq;

        // Don't do anything special when not human readable.
        if !s.is_human_readable() {
            serde::Serialize::serialize(v, s)
        } else {
            let mut seq = s.serialize_seq(Some(v.len()))?;
            for pair in v.iter() {
                seq.serialize_element(&pair)?;
            }
            seq.end()
        }
    }

    pub fn deserialize<'de, D, T, U>(d: D) -> Result<BTreeMap<T, U>, D::Error>
    where
        D: serde::Deserializer<'de>,
        T: serde::Deserialize<'de> + core::hash::Hash + Eq + Ord,
        U: serde::Deserialize<'de>,
    {
        use core::marker::PhantomData;

        struct Visitor<T, U>(PhantomData<(T, U)>);
        impl<'de, T, U> serde::de::Visitor<'de> for Visitor<T, U>
        where
            T: serde::Deserialize<'de> + core::hash::Hash + Eq + Ord,
            U: serde::Deserialize<'de>,
        {
            type Value = BTreeMap<T, U>;

            fn expecting(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
                write!(f, "a sequence of pairs")
            }

            fn visit_seq<A: serde::de::SeqAccess<'de>>(
                self,
                mut a: A,
            ) -> Result<Self::Value, A::Error> {
                let mut ret = BTreeMap::new();
                while let Some((key, value)) = a.next_element()? {
                    ret.insert(key, value);
                }
                Ok(ret)
            }
        }

        // Don't do anything special when not human readable.
        if !d.is_human_readable() {
            serde::Deserialize::deserialize(d)
        } else {
            d.deserialize_seq(Visitor(PhantomData))
        }
    }
}

#[cfg(feature = "alloc")]
pub mod btreemap_as_seq_byte_values {
    //! Module for serialization of BTreeMaps as lists of sequences because
    //! serde_json will not serialize hashmaps with non-string keys be default.
    #![allow(missing_docs)]

    // NOTE: This module can be exactly copied to use with HashMap.

    use serde::{Deserialize, Serialize};

    use crate::alloc::collections::BTreeMap;
    #[cfg(all(feature = "alloc", not(feature = "std")))]
    use crate::alloc::vec::Vec;

    /// A custom key-value pair type that serialized the bytes as hex.
    #[derive(Debug, Deserialize)]
    struct OwnedPair<T>(
        T,
        #[serde(deserialize_with = "crate::serde::hex_bytes::deserialize")] Vec<u8>,
    );

    /// A custom key-value pair type that serialized the bytes as hex.
    #[derive(Debug, Serialize)]
    struct BorrowedPair<'a, T: 'static>(
        &'a T,
        #[serde(serialize_with = "crate::serde::hex_bytes::serialize")] &'a [u8],
    );

    pub fn serialize<S, T>(v: &BTreeMap<T, Vec<u8>>, s: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
        T: serde::Serialize + core::hash::Hash + Eq + Ord + 'static,
    {
        use serde::ser::SerializeSeq;

        // Don't do anything special when not human readable.
        if !s.is_human_readable() {
            serde::Serialize::serialize(v, s)
        } else {
            let mut seq = s.serialize_seq(Some(v.len()))?;
            for (key, value) in v.iter() {
                seq.serialize_element(&BorrowedPair(key, value))?;
            }
            seq.end()
        }
    }

    pub fn deserialize<'de, D, T>(d: D) -> Result<BTreeMap<T, Vec<u8>>, D::Error>
    where
        D: serde::Deserializer<'de>,
        T: serde::Deserialize<'de> + core::hash::Hash + Eq + Ord,
    {
        use core::marker::PhantomData;

        struct Visitor<T>(PhantomData<T>);
        impl<'de, T> serde::de::Visitor<'de> for Visitor<T>
        where
            T: serde::Deserialize<'de> + core::hash::Hash + Eq + Ord,
        {
            type Value = BTreeMap<T, Vec<u8>>;

            fn expecting(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
                write!(f, "a sequence of pairs")
            }

            fn visit_seq<A: serde::de::SeqAccess<'de>>(
                self,
                mut a: A,
            ) -> Result<Self::Value, A::Error> {
                let mut ret = BTreeMap::new();
                while let Option::Some(OwnedPair(key, value)) = a.next_element()? {
                    ret.insert(key, value);
                }
                Ok(ret)
            }
        }

        // Don't do anything special when not human readable.
        if !d.is_human_readable() {
            serde::Deserialize::deserialize(d)
        } else {
            d.deserialize_seq(Visitor(PhantomData))
        }
    }
}

mod hex_bytes {
    //! Module for serialization of byte arrays as hex strings.
    #![allow(missing_docs)]

    use crate::FromHex;

    pub fn serialize<T, S>(bytes: &T, s: S) -> Result<S::Ok, S::Error>
    where
        T: serde::Serialize + AsRef<[u8]>,
        S: serde::Serializer,
    {
        // Don't do anything special when not human readable.
        if !s.is_human_readable() {
            serde::Serialize::serialize(bytes, s)
        } else {
            serde::Serialize::serialize(&super::SerializeBytesAsHex(bytes.as_ref()), s)
        }
    }

    pub fn deserialize<'de, D, B>(d: D) -> Result<B, D::Error>
    where
        D: serde::Deserializer<'de>,
        B: serde::Deserialize<'de> + FromHex,
    {
        struct Visitor<B>(core::marker::PhantomData<B>);

        impl<'de, B: FromHex> serde::de::Visitor<'de> for Visitor<B> {
            type Value = B;

            fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
                formatter.write_str("an ASCII hex string")
            }

            fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                if let Ok(hex) = core::str::from_utf8(v) {
                    FromHex::from_hex(hex).map_err(E::custom)
                } else {
                    return Err(E::invalid_value(serde::de::Unexpected::Bytes(v), &self));
                }
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                FromHex::from_hex(v).map_err(E::custom)
            }
        }

        // Don't do anything special when not human readable.
        if !d.is_human_readable() {
            serde::Deserialize::deserialize(d)
        } else {
            d.deserialize_str(Visitor(core::marker::PhantomData))
        }
    }
}
