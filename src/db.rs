use sqlx::{migrate::MigrateDatabase, Sqlite, SqlitePool};
use tracing::info;

/// Sets up the database: creates it if it doesn't exist and runs migrations.
pub async fn setup_database(db_url: &str) -> eyre::Result<SqlitePool> {
    if !Sqlite::database_exists(db_url).await.unwrap_or(false) {
        info!("Creating database {}", db_url);
        Sqlite::create_database(db_url).await?;
    }

    let pool = SqlitePool::connect(db_url).await?;
    sqlx::migrate!("./migrations").run(&pool).await?;

    Ok(pool)
}

/// Logs a detected arbitrage opportunity to the database.
pub async fn log_opportunity(
    pool: &SqlitePool,
    buy_dex: &str,
    sell_dex: &str,
    token_in: &str,
    token_out: &str,
    amount_in: f64,
    amount_out: f64,
    simulated_profit_usd: f64,
) -> eyre::Result<()> {
    sqlx::query!(
        r#"
        INSERT INTO arbitrage_opportunities (
            buy_dex, sell_dex, token_in, token_out, amount_in, amount_out, simulated_profit_usd
        ) VALUES ( ?1, ?2, ?3, ?4, ?5, ?6, ?7 )
        "#,
        buy_dex,
        sell_dex,
        token_in,
        token_out,
        amount_in,
        amount_out,
        simulated_profit_usd
    )
    .execute(pool)
    .await?;

    Ok(())
}
