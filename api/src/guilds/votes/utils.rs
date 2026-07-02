use http::StatusCode;
use twilight_model::channel::message::component::{ActionRow, Button, ButtonStyle};
use twilight_model::channel::message::{Component, EmojiReactionType};
use twilight_model::id::Id;
use twilight_model::id::marker::ChannelMarker;

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

/// Returns whether `channel_id` is one of the channels belonging to a guild.
///
/// `guild_channel_ids` should be the IDs of the channels fetched for the
/// guild in question (e.g. via `discord::get_guild_channels`). This is used
/// to guard against a caller-supplied `channel_id` that belongs to a
/// different guild than the one the caller was authorized against.
pub fn channel_belongs_to_guild(
    channel_id: Id<ChannelMarker>,
    guild_channel_ids: &[Id<ChannelMarker>],
) -> bool {
    guild_channel_ids.contains(&channel_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn channel_belongs_to_guild_accepts_channel_in_guild() {
        let target = Id::new(123);
        let guild_channel_ids = vec![Id::new(111), Id::new(123), Id::new(222)];

        assert!(channel_belongs_to_guild(target, &guild_channel_ids));
    }

    #[test]
    fn channel_belongs_to_guild_rejects_channel_from_another_guild() {
        let foreign_channel = Id::new(999);
        let guild_channel_ids = vec![Id::new(111), Id::new(123), Id::new(222)];

        assert!(!channel_belongs_to_guild(foreign_channel, &guild_channel_ids));
    }

    #[test]
    fn channel_belongs_to_guild_rejects_when_guild_has_no_channels() {
        let target = Id::new(123);
        let guild_channel_ids: Vec<Id<ChannelMarker>> = vec![];

        assert!(!channel_belongs_to_guild(target, &guild_channel_ids));
    }
}
