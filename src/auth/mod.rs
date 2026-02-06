use anyhow::Result;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, TokenData, Validation};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use chrono::{Utc, Duration};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,           // subject (worker_id or user_id)
    pub exp: i64,              // expiration time
    pub iat: i64,              // issued at
    pub role: String,          // role (dispatcher, worker, admin)
    pub permissions: Vec<String>, // specific permissions
}

impl Claims {
    pub fn new(sub: String, role: String, permissions: Vec<String>) -> Self {
        let now = Utc::now();
        let exp = now + Duration::hours(24);
        
        Self {
            sub,
            exp: exp.timestamp(),
            iat: now.timestamp(),
            role,
            permissions,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AuthManager {
    secret: String,
    tokens: Arc<RwLock<HashMap<String, Claims>>>,
}

impl AuthManager {
    pub fn new(secret: String) -> Self {
        Self {
            secret,
            tokens: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Generate JWT token
    pub fn generate_token(&self, claims: &Claims) -> Result<String> {
        let encoding_key = EncodingKey::from_secret(self.secret.as_bytes());
        let token = encode(&Header::default(), claims, &encoding_key)?;
        Ok(token)
    }

    /// Verify and decode JWT token
    pub fn verify_token(&self, token: &str) -> Result<Claims> {
        let decoding_key = DecodingKey::from_secret(self.secret.as_bytes());
        let token_data: TokenData<Claims> = decode(
            token,
            &decoding_key,
            &Validation::default(),
        )?;
        Ok(token_data.claims)
    }

    /// Register a token (for revocation tracking)
    pub fn register_token(&self, token_id: String, claims: Claims) {
        self.tokens.write().insert(token_id, claims);
    }

    /// Revoke a token
    pub fn revoke_token(&self, token_id: &str) {
        self.tokens.write().remove(token_id);
    }

    /// Check if token is revoked
    pub fn is_revoked(&self, token_id: &str) -> bool {
        !self.tokens.read().contains_key(token_id)
    }

    /// Check if claims have permission
    pub fn has_permission(&self, claims: &Claims, permission: &str) -> bool {
        claims.permissions.contains(&permission.to_string())
        || claims.permissions.contains(&"*".to_string()) // wildcard
    }

    /// Check if claims have role
    pub fn has_role(&self, claims: &Claims, role: &str) -> bool {
        claims.role == role
    }
}

impl Default for AuthManager {
    fn default() -> Self {
        Self::new("default-secret-key-change-in-production".to_string())
    }
}

/// Role-based access control
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    Admin,
    Dispatcher,
    Worker,
    Client,
}

impl Role {
    pub fn to_string(&self) -> String {
        match self {
            Role::Admin => "admin".to_string(),
            Role::Dispatcher => "dispatcher".to_string(),
            Role::Worker => "worker".to_string(),
            Role::Client => "client".to_string(),
        }
    }

    pub fn default_permissions(&self) -> Vec<String> {
        match self {
            Role::Admin => vec![
                "create_task".to_string(),
                "cancel_task".to_string(),
                "manage_workers".to_string(),
                "view_logs".to_string(),
                "manage_users".to_string(),
                "system_config".to_string(),
                "*".to_string(),
            ],
            Role::Dispatcher => vec![
                "create_task".to_string(),
                "assign_task".to_string(),
                "view_tasks".to_string(),
                "manage_workers".to_string(),
                "view_logs".to_string(),
            ],
            Role::Worker => vec![
                "execute_task".to_string(),
                "report_progress".to_string(),
                "view_own_tasks".to_string(),
            ],
            Role::Client => vec![
                "create_task".to_string(),
                "view_own_tasks".to_string(),
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_generation_and_verification() {
        let manager = AuthManager::new("test-secret".to_string());
        let claims = Claims::new(
            "worker-1".to_string(),
            "worker".to_string(),
            vec!["execute_task".to_string()],
        );
        
        let token = manager.generate_token(&claims).unwrap();
        let verified = manager.verify_token(&token).unwrap();
        
        assert_eq!(verified.sub, "worker-1");
        assert_eq!(verified.role, "worker");
    }

    #[test]
    fn test_role_permissions() {
        let admin_perms = Role::Admin.default_permissions();
        assert!(admin_perms.contains(&"*".to_string()));
        
        let worker_perms = Role::Worker.default_permissions();
        assert!(worker_perms.contains(&"execute_task".to_string()));
        assert!(!worker_perms.contains(&"manage_users".to_string()));
    }

    #[test]
    fn test_permission_check() {
        let manager = AuthManager::new("test-secret".to_string());
        let claims = Claims::new(
            "user-1".to_string(),
            "admin".to_string(),
            Role::Admin.default_permissions(),
        );
        
        assert!(manager.has_permission(&claims, "manage_users"));
        assert!(manager.has_permission(&claims, "any_permission")); // admin has *
    }
}
