use argon2::password_hash::PasswordHasher;
use argon2::Argon2;

pub async fn run_migrations(pool: &sqlx::PgPool) -> Result<(), sqlx::Error> {
    for sql in [
        include_str!("../../migrations/001_initial.sql"),
        include_str!("../../migrations/002_native_uuids.sql"),
    ] {
        for statement in sql.split(';') {
            let stmt = statement.trim();
            if !stmt.is_empty() {
                sqlx::query(stmt).execute(pool).await?;
            }
        }
    }

    seed_admin(pool).await?;

    Ok(())
}

async fn seed_admin(pool: &sqlx::PgPool) -> Result<(), sqlx::Error> {
    let hash = Argon2::default()
        .hash_password(b"admin")
        .expect("failed to hash admin password")
        .to_string();

    sqlx::query(
    "INSERT INTO users (email, password_hash, name, role) VALUES ($1, $2, $3, 'admin') ON CONFLICT (email) DO NOTHING",
    )
    .bind("admin")
    .bind(&hash)
    .bind("Admin")
    .execute(pool)
    .await?;

    Ok(())
}
