from dislord.discord.base import BaseModel
from dislord.discord.resources.auto_moderation.enums import TriggerType, EventType, KeywordPresetType, ActionType
from dislord.discord.type import Snowflake, Missing

class ActionMetadata(BaseModel):
    channel_id: Snowflake
    duration_seconds: int
    custom_message: str | Missing


class AutoModerationAction(BaseModel):
    type: ActionType
    metadata: ActionMetadata | Missing


class TriggerMetadata(BaseModel):
    keyword_filter: list[str]
    regex_patterns: list[str]
    presets: list[KeywordPresetType]
    allow_list: list[str]
    mention_total_limit: int
    mention_raid_protection_enabled: bool


class AutoModerationRule(BaseModel):
    id: Snowflake
    guild_id: Snowflake
    name: str
    creator_id: Snowflake
    event_type: EventType
    trigger_type: TriggerType
    trigger_metadata: TriggerMetadata
    actions: list[AutoModerationAction]
    enabled: bool
    exempt_roles: list[Snowflake]
    exempt_channels: list[Snowflake]