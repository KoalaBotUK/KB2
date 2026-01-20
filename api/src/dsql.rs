use aws_config::{BehaviorVersion, Region, SdkConfig};
use aws_sdk_dsql::auth_token::{AuthToken, AuthTokenGenerator, Config};
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use sqlx::{Pool, Postgres};
use std::time::Duration;
use sqlx::query::Query;
use tokio::time;

const SECONDS_PER_MINUTE: u64 = 60;

const TOKEN_EXPIRATION_MINUTES: u64 = 15;
const TOKEN_EXPIRATION_SECONDS: u64 = TOKEN_EXPIRATION_MINUTES * SECONDS_PER_MINUTE;

const TOKEN_REFRESH_MINUTES: u64 = TOKEN_EXPIRATION_MINUTES - 5;
const TOKEN_REFRESH_SECONDS: u64 = TOKEN_REFRESH_MINUTES * SECONDS_PER_MINUTE;

const _: () = assert!(
    TOKEN_EXPIRATION_MINUTES > TOKEN_REFRESH_MINUTES,
    "Token expiration time must be greater than refresh time"
);

async fn generate_password_token(
    cluster_user: &str,
    signer: &AuthTokenGenerator,
    sdk_config: &SdkConfig,
) -> AuthToken {
    if cluster_user == "admin" {
        signer
            .db_connect_admin_auth_token(sdk_config)
            .await
            .unwrap()
    } else {
        signer.db_connect_auth_token(sdk_config).await.unwrap()
    }
}

/// Established a pooled connection with periodic credential refresh.
pub async fn establish_connection(
    cluster_user: String,
    cluster_endpoint: String,
    region: &Region,
) -> anyhow::Result<Pool<Postgres>> {
    let sdk_config = aws_config::load_defaults(BehaviorVersion::latest()).await;
    let signer = AuthTokenGenerator::new(
        Config::builder()
            .hostname(&cluster_endpoint)
            .region(region.clone())
            .expires_in(TOKEN_EXPIRATION_SECONDS)
            .build()
            .unwrap(),
    );

    let password_token = generate_password_token(&cluster_user, &signer, &sdk_config).await;
    let schema = match cluster_user.as_str() {
        "admin" => "public",
        _ => "myschema",
    };

    let connection_options = PgConnectOptions::new()
        .host(&cluster_endpoint)
        .port(5432)
        .database("postgres")
        .username(&cluster_user)
        .password(password_token.as_str())
        .ssl_mode(sqlx::postgres::PgSslMode::VerifyFull);

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .after_connect(move |conn, _meta| {
            Box::pin(async move {
                sqlx::query(format!("SET search_path = {schema}").as_str())
                    .execute(conn)
                    .await
                    .map(|_| ())
            })
        })
        .connect_with(connection_options.clone())
        .await?;

    // Periodically refresh the password by regenerating the token. This runs every
    // TOKEN_REFRESH_MINUTES and provides a token valid for TOKEN_EXPIRATION_MINUTES.
    let _pool = pool.clone(); // Pool uses an Arc internally
    tokio::spawn(async move {
        loop {
            time::sleep(Duration::from_secs(TOKEN_REFRESH_SECONDS)).await;

            let password_token = generate_password_token(&cluster_user, &signer, &sdk_config).await;
            let connect_options_with_new_token =
                connection_options.clone().password(password_token.as_str());
            _pool.set_connect_options(connect_options_with_new_token);
        }
    });

    Ok(pool)
}

pub fn in_params<T>(vec: &[T]) -> String {
    (1..=vec.len())
        .map(|i| format!("${i}"))
        .collect::<Vec<String>>()
        .join(",")
}

pub fn bind_in_params<'q, T>(
    mut query: Query<'q, Postgres, sqlx::postgres::PgArguments>,
    vec: &'q [T],
) -> Query<'q, Postgres, sqlx::postgres::PgArguments>
where
    T: sqlx::Encode<'q, Postgres> + sqlx::Type<Postgres> + Send + 'q,
{
    for v in vec {
        query = query.bind(v);
    }
    query
}