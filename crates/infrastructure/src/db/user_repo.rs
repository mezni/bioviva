use sqlx::PgPool;
use uuid::Uuid;
use chrono::Utc;

use bioivara_domain::User;

pub struct PgUserRepo {
    pool: PgPool,
}

impl PgUserRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}