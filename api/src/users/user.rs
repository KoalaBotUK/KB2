// use aws_sdk_dynamodb::Client;
// use lambda_http::{run, service_fn, tracing, Body, Error, Request, Response};
// use serde::{Deserialize, Serialize};
// use serde_dynamo::to_item;
//
//
//
// pub struct Email {
//     pub email_address: String,
//     pub token: String,
//     pub token_expiry: String,
//     pub active: bool,
//     pub origin: String // Replace with enum
// }
//
// #[derive(Clone, Debug, Serialize, Deserialize)]
// pub struct User {
//     pub user_id: String,
//     pub emails: Vec<String>,
//     pub username: String,
//     pub first_name: String,
//     pub last_name: String,
// }
//
//
// // Add an item to a table.
// // snippet-start:[dynamodb.rust.add-item]
// pub async fn add_item(client: &Client, item: Item, table: &str) -> Result<(), Error> {
//     let item = to_item(item)?;
//
//     let request = client.put_item().table_name(table).set_item(Some(item));
//
//     tracing::info!("adding item to DynamoDB");
//
//     let _resp = request.send().await?;
//
//     Ok(())
// }