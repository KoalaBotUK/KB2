from enum import IntEnum

from dislord.discord.base import BaseModel
from dislord.discord.type import Snowflake


class PrivacyLevel(IntEnum):
    PUBLIC = 1
    GUILD_ONLY = 2


class StageInstance(BaseModel):
    id: Snowflake
    guild_id: Snowflake
    channel_id: Snowflake
    topic: str
    privacy_level: PrivacyLevel
