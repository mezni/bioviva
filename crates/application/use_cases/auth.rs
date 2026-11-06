use bioivara_domain::{User, DomainError, NewUser};
use bioivara_application::ports::UserRepositoryTrait;
use argon2::Argon2;

pub struct RegisterUser {
    pub email: String,
    pub plain_password: String,
}

pub struct RegisteredUser {
    pub user: User,
    pub jwt_token: String,
}

pub async fn handle_register(
    repo: impl UserRepositoryTrait,
    payload: RegisterUser,
) -> Result<RegisteredUser, DomainError> {
    // Check if user already exists
    let existing = repo.find_by_email(&payload.email).await?;
    if existing.is_some() {
        return Err(DomainError::InvalidCredentials);
    }

    // Create new user with Argon2 hash
    let new_user = NewUser::new(payload.email.clone(), payload.plain_password, bioivara_domain::UserRole::Consumer);

    let user = repo.create(&new_user).await?;

    // Generate JWT token (delegated to infrastructure)
    let jwt_token = "placeholder-jwt-token".to_string();

    Ok(RegisteredUser { user, jwt_token })
}