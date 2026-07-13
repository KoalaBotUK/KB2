//! The worker's Discord surface, abstracted behind [`DiscordRoles`] so the
//! reconciliation logic can be unit- and performance-tested against mock or
//! stub gateways, with [`TwilightRoles`] as the production implementation.
//!
//! Unlike the api crate's `retry_on_rl` helpers, these deliberately do NOT
//! sleep through rate limits in-process: a 429 is *classified and surfaced*
//! so the orchestration layer owns all waiting (short waits absorbed inline
//! under a budget, long waits via a delayed SQS continuation — where the
//! wait costs no compute at all).

use std::sync::Arc;
use std::time::Duration;

use twilight_http::api_error::ApiError;
use twilight_http::error::ErrorType;
use twilight_model::id::Id;
use twilight_model::id::marker::{GuildMarker, RoleMarker, UserMarker};

/// How a single Discord role call failed, in terms the worker's control flow
/// understands.
#[derive(Debug)]
pub enum RoleCallError {
    /// 429: the caller decides whether to absorb the wait inline or hand it
    /// to SQS as a delayed continuation.
    RateLimited { retry_after: Duration },
    /// 403/404 (member left, role deleted, hierarchy): recorded in the job's
    /// error count and skipped — parity with the legacy handlers'
    /// NOT_FOUND tolerance. Never fails the job.
    Skip,
    /// Anything else (5xx, transport, DB of the token bucket...): the whole
    /// SQS message errors and is redelivered to resume from the checkpoint.
    Fatal(anyhow::Error),
}

pub trait DiscordRoles: Send + Sync {
    fn add_role(
        &self,
        guild_id: Id<GuildMarker>,
        user_id: Id<UserMarker>,
        role_id: Id<RoleMarker>,
    ) -> impl Future<Output = Result<(), RoleCallError>> + Send;

    fn remove_role(
        &self,
        guild_id: Id<GuildMarker>,
        user_id: Id<UserMarker>,
        role_id: Id<RoleMarker>,
    ) -> impl Future<Output = Result<(), RoleCallError>> + Send;
}

pub struct TwilightRoles {
    client: Arc<twilight_http::Client>,
}

impl TwilightRoles {
    pub fn new(client: Arc<twilight_http::Client>) -> Self {
        TwilightRoles { client }
    }
}

fn classify(e: twilight_http::Error) -> RoleCallError {
    match e.kind() {
        ErrorType::Response {
            error: ApiError::Ratelimited(ratelimited),
            ..
        } => RoleCallError::RateLimited {
            retry_after: Duration::from_secs_f64(ratelimited.retry_after),
        },
        ErrorType::Response { status, .. } if status.get() == 403 || status.get() == 404 => {
            RoleCallError::Skip
        }
        _ => RoleCallError::Fatal(e.into()),
    }
}

impl DiscordRoles for TwilightRoles {
    async fn add_role(
        &self,
        guild_id: Id<GuildMarker>,
        user_id: Id<UserMarker>,
        role_id: Id<RoleMarker>,
    ) -> Result<(), RoleCallError> {
        self.client
            .add_guild_member_role(guild_id, user_id, role_id)
            .await
            .map(|_| ())
            .map_err(classify)
    }

    async fn remove_role(
        &self,
        guild_id: Id<GuildMarker>,
        user_id: Id<UserMarker>,
        role_id: Id<RoleMarker>,
    ) -> Result<(), RoleCallError> {
        self.client
            .remove_guild_member_role(guild_id, user_id, role_id)
            .await
            .map(|_| ())
            .map_err(classify)
    }
}
