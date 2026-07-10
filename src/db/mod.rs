pub async fn run_migrations(pool: &sqlx::SqlitePool) -> Result<(), sqlx::Error> {
    let sql = include_str!("../../migrations/001_initial.sql");
    for statement in sql.split(';') {
        let stmt = statement.trim();
        if !stmt.is_empty() {
            sqlx::query(stmt).execute(pool).await?;
        }
    }
    Ok(())
}
