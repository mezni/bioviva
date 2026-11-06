use jsonwebtoken::{EncodingKey, DecodingKey, TokenData, encode, decode, Validation};
use chrono::{Utc, Duration};
use bioivara_domain::User;

const JWT_SECRET: &[u8] = b"bioviva-jwt-secret-2026-hmac-sha256-key";

const ACCESS_TOKEN_TTL: i64 = 900;  // 15 minutes
const REFRESH_TOKEN_TTL: i64 = 604800;  // 7 days

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Claims {
    pub sub: uuid::Uuid,
    pub role: String,
    pub exp: i64,
    pub iat: i64,
}

pub struct JwtService {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
}

impl JwtService {
    pub fn new(secret: &[u8]) -> Self {
        Self {
            encoding_key: EncodingKey::from_secret(secret),
            decoding_key: DecodingKey::from_secret(secret),
        }
    }

    pub fn generate_access_token(&self, user_id: uuid::Uuid, role: &str) -> String {
        let now = Utc::now();
        let claims = Claims {
            sub: user_id,
            role: role.to_string(),
            exp: (now + Duration::minutes(15)).timestamp(),
            iat: now.timestamp(),
        };
        encode(&jsonwebtoken::Header::default(), &claims, &self.encoding_key).unwrap()
    }

    pub fn verify_token(&self, token: &str) -> Result<TokenData<Claims>, jsonwebtoken::errors::Error> {
        let mut validation = Validation::default();
        validation.validate_exp = true;
        validation.validate_nbf = true;
        decode::<Claims>(token, &self.decoding_key, &validation)
    }
}