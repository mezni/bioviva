use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UserRole {
    Consumer,
    Producer,
    Admin,
}

#[derive(Debug, Clone)]
pub struct User {
    pub id: uuid::Uuid,
    pub email: String,
    pub password_hash: String,
    pub role: UserRole,
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
}

#[derive(Debug, Clone)]
pub struct NewUser {
    pub email: String,
    pub plain_password: String,
    pub role: UserRole,
}

impl NewUser {
    pub fn new(email: String, plain_password: String, role: UserRole) -> Self {
        Self {
            email,
            plain_password,
            role,
        }
    }
}