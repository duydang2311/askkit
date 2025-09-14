use aes_gcm::{
    aead::{generic_array::GenericArray, Aead, OsRng},
    AeadCore, Aes256Gcm, Key, KeyInit,
};
use base64::{prelude::BASE64_STANDARD, Engine};
use keyring::Entry;

use crate::common::error::AppError;

pub trait Cipher: Send + Sync {
    fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>, AppError>;
    fn decrypt(&self, ciphertext: &[u8]) -> Result<Vec<u8>, AppError>;
    fn encrypt_str_base64(&self, plaintext: &str) -> Result<String, AppError>;
    fn decrypt_base64_str(&self, ciphertext_base64: &str) -> Result<String, AppError>;
}

pub struct KeyringAesGcmCipher {}

impl KeyringAesGcmCipher {
    pub fn new() -> Self {
        KeyringAesGcmCipher {}
    }
}

impl Cipher for KeyringAesGcmCipher {
    fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>, AppError> {
        let cipher = create_aes256gcm()?;
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        let ciphertext = cipher.encrypt(&nonce, plaintext).map_err(AppError::from)?;
        Ok([nonce.as_slice(), ciphertext.as_slice()].concat())
    }

    fn decrypt(&self, ciphertext: &[u8]) -> Result<Vec<u8>, AppError> {
        println!("decrypt ciphertext {:?}", ciphertext);
        let cipher = create_aes256gcm()?;
        let nonce = GenericArray::from_slice(&ciphertext[..12]);
        let ciphertext = &ciphertext[12..];
        cipher.decrypt(nonce, ciphertext).map_err(AppError::from)
    }

    fn encrypt_str_base64(&self, plaintext: &str) -> Result<String, AppError> {
        Ok(BASE64_STANDARD.encode(self.encrypt(plaintext.as_bytes())?))
    }

    fn decrypt_base64_str(&self, ciphertext_base64: &str) -> Result<String, AppError> {
        self.decrypt(
            BASE64_STANDARD
                .decode(ciphertext_base64)
                .map_err(AppError::from)?
                .as_slice(),
        )
        .and_then(|a| String::from_utf8(a).map_err(AppError::from))
    }
}

fn create_aes256gcm() -> Result<Aes256Gcm, AppError> {
    let entry = Entry::new("askkit", "local").map_err(AppError::from)?;
    let key = entry
        .get_secret()
        .map(|a| Key::<Aes256Gcm>::from_slice(a.as_slice()).to_owned())
        .or_else(|e| match e {
            keyring::Error::NoEntry => {
                let key = Aes256Gcm::generate_key(OsRng);
                entry.set_secret(key.as_slice()).map_err(AppError::from)?;
                Ok(key)
            }
            e => Err(AppError::from(e)),
        })?;
    Ok(Aes256Gcm::new(&key))
}
