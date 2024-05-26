from dislord.discord.base import BaseModel
from dislord.discord.interactions.application_commands.models import ApplicationCommand
from dislord.discord.resources.audit_log.enums import AuditLogEvent
from dislord.discord.resources.auto_moderation.models import AutoModerationRule
from dislord.discord.resources.channel.channel import Channel
from dislord.discord.type import Snowflake, Missing


class AuditLogChange(BaseModel):
    new_value: str | int | float | Missing
    old_value: str | int | float | Missing
    key: str


class OptionalAuditEntryInfo(BaseModel):
    application_id: Snowflake
    auto_moderation_rule_name: str
    auto_moderation_rule_trigger_type: str
    channel_id: Snowflake
    count: str
    delete_member_days: str
    id: Snowflake
    members_removed: str
    message_id: Snowflake
    role_name: str
    type: str
    integration_type: str


class AuditLogEntry(BaseModel):
    target_id: str | None
    changes: list[AuditLogChange] | Missing
    user_id: Snowflake | None
    id: Snowflake
    action_type: AuditLogEvent
    options: OptionalAuditEntryInfo | Missing
    reason: str | Missing


class AuditLog(BaseModel):
    application_commands: list[ApplicationCommand]
    audit_log_entries: list[AuditLogEntry]
    auto_moderation_rules: list[AutoModerationRule]
    # guild_scheduled_events: list[GuildScheduledEvent] FIXME
    # integrations: list[PartialIntegration] FIXME
    threads: list[Channel]
    # users: list[User] FIXME
    # webhooks: list[Webhook] FIXME
