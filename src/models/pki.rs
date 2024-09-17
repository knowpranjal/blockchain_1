use ring::rand::SystemRandom;
use ring::signature::{Ed25519KeyPair, KeyPair, Signature, VerificationAlgorithm, ED25519};

pub struct KeyPairWrapper {
    pub key_pair: Ed25519KeyPair,
}

impl KeyPairWrapper {
    pub fn generate() -> Result<Self, ring::error::Unspecified> {
        let rng = SystemRandom::new();
        let pkcs8_bytes = Ed25519KeyPair::generate_pkcs8(&rng)?;
        let key_pair = Ed25519KeyPair::from_pkcs8(pkcs8_bytes.as_ref())?;
        Ok(KeyPairWrapper { key_pair })
    }

    pub fn public_key(&self) -> &[u8] {
        self.key_pair.public_key().as_ref()
    }

    pub fn sign(&self, message: &[u8]) -> Signature {
        self.key_pair.sign(message)
    }

    pub fn verify(
        public_key: &[u8],
        message: &[u8],
        signature: &[u8],
    ) -> Result<(), ring::error::Unspecified> {
        let peer_public_key = ring::signature::UnparsedPublicKey::new(&ED25519, public_key);
        peer_public_key.verify(message, signature)
    }
}


fn main() -> Result<(), ring::error::Unspecified> {
    // Generate a key pair
    let key_pair_wrapper = KeyPairWrapper::generate()?;

    // Get public key
    let public_key = key_pair_wrapper.public_key();

    // Sign a message
    let message = b"Hello, world!";
    let signature = key_pair_wrapper.sign(message);

    // Verify the signature
    KeyPairWrapper::verify(public_key, message, signature.as_ref())?;
    println!("Signature verified successfully!");

    Ok(())
}
