use argon2::password_hash::PasswordHasher;
use argon2::Argon2;

/// Returns the server-side secret used for password peppering.
/// Must be set via `APP_SECRET` env var in production.
pub(crate) fn pepper() -> String {
    std::env::var("APP_SECRET").expect("APP_SECRET must be set. Add APP_SECRET=<random> to .env")
}

pub async fn run_migrations(pool: &sqlx::PgPool) -> Result<(), sqlx::Error> {
    for sql in [
        include_str!("../../migrations/001_initial.sql"),
        include_str!("../../migrations/002_native_uuids.sql"),
        include_str!("../../migrations/003_api_keys.sql"),
        include_str!("../../migrations/004_email_to_username.sql"),
        include_str!("../../migrations/005_card_start_date.sql"),
        include_str!("../../migrations/006_remove_user_name.sql"),
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
    let peppered = format!("{}{}", pepper(), "admin");
    let hash = Argon2::default()
        .hash_password(peppered.as_bytes())
        .expect("failed to hash admin password")
        .to_string();

    sqlx::query(
    "INSERT INTO users (username, password_hash, role) VALUES ($1, $2, 'admin') ON CONFLICT (username) DO NOTHING",
    )
    .bind("admin")
    .bind(&hash)
    .execute(pool)
    .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use argon2::PasswordVerifier;

    #[test]
    fn test_password_hash_verify_cycle() {
        dotenvy::from_filename(".env.test").ok();
        let password = "admin";
        let p = pepper();
        let peppered = format!("{}{}", p, password);

        let hash = Argon2::default()
            .hash_password(peppered.as_bytes())
            .expect("failed to hash")
            .to_string();

        let parsed = argon2::password_hash::phc::PasswordHash::new(&hash)
            .expect("failed to parse hash");

        argon2::Argon2::default()
            .verify_password(peppered.as_bytes(), &parsed)
            .expect("verify should succeed");
    }

    #[test]
    fn test_admin_login_hash_stability() {
        dotenvy::from_filename(".env.test").ok();
        // Simulates exactly what seed_admin does, then verifies
        // This catches argon2 library version mismatches early.
        let p = pepper();
        let peppered = format!("{}{}", p, "admin");

        let hash = Argon2::default()
            .hash_password(peppered.as_bytes())
            .expect("failed to hash")
            .to_string();

        let parsed = argon2::password_hash::phc::PasswordHash::new(&hash)
            .expect("failed to parse hash");

        let result = argon2::Argon2::default()
            .verify_password(peppered.as_bytes(), &parsed);

        assert!(result.is_ok(), "hash/verify cycle must succeed: {:?}", result.err());
    }
}
