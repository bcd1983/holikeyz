use anyhow::Result;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc, Duration};

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityToken {
    pub session_id: String,
    pub token: String,
    pub expires_at: DateTime<Utc>,
    pub device_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedPayload {
    pub data: String,
    pub signature: String,
    pub timestamp: DateTime<Utc>,
}

pub struct SecurityManager {
    secret_key: Vec<u8>,
}

impl SecurityManager {
    pub fn new() -> Self {
        let secret_key = Self::generate_secret_key();
        Self { secret_key }
    }

    pub fn with_key(secret_key: Vec<u8>) -> Self {
        Self { secret_key }
    }

    fn generate_secret_key() -> Vec<u8> {
        let uuid = Uuid::new_v4();
        uuid.as_bytes().to_vec()
    }

    pub fn create_session_token(&self, session_id: &str, device_id: Option<String>) -> Result<SecurityToken> {
        let token_data = format!("{}:{}", session_id, Uuid::new_v4());
        let token = self.sign_data(token_data.as_bytes())?;
        
        Ok(SecurityToken {
            session_id: session_id.to_string(),
            token,
            expires_at: Utc::now() + Duration::hours(1),
            device_id,
        })
    }

    pub fn verify_session_token(&self, token: &SecurityToken) -> Result<bool> {
        if Utc::now() > token.expires_at {
            return Ok(false);
        }
        
        let token_data = format!("{}:", token.session_id);
        Ok(self.verify_signature(token_data.as_bytes(), &token.token)?)
    }

    pub fn encrypt_credentials(&self, credentials: &str) -> Result<EncryptedPayload> {
        let timestamp = Utc::now();
        let data_with_timestamp = format!("{}:{}", timestamp.timestamp(), credentials);
        
        let encrypted_data = BASE64.encode(data_with_timestamp.as_bytes());
        
        let signature = self.sign_data(encrypted_data.as_bytes())?;
        
        Ok(EncryptedPayload {
            data: encrypted_data,
            signature,
            timestamp,
        })
    }

    pub fn decrypt_credentials(&self, payload: &EncryptedPayload) -> Result<String> {
        if !self.verify_signature(payload.data.as_bytes(), &payload.signature)? {
            anyhow::bail!("Invalid signature");
        }
        
        let age = Utc::now() - payload.timestamp;
        if age > Duration::minutes(5) {
            anyhow::bail!("Payload expired");
        }
        
        let decoded = BASE64.decode(&payload.data)?;
        let data_str = String::from_utf8(decoded)?;
        
        let parts: Vec<&str> = data_str.splitn(2, ':').collect();
        if parts.len() != 2 {
            anyhow::bail!("Invalid payload format");
        }
        
        Ok(parts[1].to_string())
    }

    fn sign_data(&self, data: &[u8]) -> Result<String> {
        let mut mac = HmacSha256::new_from_slice(&self.secret_key)
            .map_err(|e| anyhow::anyhow!("HMAC error: {}", e))?;
        mac.update(data);
        let result = mac.finalize();
        Ok(BASE64.encode(result.into_bytes()))
    }

    fn verify_signature(&self, data: &[u8], signature: &str) -> Result<bool> {
        let expected_signature = self.sign_data(data)?;
        Ok(expected_signature == signature)
    }

    pub fn generate_device_challenge(&self) -> String {
        let challenge = Uuid::new_v4().to_string();
        BASE64.encode(challenge.as_bytes())
    }

    pub fn verify_device_response(&self, challenge: &str, response: &str, device_id: &str) -> Result<bool> {
        let expected_response = self.calculate_device_response(challenge, device_id)?;
        Ok(expected_response == response)
    }

    fn calculate_device_response(&self, challenge: &str, device_id: &str) -> Result<String> {
        let data = format!("{}:{}", challenge, device_id);
        self.sign_data(data.as_bytes())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvisioningCertificate {
    pub device_id: String,
    pub public_key: String,
    pub issued_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub signature: String,
}

impl SecurityManager {
    pub fn issue_provisioning_certificate(&self, device_id: &str) -> Result<ProvisioningCertificate> {
        let public_key = BASE64.encode(Uuid::new_v4().as_bytes());
        let issued_at = Utc::now();
        let expires_at = issued_at + Duration::days(365);
        
        let cert_data = format!("{}:{}:{}:{}", 
            device_id, public_key, issued_at.timestamp(), expires_at.timestamp());
        let signature = self.sign_data(cert_data.as_bytes())?;
        
        Ok(ProvisioningCertificate {
            device_id: device_id.to_string(),
            public_key,
            issued_at,
            expires_at,
            signature,
        })
    }

    pub fn verify_provisioning_certificate(&self, cert: &ProvisioningCertificate) -> Result<bool> {
        if Utc::now() > cert.expires_at {
            return Ok(false);
        }
        
        let cert_data = format!("{}:{}:{}:{}", 
            cert.device_id, cert.public_key, cert.issued_at.timestamp(), cert.expires_at.timestamp());
        
        self.verify_signature(cert_data.as_bytes(), &cert.signature)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_token() {
        let manager = SecurityManager::new();
        let token = manager.create_session_token("test-session", Some("device-123".to_string())).unwrap();
        
        assert!(manager.verify_session_token(&token).unwrap());
        assert_eq!(token.session_id, "test-session");
        assert_eq!(token.device_id, Some("device-123".to_string()));
    }

    #[test]
    fn test_credential_encryption() {
        let manager = SecurityManager::new();
        let credentials = "ssid:password123";
        
        let encrypted = manager.encrypt_credentials(credentials).unwrap();
        let decrypted = manager.decrypt_credentials(&encrypted).unwrap();
        
        assert_eq!(credentials, decrypted);
    }

    #[test]
    fn test_device_challenge() {
        let manager = SecurityManager::new();
        let challenge = manager.generate_device_challenge();
        let device_id = "device-456";
        
        let response = manager.calculate_device_response(&challenge, device_id).unwrap();
        assert!(manager.verify_device_response(&challenge, &response, device_id).unwrap());
    }

    #[test]
    fn test_provisioning_certificate() {
        let manager = SecurityManager::new();
        let cert = manager.issue_provisioning_certificate("device-789").unwrap();
        
        assert!(manager.verify_provisioning_certificate(&cert).unwrap());
        assert_eq!(cert.device_id, "device-789");
    }
}