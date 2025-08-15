mod models;

use crate::users::models::User;
use crate::AppState;
use aws_sdk_dynamodb::types::AttributeValue;
use axum::extract::{Path, State};
use axum::routing::get;
use axum::{Extension, Json};
use http::StatusCode;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use lambda_http::tracing::error;
use twilight_http::Client as DiscordClient;
use twilight_model::id::marker::UserMarker;
use twilight_model::id::Id;

pub fn router() -> axum::Router<AppState> {
    axum::Router::new()
        .route("/", get(get_users))
        .route("/{user_id}", get(get_users_id).put(put_users_id))
}

async fn get_users(
) -> Json<Value> {
    todo!()
}

async fn get_users_id(
    Path(user_id): Path<Id<UserMarker>>,
    Extension(discord_user): Extension<Arc<DiscordClient>>,
    State(app_state): State<AppState>,
) -> Result<Json<Value>, StatusCode> {
    let logged_in_user = discord_user.current_user().await.unwrap().model().await.unwrap();
    if logged_in_user.id.ne(&user_id) {
        return Err(StatusCode::NOT_FOUND)
    }

    let result = app_state.dynamo.query().table_name(format!("kb2_users_{}",std::env::var("DEPLOYMENT_ENV").expect("DEPLOYMENT_ENV must be set"),))
        .key_condition_expression("#uid = :uid")
        .expression_attribute_names("#uid", "user_id")
        .expression_attribute_values(":uid", AttributeValue::S(user_id.to_string()))
        .send()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
        .and_then(|resp| {
            let items = resp.items.unwrap_or_default();
            if items.is_empty() {
                return Err(StatusCode::NOT_FOUND);
            }
            let user: User = (&items[0]).into();
            Ok(user)
        })?;

    Ok(Json(json!(result)))
}

async fn put_users_id(
    Path(user_id): Path<Id<UserMarker>>,
    Extension(discord_user): Extension<Arc<DiscordClient>>,
    State(app_state): State<AppState>,
    Json(user): Json<User>
) -> Result<Json<Value>, StatusCode> {
    let logged_in_user = discord_user.current_user().await.unwrap().model().await.unwrap();
    if logged_in_user.id.ne(&user_id) {
        return Err(StatusCode::NOT_FOUND)
    }
    // Write user to DynamoDB
    let user_id_str = user_id.to_string();
    let mut item = HashMap::new();
    item.insert("user_id".to_string(), AttributeValue::S(user_id_str));
    item.insert(
        "links".to_string(),
        AttributeValue::L(
            user.links
                .clone()
                .into_iter()
                .map(|l| {
                    let mut map = HashMap::new();
                    map.insert("link_address".to_string(), AttributeValue::S(l.link_address.clone()));
                    map.insert("linked_at".to_string(), AttributeValue::N(l.linked_at.clone().to_string()));
                    AttributeValue::M(map)
                })
                .collect(),
        ),
    );
    item.insert(
        "linked_guild_ids".to_string(),
        AttributeValue::L(
            user.linked_guild_ids
                .into_iter()
                .map(|id| AttributeValue::S(id))
                .collect(),
        ),
    );

    let resp = app_state.dynamo
        .put_item()
        .table_name(format!("kb2_users_{}",std::env::var("DEPLOYMENT_ENV").expect("DEPLOYMENT_ENV must be set"),))
        .set_item(Some(item))
        .send()
        .await;

    match resp {
        Ok(_) => get_users_id(
            Path(user_id.into()),
            Extension(discord_user),
            State(app_state),
        ).await,
        Err(e) => {
            error!("Dynamodb write error: {}", e.into_service_error());
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        },
    }
}