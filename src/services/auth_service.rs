use argon2::password_hash::{PasswordHasher, PasswordVerifier};
use argon2::password_hash::phc::PasswordHash;
use argon2::Argon2;
use uuid::Uuid;

use crate::error::AppError;
use crate::models::user::User;
use crate::repository;

fn pepper() -> String {
    crate::db::pepper()
}

/// Business logic for authentication, shared by HTML and JSON handlers.
pub struct AuthService {
    db: sqlx::PgPool,
}

impl AuthService {
    pub fn new(db: sqlx::PgPool) -> Self {
        Self { db }
    }

    pub async fn register(&self, username: &str, password: &str) -> Result<User, AppError> {
        if username.is_empty() || password.is_empty() {
            return Err(AppError::BadRequest("username and password are required".into()));
        }

        if repository::user_repo::find_by_username(&self.db, username).await?.is_some() {
            return Err(AppError::BadRequest("username already registered".into()));
        }

        let peppered = format!("{}{}", pepper(), password);
        let pw_hash = Argon2::default()
            .hash_password(peppered.as_bytes())
            .map_err(|_| AppError::Internal("failed to hash password".into()))?
            .to_string();

        repository::user_repo::create(&self.db, username, &pw_hash).await
    }

    pub async fn login(&self, username: &str, password: &str) -> Result<User, AppError> {
        let user = repository::user_repo::find_by_username(&self.db, username)
            .await?
            .ok_or(AppError::Unauthorized("invalid username or password".into()))?;

        let parsed = PasswordHash::new(&user.password_hash)
            .map_err(|_| AppError::Internal("auth error".into()))?;

        let peppered = format!("{}{}", pepper(), password);
        Argon2::default()
            .verify_password(peppered.as_bytes(), &parsed)
            .map_err(|_| AppError::Unauthorized("invalid username or password".into()))?;

        Ok(user)
    }

    pub async fn get_user(&self, uid: &Uuid) -> Result<User, AppError> {
        repository::user_repo::find_by_id(&self.db, uid)
            .await?
            .ok_or(AppError::NotFound("user not found".into()))
    }

    pub async fn change_password(&self, uid: &Uuid, current_password: &str, new_password: &str) -> Result<(), AppError> {
        let user = self.get_user(uid).await?;

        let parsed = PasswordHash::new(&user.password_hash)
            .map_err(|_| AppError::Internal("auth error".into()))?;

        let peppered = format!("{}{}", pepper(), current_password);
        Argon2::default()
            .verify_password(peppered.as_bytes(), &parsed)
            .map_err(|_| AppError::Unauthorized("invalid password".into()))?;

        let new_peppered = format!("{}{}", pepper(), new_password);
        let new_hash = Argon2::default()
            .hash_password(new_peppered.as_bytes())
            .map_err(|_| AppError::Internal("failed to hash password".into()))?
            .to_string();

        repository::user_repo::update_password(&self.db, uid, &new_hash).await
    }
}
