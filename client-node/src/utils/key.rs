use aes::Aes256;
use aes::cipher::{KeyIvInit, StreamCipher};
use ctr::Ctr128BE;
use rand::RngCore;
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
#[derive(Debug, Clone)]
pub struct EncryptedPayload {
    pub encrypted_key: Vec<u8>,
    pub encrypted_key_size: u8,
    pub iv: Vec<u8>,
    pub ciphertext: Vec<u8>,
}

impl EncryptedPayload {
    pub fn new(encrypted_key: Vec<u8>, iv: Vec<u8>, ciphertext: Vec<u8>) -> Self {
        let encrypted_key_size = encrypted_key.len() as u8;
        EncryptedPayload {
            encrypted_key_size,
            encrypted_key,
            iv,
            ciphertext,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut data = Vec::new();
        data.extend(&self.encrypted_key_size.to_be_bytes());
        data.extend(&self.encrypted_key);
        data.extend(&self.iv);
        data.extend(&self.ciphertext);
        data
    }

    pub fn from_bytes(data: &[u8]) -> Self {
        if data.len() < 1 + 32 + 16 {
            panic!("Data too short to contain encrypted payload");
        }
        let encrypted_key_size = data[0] as usize;
        if data.len() < 1 + encrypted_key_size as usize + 16 {
            panic!("Data too short to contain encrypted payload");
        }
        let iv_size = 16; // Size of AES IV

        let encrypted_key = data[1..encrypted_key_size + 1].to_vec();
        let iv = data[encrypted_key_size + 1..encrypted_key_size + 1 + iv_size].to_vec();
        let ciphertext = data[encrypted_key_size + 1 + iv_size..].to_vec();

        EncryptedPayload {
            encrypted_key,
            encrypted_key_size: encrypted_key_size as u8,
            iv,
            ciphertext,
        }
    }
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
        type Aes256Ctr = Ctr128BE<Aes256>;

        // Generate AES key and IV
        let mut aes_key = [0u8; 32];
        let mut aes_iv = [0u8; 16];
        let mut rng = rand::thread_rng();
        rng.fill_bytes(&mut aes_key);
        rng.fill_bytes(&mut aes_iv);

        // Encrypt the message using AES
        let mut message_bytes = message.as_bytes().to_vec();
        let mut cipher = Aes256Ctr::new_from_slices(&aes_key, &aes_iv).unwrap();
        cipher.apply_keystream(&mut message_bytes);

        // Encrypt the AES key using RSA
        let encrypted_aes_key = self
            .public_key
            .encrypt(&mut OsRng, Pkcs1v15Encrypt, &aes_key)
            .unwrap();

        EncryptedPayload::new(encrypted_aes_key, aes_iv.to_vec(), message_bytes).to_bytes()
    }

    pub fn decode_message(&self, encrypted_message: &[u8]) -> String {
        type Aes256Ctr = Ctr128BE<Aes256>;

        let payload = EncryptedPayload::from_bytes(encrypted_message);

        // Decrypt the AES key using the RSA private key
        let aes_key = self
            .private_key
            .decrypt(Pkcs1v15Encrypt, &payload.encrypted_key)
            .unwrap();

        // Decrypt the message using AES
        let mut cipher = Aes256Ctr::new_from_slices(&aes_key, &payload.iv).unwrap();
        let mut decrypted_message = payload.ciphertext.clone();
        cipher.apply_keystream(&mut decrypted_message);

        String::from_utf8(decrypted_message).expect("Invalid UTF-8")
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
        type Aes256Ctr = Ctr128BE<Aes256>;

        // Generate AES key and IV
        let mut aes_key = [0u8; 32];
        let mut aes_iv = [0u8; 16];
        let mut rng = rand::thread_rng();
        rng.fill_bytes(&mut aes_key);
        rng.fill_bytes(&mut aes_iv);

        // Encrypt the message using AES
        let mut message_bytes = message.as_bytes().to_vec();
        let mut cipher = Aes256Ctr::new_from_slices(&aes_key, &aes_iv).unwrap();
        cipher.apply_keystream(&mut message_bytes);

        // Encrypt the AES key using RSA
        let encrypted_aes_key = self
            .key
            .encrypt(&mut OsRng, Pkcs1v15Encrypt, &aes_key)
            .unwrap();

        EncryptedPayload::new(encrypted_aes_key, aes_iv.to_vec(), message_bytes).to_bytes()
    }
}
