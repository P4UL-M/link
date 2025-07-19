use rand::rngs::OsRng;
use rsa::pkcs1::{DecodeRsaPublicKey, EncodeRsaPublicKey};
use rsa::traits::PublicKeyParts;
use rsa::{Pkcs1v15Encrypt, RsaPrivateKey, RsaPublicKey};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Key {
    pub public_key: RsaPublicKey,
    pub private_key: RsaPrivateKey,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SerializableKey {
    pub public_key: Vec<u8>, // Using Vec<u8> for serialization
    pub name: Vec<u8>,       // Using Vec<u8> for serialization
}

impl Key {
    pub fn new(public_key: RsaPublicKey, private_key: RsaPrivateKey) -> Self {
        Key {
            public_key,
            private_key,
        }
    }

    pub fn generate() -> Self {
        let mut rng = OsRng::default();
        let private_key = RsaPrivateKey::new(&mut rng, 512).expect("key gen failed");
        let public_key = RsaPublicKey::from(&private_key);
        Key {
            public_key,
            private_key,
        }
    }

    pub fn is_valid(&self) -> bool {
        // You can customize validation logic if needed
        self.public_key.n().bits() > 0 && self.private_key.n().bits() > 0
    }

    pub fn encode_message(&self, message: &str) -> Vec<u8> {
        // encode the message using the public key
        self.public_key
            .encrypt(&mut OsRng, Pkcs1v15Encrypt, message.as_bytes())
            .unwrap()
    }

    pub fn decode_message(&self, encrypted_message: &[u8]) -> String {
        // decode the message using the private key
        let decrypted = self
            .private_key
            .decrypt(Pkcs1v15Encrypt, encrypted_message)
            .expect("Decryption failed");
        String::from_utf8(decrypted).expect("Invalid UTF-8")
    }

    pub fn to_serializable(&self) -> SerializableKey {
        SerializableKey {
            public_key: self.public_key.to_pkcs1_der().unwrap().as_bytes().to_vec(),
            name: self.public_key.n().to_bytes_be(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PublicKey {
    pub key: RsaPublicKey,
}

impl PublicKey {
    pub fn new(key: RsaPublicKey) -> Self {
        PublicKey { key }
    }

    pub fn to_serializable(&self) -> SerializableKey {
        SerializableKey {
            public_key: self.key.to_pkcs1_der().unwrap().as_bytes().to_vec(),
            name: self.key.n().to_bytes_be(),
        }
    }

    pub fn from_serializable(serial: &SerializableKey) -> Self {
        let public_key = RsaPublicKey::from_pkcs1_der(&serial.public_key).unwrap();

        PublicKey { key: public_key }
    }

    pub fn encode_message(&self, message: &str) -> Vec<u8> {
        // encode the message using the public key
        self.key
            .encrypt(&mut OsRng, Pkcs1v15Encrypt, message.as_bytes())
            .unwrap()
    }
}
