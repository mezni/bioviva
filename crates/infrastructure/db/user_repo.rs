use sqlx::PgPool;
use bioivara_domain::User;
use bioivara_application::ports::UserRepositoryTrait;
use bioivara_domain::DomainError;
use argon2::Argon2;
use uuid::Uuid;
use chrono::Utc;

pub struct PgUserRepo {
    pool: PgPool,
}

impl PgUserRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl bioivara_application::ports::UserRepositoryTrait for PgUserRepo {
    async fn find_by_email(&self, email: &str) -> Result<Option<User>, bioivara_domain::DomainError> {
        let row = sqlx::query_as!(
            User,
            "SELECT id, email, password_hash, role FROM auth.users WHERE email = $1",
            email
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|_| bioivara_domain::DomainError::DbError)?;

        Ok(row)
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, bioivara_domain::DomainError> {
        let row = sqlx::query_as!(
            User,
            "SELECT id, email, password_hash, role FROM auth.users WHERE id = $1",
            id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|_| bioivara_domain::DomainError::DbError)?;

        Ok(row)
    }

    async fn create(&self, new_user: &bioivara_domain::NewUser) -> Result<User, bioivara_domain::DomainError> {
        let argon2 = Argon2::default();
        let hash = argon2
            .hash_password(new_user.plain_password.as_bytes(), Some(&argon2::PasswordHasher::default_memory_cost()))
            .map_err(|_| bioivara_domain::DomainError::HashingError)?
            .to_owned();

        let row = sqlx::query_as!(
            User,
            "INSERT INTO auth.users (id, email, password_hash, role) VALUES ($1, $2, $3, 'consumer') RETURNING *",
            new_user.id,
            new_user.email,
            hash
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|_| bioivara_domain::DomainError::DbError)?;

        Ok(row)
    }
}