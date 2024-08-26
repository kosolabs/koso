#[cfg(test)]
mod tests {
    use sqlx::PgPool;

    #[test_log::test(sqlx::test)]
    async fn basic_test(pool: PgPool) -> sqlx::Result<()> {
        let projects: Vec<(String,)> = sqlx::query_as("SELECT project_id FROM projects")
            .fetch_all(&pool)
            .await
            .unwrap();
        assert_eq!(projects.len(), 1);
        assert_eq!(projects.first().unwrap().0, "koso-staging");
        Ok(())
    }
}
