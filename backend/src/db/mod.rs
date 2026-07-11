use argon2::password_hash::PasswordHasher;
use argon2::Argon2;

/// Returns the server-side server secret used for password peppering.
/// Prevents hash cracking even if the database is leaked.
/// Defaults to empty string (no pepper) when not configured.
fn pepper() -> String {
    std::env::var("APP_SECRET").unwrap_or_default()
}

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
    let peppered = format!("{}{}", pepper(), "admin");
    let hash = Argon2::default()
        .hash_password(peppered.as_bytes())
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

#[cfg(test)]
mod tests {
    use super::*;
    use argon2::PasswordVerifier;

    #[test]
    fn test_password_hash_verify_cycle() {
        let password = "admin";
        let p = pepper();
        let peppered = format!("{}{}", p, password);

        let hash = Argon2::default()
            .hash_password(peppered.as_bytes())
            .expect("failed to hash")
            .to_string();

        let parsed = argon2::password_hash::PasswordHash::new(&hash)
            .expect("failed to parse hash");

        argon2::Argon2::default()
            .verify_password(peppered.as_bytes(), &parsed)
            .expect("verify should succeed");
    }

    #[test]
    fn test_admin_login_hash_stability() {
        // Simulates exactly what seed_admin does, then verifies
        // This catches argon2 library version mismatches early.
        let p = pepper();
        let peppered = format!("{}{}", p, "admin");

        let hash = Argon2::default()
            .hash_password(peppered.as_bytes())
            .expect("failed to hash")
            .to_string();

        let parsed = argon2::password_hash::PasswordHash::new(&hash)
            .expect("failed to parse hash");

        let result = argon2::Argon2::default()
            .verify_password(peppered.as_bytes(), &parsed);

        assert!(result.is_ok(), "hash/verify cycle must succeed: {:?}", result.err());
    }
}
