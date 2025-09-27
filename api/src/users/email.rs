use aws_sdk_sesv2::Client;
use aws_sdk_sesv2::types::{Destination, EmailContent, Template};
use http::StatusCode;
use serde::Serialize;
use crate::discord::ise;

#[derive(Serialize)]
struct TemplateData {
    name: String,
    link_url: String
}

pub async fn send_verify_email(
    client: &Client,
    global_name: &str,
    email_addr: String,
    token: &str,
) -> Result<(), StatusCode> {
    let env = std::env::var("DEPLOYMENT_ENV").expect("DEPLOYMENT_ENV must be set");
    let mut host = "koalabot.uk".to_string();
    if env != "prod" {
        host = format!("{}.{}", env, host);
    }

    let t_data = serde_json::to_string(&TemplateData {
        name: global_name.to_string(),
        link_url: format!("https://{host}/verify/email/callback?token={token}").to_string(),
    }).map_err(ise)?;
    
    let email_content = EmailContent::builder()
        .template(
            Template::builder()
                .template_name(format!("kb2-verify-template-{env}"))
                .template_data(t_data)
                .build(),
        )
        .build();

    client
        .send_email()
        .from_email_address(format!("no-reply@{host}"))
        .destination(Destination::builder().to_addresses(email_addr).build())
        .content(email_content)
        .send()
        .await
        .map_err(ise)?;

    Ok(())
}

