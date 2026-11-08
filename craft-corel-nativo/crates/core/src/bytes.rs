//! Picture bytes shared between copies of a document.
//!
//! Undo steps, background renders and copied objects all clone the
//! drawing; with plain `Vec<u8>` every clone copied every picture. A
//! [`SharedBytes`] clone is a reference count, and code that makes new
//! pixels replaces the whole value. In the JSON format it is the same
//! base64 text as before, so files do not change.

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::ops::Deref;
use std::sync::Arc;

/// Immutable bytes behind a shared pointer (a PNG, as a rule).
#[derive(Clone, Default)]
pub struct SharedBytes(Arc<[u8]>);

impl SharedBytes {
    /// Whether both share one allocation (a cheap "same picture" test).
    pub fn ptr_eq(a: &SharedBytes, b: &SharedBytes) -> bool {
        Arc::ptr_eq(&a.0, &b.0)
    }

    /// The bytes as a slice.
    pub fn as_slice(&self) -> &[u8] {
        &self.0
    }

    /// The address of the shared allocation, a key for caches. Once every
    /// clone is gone the address can be reused by other bytes, so a cache
    /// that outlives the value it was keyed from must keep a clone of it.
    pub fn addr(&self) -> usize {
        self.0.as_ptr() as usize
    }
}

impl Deref for SharedBytes {
    type Target = [u8];
    fn deref(&self) -> &[u8] {
        &self.0
    }
}

impl AsRef<[u8]> for SharedBytes {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl From<Vec<u8>> for SharedBytes {
    fn from(v: Vec<u8>) -> Self {
        SharedBytes(v.into())
    }
}

impl From<&[u8]> for SharedBytes {
    fn from(v: &[u8]) -> Self {
        SharedBytes(v.into())
    }
}

impl<const N: usize> From<[u8; N]> for SharedBytes {
    fn from(v: [u8; N]) -> Self {
        SharedBytes(v.as_slice().into())
    }
}

impl From<SharedBytes> for Vec<u8> {
    fn from(v: SharedBytes) -> Self {
        v.0.to_vec()
    }
}

impl PartialEq for SharedBytes {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0) || self.0[..] == other.0[..]
    }
}

impl Eq for SharedBytes {}

impl PartialEq<[u8]> for SharedBytes {
    fn eq(&self, other: &[u8]) -> bool {
        self.0[..] == *other
    }
}

impl PartialEq<Vec<u8>> for SharedBytes {
    fn eq(&self, other: &Vec<u8>) -> bool {
        self.0[..] == other[..]
    }
}

impl PartialEq<SharedBytes> for Vec<u8> {
    fn eq(&self, other: &SharedBytes) -> bool {
        self[..] == other.0[..]
    }
}

impl std::hash::Hash for SharedBytes {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0[..].hash(state)
    }
}

/// Prints like the byte list it replaces.
impl std::fmt::Debug for SharedBytes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0[..].fmt(f)
    }
}

/// Base64 text, as `png_bytes` writes a byte vector.
impl Serialize for SharedBytes {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        crate::document::png_bytes::serialize(&self.0, s)
    }
}

impl<'de> Deserialize<'de> for SharedBytes {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        crate::document::png_bytes::deserialize(d).map(SharedBytes::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clones_share_and_serde_matches_the_byte_vector_form() {
        let a = SharedBytes::from(vec![1u8, 2, 3, 250]);
        let b = a.clone();
        assert!(SharedBytes::ptr_eq(&a, &b));
        assert_eq!(a, vec![1u8, 2, 3, 250]);
        #[derive(Serialize)]
        struct Old {
            #[serde(with = "crate::document::png_bytes")]
            png: Vec<u8>,
        }
        #[derive(Serialize, Deserialize)]
        struct New {
            png: SharedBytes,
        }
        let old = serde_json::to_string(&Old {
            png: vec![1, 2, 3, 250],
        })
        .unwrap();
        let new = serde_json::to_string(&New { png: a.clone() }).unwrap();
        assert_eq!(old, new);
        let back: New = serde_json::from_str(&old).unwrap();
        assert_eq!(back.png, a);
        assert_eq!(format!("{a:?}"), format!("{:?}", vec![1u8, 2, 3, 250]));
    }
}
