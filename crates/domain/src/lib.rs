use uuid::Uuid;
use chrono::NaiveDate;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UserRole {
    Consumer,
    Producer,
    Admin,
}

#[derive(Debug, Clone)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub password_hash: String,
    pub role: UserRole,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Error)]
pub enum DomainError {
    #[error("Invalid email format")]
    InvalidEmailFormat,
    #[error("Password too weak")]
    PasswordTooWeak,
    #[error("User not found")]
    UserNotFound,
    #[error("Invalid credentials")]
    InvalidCredentials,
    #[error("Insufficient stock")]
    InsufficientStock,
    #[error("Domain error: {0}")]
    StringError(String),
}

impl std::fmt::Display for DomainError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DomainError::InvalidEmailFormat => write!(f, "Invalid email format"),
            DomainError::PasswordTooWeak => write!(f, "Password too weak"),
            DomainError::UserNotFound => write!(f, "User not found"),
            DomainError::InvalidCredentials => write!(f, "Invalid credentials"),
            DomainError::InsufficientStock => write!(f, "Insufficient stock"),
            DomainError::StringError(s) => write!(f, "{}", s),
        }
    }
}

impl std::error::Error for DomainError {}

#[derive(Debug, Clone)]
pub struct NewUser {
    pub id: Uuid,
    pub email: String,
    pub plain_password: String,
    pub role: UserRole,
}

impl NewUser {
    pub fn new(email: String, plain_password: String, role: UserRole) -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            email,
            plain_password,
            role,
        }
    }
}