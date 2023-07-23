use sqlx::SqlitePool;
use std::env;
use crate::blog_post::BlogPost;

pub async fn insert_post(blog_post: &BlogPost) -> Result<(), sqlx::Error> {
    let conn = SqlitePool::connect(&env::var("DATABASE_URL").unwrap()).await?;
    sqlx::query!(r#"
            INSERT INTO posts(author, body)
            VALUES (?, ?)
        "#, blog_post.name, blog_post.body).execute(&conn).await?;
    Ok(())
}
