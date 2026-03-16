use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub tenant_id: String,
    pub exp: usize,
}

pub struct AuthService {
    decoding_key: DecodingKey,
}

impl AuthService {
    pub fn new(secret: &[u8]) -> Self {
        Self {
            decoding_key: DecodingKey::from_secret(secret),
        }
    }

    pub fn verify_token(&self, token: &str) -> Result<Claims, String> {
        let validation = Validation::new(Algorithm::HS256);
        match decode::<Claims>(token, &self.decoding_key, &validation) {
            Ok(token_data) => Ok(token_data.claims),
            Err(e) => Err(format!("Invalid token: {}", e)),
        }
    }
}
