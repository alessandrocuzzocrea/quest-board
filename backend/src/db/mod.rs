use argon2::password_hash::PasswordHasher;
use argon2::Argon2;

pub async fn run_migrations(pool: &sqlx::PgPool) -> Result<(), sqlx::Error> {
    let sql = include_str!("../../migrations/001_initial.sql");
    for statement in sql.split(';') {
        let stmt = statement.trim();
        if !stmt.is_empty() {
            sqlx::query(stmt).execute(pool).await?;
        }
    }

    seed_admin(pool).await?;

    Ok(())
}

async fn seed_admin(pool: &sqlx::PgPool) -> Result<(), sqlx::Error> {
    let exists: Option<(String,)> =
        sqlx::query_as("SELECT id FROM users WHERE email = $1")
            .bind("admin")
            .fetch_optional(pool)
            .await?;

    if exists.is_some() {
        return Ok(());
    }

    let id = uuid::Uuid::new_v4().to_string();
    let hash = Argon2::default()
        .hash_password(b"admin")
        .expect("failed to hash admin password")
        .to_string();

    sqlx::query(
        "INSERT INTO users (id, email, password_hash, name, role) VALUES ($1, $2, $3, $4, 'admin')",
    )
    .bind(&id)
    .bind("admin")
    .bind(&hash)
    .bind("Admin")
    .execute(pool)
    .await?;

    tracing::info!("seeded default admin user (email: admin, password: admin)");
    Ok(())
}
