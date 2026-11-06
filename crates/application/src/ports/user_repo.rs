use bioivara_domain::{User, DomainError};
use uuid::Uuid;

pub trait UserRepositoryTrait: Send + Sync + 'static {
    async fn find_by_email(&self, email: &str) -> Result<Option<User>, DomainError>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, DomainError>;
    async fn create(&self, new_user: &bioivara_domain::NewUser) -> Result<User, DomainError>;
}