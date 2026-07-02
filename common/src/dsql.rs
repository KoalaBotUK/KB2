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

// If a refresh attempt fails, retry after this (much shorter) interval instead of
// waiting for the next full refresh cycle, so we have a good chance of getting a
// fresh token in place before the current one expires.
const TOKEN_REFRESH_RETRY_SECONDS: u64 = 30;

const _: () = assert!(
    TOKEN_EXPIRATION_MINUTES > TOKEN_REFRESH_MINUTES,
    "Token expiration time must be greater than refresh time"
);

async fn generate_password_token(
    cluster_user: &str,
    signer: &AuthTokenGenerator,
    sdk_config: &SdkConfig,
) -> anyhow::Result<AuthToken> {
    if cluster_user == "admin" {
        signer
            .db_connect_admin_auth_token(sdk_config)
            .await
            .map_err(|e| anyhow::anyhow!("{e}"))
    } else {
        signer
            .db_connect_auth_token(sdk_config)
            .await
            .map_err(|e| anyhow::anyhow!("{e}"))
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

    let password_token = generate_password_token(&cluster_user, &signer, &sdk_config).await?;
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

            match generate_password_token(&cluster_user, &signer, &sdk_config).await {
                Ok(password_token) => {
                    let connect_options_with_new_token =
                        connection_options.clone().password(password_token.as_str());
                    _pool.set_connect_options(connect_options_with_new_token);
                }
                Err(err) => {
                    // Keep the loop alive on a transient failure (e.g. an STS/DSQL
                    // hiccup) instead of letting the detached task die, which would
                    // otherwise permanently stop token refreshes for this pool.
                    tracing::error!(
                        error = ?err,
                        "failed to refresh DSQL auth token, will retry shortly"
                    );
                    time::sleep(Duration::from_secs(TOKEN_REFRESH_RETRY_SECONDS)).await;
                }
            }
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Regression test for #51: `generate_password_token` used to `.unwrap()` the
    /// result of the AWS SDK call, so any transient failure (e.g. missing/expired
    /// credentials) would panic the caller. In `establish_connection`'s background
    /// refresh loop, that panic killed the detached `tokio::spawn` task for good,
    /// permanently stopping token refreshes until the process restarted.
    ///
    /// The DSQL auth token is generated by purely local SigV4 signing (see
    /// `aws-sdk-dsql`'s `auth_token::AuthTokenGenerator::inner`): it resolves
    /// credentials and a region from the given `SdkConfig`/`Config` and signs a URL
    /// locally, with no network I/O. That makes it possible to deterministically
    /// trigger the "failed to obtain a token" path offline, without live AWS
    /// credentials or network access, simply by giving it an `SdkConfig` with no
    /// credentials provider configured.
    ///
    /// Before the fix this would panic (via `.unwrap()`) instead of returning an
    /// `Err`; this test asserts the current, fixed behaviour.
    #[tokio::test]
    async fn generate_password_token_returns_err_instead_of_panicking_without_credentials() {
        // Deliberately omit a credentials provider so the SDK call fails
        // immediately and synchronously with no network access required.
        let sdk_config = SdkConfig::builder()
            .region(Region::new("us-east-1"))
            .build();

        let signer = AuthTokenGenerator::new(
            Config::builder()
                .hostname("example.dsql.us-east-1.on.aws")
                .region(Region::new("us-east-1"))
                .expires_in(TOKEN_EXPIRATION_SECONDS)
                .build()
                .unwrap(),
        );

        let result = generate_password_token("admin", &signer, &sdk_config).await;

        assert!(
            result.is_err(),
            "generate_password_token should return an Err (via anyhow::Result) when \
             credentials are unavailable, not panic"
        );

        // Also exercise the non-admin branch for good measure.
        let result = generate_password_token("myuser", &signer, &sdk_config).await;
        assert!(result.is_err());
    }

    /// A failed refresh must retry well before the *current* token expires, so the
    /// pool has multiple chances to obtain a fresh one. This pins down the
    /// relationship between the constants introduced by the fix for #51: the retry
    /// delay after a failure must be meaningfully shorter than both the normal
    /// refresh cadence and the token's total lifetime.
    #[test]
    fn token_refresh_retry_interval_is_shorter_than_the_normal_refresh_cycle() {
        assert!(
            TOKEN_REFRESH_RETRY_SECONDS < TOKEN_REFRESH_SECONDS,
            "a failed refresh should retry sooner than the next full refresh cycle"
        );
        assert!(
            TOKEN_REFRESH_RETRY_SECONDS < TOKEN_EXPIRATION_SECONDS,
            "a failed refresh should retry well before the current token expires"
        );
    }
}