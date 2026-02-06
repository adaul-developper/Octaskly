use anyhow::Result;

/// Security module for optional encryption and authentication
pub struct Security {
    preshared_key: Option<String>,
    whitelist: Vec<String>,
}

impl Security {
    pub fn new() -> Self {
        Self {
            preshared_key: None,
            whitelist: Vec::new(),
        }
    }

    /// Set pre-shared key for authentication
    pub fn set_preshared_key(&mut self, key: String) {
        self.preshared_key = Some(key);
    }

    /// Verify pre-shared key
    pub fn verify_key(&self, key: &str) -> bool {
        match &self.preshared_key {
            Some(stored_key) => stored_key == key,
            None => true, // No key set, allow all
        }
    }

    /// Add worker to whitelist
    pub fn add_to_whitelist(&mut self, worker_id: String) {
        self.whitelist.push(worker_id);
    }

    /// Check if worker is whitelisted
    pub fn is_whitelisted(&self, worker_id: &str) -> bool {
        if self.whitelist.is_empty() {
            return true; // No whitelist, allow all
        }
        self.whitelist.contains(&worker_id.to_string())
    }

    /// Encrypt data (stub for future implementation)
    pub fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>> {
        // TODO: Implement actual encryption
        Ok(data.to_vec())
    }

    /// Decrypt data (stub for future implementation)
    pub fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>> {
        // TODO: Implement actual decryption
        Ok(data.to_vec())
    }
}

impl Default for Security {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_security_no_key() {
        let security = Security::new();
        assert!(security.verify_key("anything"));
    }

    #[test]
    fn test_security_with_key() {
        let mut security = Security::new();
        security.set_preshared_key("secret".to_string());

        assert!(security.verify_key("secret"));
        assert!(!security.verify_key("wrong"));
    }
}
