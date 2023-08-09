use sqlx::SqlitePool;
use std::env;

#[derive(Clone)]
pub struct PostDatabase {
    pool: SqlitePool,
}

impl PostDatabase {
    pub async fn create() -> Result<Self, sqlx::Error> {
        let pool = SqlitePool::connect(&env::var("DATABASE_URL").unwrap()).await?;
        Ok(Self { pool })
    }

    pub async fn get_user(&self, email: &str) -> Result<Option<(u32,)>, sqlx::Error> {
        let user: Option<(u32,)> = sqlx::query_as(
            r#"
                SELECT id
                FROM users
                WHERE email = ?;
            "#,
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await?;
        Ok(user)
    }

    pub async fn insert_user(&self, email: &str) -> Result<u32, sqlx::Error> {
        let user: (u32,) = sqlx::query_as(
            r#"
                INSERT INTO users(email)
                VALUES (?)
                RETURNING id
            "#,
        )
        .bind(email)
        .fetch_one(&self.pool)
        .await?;
        Ok(user.0)
    }

    pub async fn insert_user_challenge(
        &self,
        user_id: u32,
        challenge: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
                INSERT INTO user_credential_challenges(user_id, challenge)
                VALUES (?, ?)
            "#,
            user_id,
            challenge
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn user_challenge_exists(
        &self,
        challenge: &str,
        email: &str,
    ) -> Result<bool, sqlx::Error> {
        let challenge_count = sqlx::query_scalar!(
            r#"
                SELECT COUNT(1)
                FROM user_credential_challenges uc
                JOIN users u
                ON u.id = uc.user_id
                WHERE u.email = ?
                AND challenge = ?
                LIMIT 1
             "#,
            email,
            challenge
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(challenge_count > 0)
    }
}
