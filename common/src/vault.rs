use ring::aead::*;
use ring::pbkdf2::*;
use ring::digest::SHA256;
use ring::rand::{SystemRandom, SecureRandom};
use rand::{thread_rng, Rng};
use crate::log_and_panic;


pub struct Vault {
    pub salt: Vec<u8>,
    pub opening_key: OpeningKey,
    pub sealing_key: SealingKey,
    pub nonce: Vec<u8>,
}

impl Default for Vault {
    fn default() -> Self {
        Self::new()
    }
}

impl Vault {
    pub fn new() -> Self {
        let mut rng = thread_rng();

        let password_size: usize = rng.gen_range(8, 100);
        let mut password = vec![0u8; password_size];
        let ring_rand = SystemRandom::new();
        ring_rand.fill(&mut password)
            .unwrap_or_else(|_| log_and_panic("Cannot fill random password"));

        let salt_size: usize = rng.gen_range(8, 100);
        let mut salt = vec![0u8; salt_size];
        let ring_rand = SystemRandom::new();
        ring_rand.fill(&mut salt)
            .unwrap_or_else(|_| log_and_panic("Cannot fill the salt"));

        let mut key = [0; 32];
        derive(&SHA256, 100, &salt, &password[..], &mut key);

        let opening_key = OpeningKey::new(&CHACHA20_POLY1305, &key)
            .unwrap_or_else(|_| log_and_panic("Cannot generate opening key"));
        let sealing_key = SealingKey::new(&CHACHA20_POLY1305, &key)
            .unwrap_or_else(|_| log_and_panic("Cannot generate sealing key"));

        let mut nonce = vec![0; 12];
        let ring_rand = SystemRandom::new();
        ring_rand.fill(&mut nonce)
            .unwrap_or_else(|_| log_and_panic("Cannot generate nonce"));

        Vault {
            salt,
            opening_key,
            sealing_key,
            nonce
        }

    }

    pub fn encrypt(&self, passwd: &mut String) -> Vec<u8> {
        let passwd: &mut [u8] = unsafe {passwd.as_bytes_mut() };
        let mut passwd = &mut passwd.to_vec();
        let additional_data: [u8; 0] = [];
        for _ in 0..CHACHA20_POLY1305.tag_len() {
            passwd.push(0);
        }
        let _ = seal_in_place(&self.sealing_key, &self.nonce,
                              &additional_data, &mut passwd,
                                    CHACHA20_POLY1305.tag_len())
            .unwrap_or_else(|_| log_and_panic("Cannot encrypt password"));
        passwd.clone()
    }

    pub fn decrypt(&self, passwd: &[u8]) -> String {
        let mut passwd = passwd.to_owned();
        let additional_data: [u8; 0] = [];
        let res = open_in_place(&self.opening_key, &self.nonce,
                                &additional_data, 0, &mut passwd)
            .unwrap_or_else(|_| log_and_panic("Cannot decrypt password"));
        String::from_utf8(res.to_vec())
            .unwrap_or_else(|_| 
                log_and_panic("Cannot convert the decrypted password to text"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encryption() {
        let vault = Vault::new();
        let original = String::from("very$secure*passw0rd#");
        let mut encrypted = original.to_owned();
        let encrypted = &mut vault.encrypt(&mut encrypted);
        let decrypted = &mut vault.decrypt(encrypted.as_slice());
        assert_ne!(original.clone().into_bytes(), *encrypted);
        assert_eq!(*original, *decrypted);
    }
}
