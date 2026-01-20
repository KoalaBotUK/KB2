use http::StatusCode;
use twilight_model::channel::message::component::{ActionRow, Button, ButtonStyle};
use twilight_model::channel::message::{Component, EmojiReactionType};

pub trait VoteOptionComponent {
    fn emoji(&self) -> &Option<EmojiReactionType>;
    fn label(&self) -> &Option<String>;
    fn custom_id(&self) -> String {
        format!(
            "vt{}{}",
            self.label().clone().unwrap_or_default().replace(" ", ""),
            self.emoji()
                .clone()
                .map(|e| {
                    match e {
                        EmojiReactionType::Unicode { name } => name,
                        EmojiReactionType::Custom { id, .. } => id.to_string(),
                    }
                })
                .unwrap_or_default()
        )
    }

    fn to_component(&self) -> Component {
        Component::Button(Button {
            custom_id: Some(self.custom_id()),
            disabled: false,
            emoji: self.emoji().clone(),
            label: self.label().clone(),
            style: ButtonStyle::Secondary,
            url: None,
            sku_id: None,
        })
    }
}

pub fn group_to_rows(components: Vec<Component>) -> Result<Vec<Component>, StatusCode> {
    let rows: Vec<Component> = components
        .chunks(5)
        .map(|chunk| {
            Component::ActionRow(ActionRow {
                components: Vec::from(chunk),
            })
        })
        .collect();
    if rows.len() > 5 {
        return Err(StatusCode::BAD_REQUEST);
    }
    Ok(rows)
}
