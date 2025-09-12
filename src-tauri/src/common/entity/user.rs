use sqlx::prelude::FromRow;
use uuid::Uuid;

#[derive(FromRow)]
pub struct UserRow {
    pub created_at: i64,
    pub updated_at: i64,
    pub id: Uuid,
}

#[derive(FromRow)]
pub struct UserAgentRow {
    pub created_at: i64,
    pub updated_at: i64,
    pub user_id: Uuid,
    pub agent_id: Uuid,
}
