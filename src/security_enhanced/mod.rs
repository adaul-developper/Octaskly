use anyhow::Result;
use aes_gcm::{Aes256Gcm, Key, Nonce, aead::Aead};
use rand::Rng;
use sha2::{Sha256, Digest};

/// Enhanced security module with encryption and key management
pub struct SecurityManager {
    preshared_key: String,
    whitelist: Vec<String>,
    cipher: Option<Aes256Gcm>,
}

impl SecurityManager {
    pub fn new(preshared_key: String) -> Self {
        // Derive encryption key from preshared key
        let mut hasher = Sha256::new();
        hasher.update(preshared_key.as_bytes());
        let key_bytes = hasher.finalize();
        
        let key = Key::<Aes256Gcm>::from_slice(&key_bytes[..]);
        let cipher = <Aes256Gcm as aes_gcm::KeyInit>::new(key);

        Self {
            preshared_key,
            whitelist: Vec::new(),
            cipher: Some(cipher),
        }
    }

    /// Create key from password
    pub fn derive_key(password: &str) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(password.as_bytes());
        let result = hasher.finalize();
        
        let mut key = [0u8; 32];
        key.copy_from_slice(&result[..32]);
        key
    }

    /// Encrypt data with AES-256-GCM
    pub fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>> {
        if let Some(cipher) = &self.cipher {
            let mut rng = rand::thread_rng();
            let nonce_bytes: [u8; 12] = rng.gen();
            let nonce = Nonce::from_slice(&nonce_bytes);
            
            let ciphertext = cipher.encrypt(nonce, plaintext)
                .map_err(|e| anyhow::anyhow!("Encryption failed: {}", e))?;
            
            // Prepend nonce to ciphertext
            let mut result = nonce_bytes.to_vec();
            result.extend_from_slice(&ciphertext);
            Ok(result)
        } else {
            Ok(plaintext.to_vec())
        }
    }

    /// Decrypt data with AES-256-GCM
    pub fn decrypt(&self, encrypted_data: &[u8]) -> Result<Vec<u8>> {
        if let Some(cipher) = &self.cipher {
            if encrypted_data.len() < 12 {
                return Err(anyhow::anyhow!("Invalid encrypted data"));
            }
            
            let nonce = Nonce::from_slice(&encrypted_data[..12]);
            let ciphertext = &encrypted_data[12..];
            
            cipher.decrypt(nonce, ciphertext)
                .map_err(|e| anyhow::anyhow!("Decryption failed: {}", e))
        } else {
            Ok(encrypted_data.to_vec())
        }
    }

    /// Verify pre-shared key
    pub fn verify_key(&self, key: &str) -> bool {
        self.preshared_key == key
    }

    /// Add worker to whitelist
    pub fn add_to_whitelist(&mut self, worker_id: String) {
        if !self.whitelist.contains(&worker_id) {
            self.whitelist.push(worker_id);
        }
    }

    /// Check if worker is whitelisted
    pub fn is_whitelisted(&self, worker_id: &str) -> bool {
        self.whitelist.is_empty() || self.whitelist.contains(&worker_id.to_string())
    }

    /// Generate HMAC token
    pub fn generate_token(&self) -> String {
        use hmac::{Hmac, Mac};
        
        type HmacSha256 = Hmac<sha2::Sha256>;
        
        let mut mac = HmacSha256::new_from_slice(self.preshared_key.as_bytes())
            .expect("Invalid key length");
        
        let timestamp = chrono::Utc::now().timestamp();
        mac.update(timestamp.to_string().as_bytes());
        
        let result = mac.finalize().into_bytes();
        result.iter().map(|b| format!("{:02x}", b)).collect::<String>()
    }

    /// Verify HMAC token
    pub fn verify_token(&self, token: &str) -> bool {
        use hmac::{Hmac, Mac};
        
        type HmacSha256 = Hmac<sha2::Sha256>;
        
        let mut mac = HmacSha256::new_from_slice(self.preshared_key.as_bytes())
            .expect("Invalid key length");
        
        let timestamp = chrono::Utc::now().timestamp();
        mac.update(timestamp.to_string().as_bytes());
        
        let result = mac.finalize().into_bytes();
        let expected = result.iter().map(|b| format!("{:02x}", b)).collect::<String>();
        expected == token
    }
}

impl Default for SecurityManager {
    fn default() -> Self {
        Self::new("default-secret-key".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encryption_decryption() {
        let manager = SecurityManager::new("test-key".to_string());
        let plaintext = b"Hello, OCTASKLY!";
        
        let encrypted = manager.encrypt(plaintext).unwrap();
        let decrypted = manager.decrypt(&encrypted).unwrap();
        
        assert_eq!(plaintext, decrypted.as_slice());
    }

    #[test]
    fn test_key_verification() {
        let manager = SecurityManager::new("secret".to_string());
        assert!(manager.verify_key("secret"));
        assert!(!manager.verify_key("wrong"));
    }

    #[test]
    fn test_whitelist() {
        let mut manager = SecurityManager::new("test".to_string());
        assert!(manager.is_whitelisted("any-worker"));
        
        manager.add_to_whitelist("worker-1".to_string());
        assert!(manager.is_whitelisted("worker-1"));
        assert!(!manager.is_whitelisted("worker-2"));
    }
}
