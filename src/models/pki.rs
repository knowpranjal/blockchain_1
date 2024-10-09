use ring::rand::SystemRandom;
use ring::signature::{Ed25519KeyPair, KeyPair, Signature, VerificationAlgorithm, ED25519};
use serde::{Serialize, Deserialize};
use serde_bytes;

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct KeyPairWrapper {
    #[serde(with = "serde_bytes")]
    pub pkcs8_bytes: Vec<u8>,
}

impl KeyPairWrapper {
    /// Generates a new key pair and returns a `KeyPairWrapper`.
    pub fn generate() -> Result<Self, ring::error::Unspecified> {
        let rng = SystemRandom::new();
        let pkcs8_bytes = Ed25519KeyPair::generate_pkcs8(&rng)?;
        Ok(KeyPairWrapper {
            pkcs8_bytes: pkcs8_bytes.as_ref().to_vec(),
        })
    }

    /// Reconstructs the `Ed25519KeyPair` from the stored PKCS8 bytes.
    pub fn key_pair(&self) -> Result<Ed25519KeyPair, ring::error::KeyRejected> {
        Ed25519KeyPair::from_pkcs8(&self.pkcs8_bytes)
    }

    /// Returns the public key as an owned `Vec<u8>`.
    pub fn public_key(&self) -> Result<Vec<u8>, ring::error::KeyRejected> {
        Ok(self.key_pair()?.public_key().as_ref().to_vec())
    }

    /// Signs a message and returns the signature.
    pub fn sign(&self, message: &[u8]) -> Result<Signature, ring::error::KeyRejected> {
        Ok(self.key_pair()?.sign(message))
    }

    /// Verifies a signature against a message and a public key.
    pub fn verify(
        public_key: &[u8],
        message: &[u8],
        signature: &[u8],
    ) -> Result<(), ring::error::Unspecified> {
        let peer_public_key = ring::signature::UnparsedPublicKey::new(&ED25519, public_key);
        peer_public_key.verify(message, signature)
    }
}
