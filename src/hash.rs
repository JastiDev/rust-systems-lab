use crate::error::HashError;
use bincode::config;
use bincode::serde::{decode_from_slice, encode_to_vec};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HashedId([u8; 32]);

impl HashedId {
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    pub fn to_hex(&self) -> String {
        hex::encode(self.0)
    }
}

impl std::fmt::Display for HashedId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

pub trait HashFn {
    fn hash_id(&self) -> HashedId;
    fn canonical_bytes(&self) -> Vec<u8>;
    fn from_canonical_bytes(bytes: &[u8]) -> Result<Self, HashError>
    where
        Self: std::marker::Sized;
}

impl<T> HashFn for T
where
    T: Serialize + for<'de> Deserialize<'de>,
{
    fn hash_id(&self) -> HashedId {
        let digest = Sha256::digest(self.canonical_bytes());
        HashedId(digest.into())
    }

    fn canonical_bytes(&self) -> Vec<u8> {
        let config = config::standard();
        encode_to_vec(self, config).expect("canonical encoding should not fail")
    }

    fn from_canonical_bytes(bytes: &[u8]) -> Result<Self, HashError> {
        let config = config::standard();
        decode_from_slice(bytes, config)
            .map(|(transaction, _)| transaction)
            .map_err(|_| HashError::CannotDecode)
    }
}
