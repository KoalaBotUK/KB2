from enum import IntFlag

from dislord.discord.base import BaseModel
from dislord.discord.type import Snowflake, Missing


class RoleTags(BaseModel):
    bot_id: Snowflake | Missing
    integration_id: Snowflake | Missing
    premium_subscriber: Missing | None
    subscription_listing_id: Snowflake | Missing
    available_for_purchase: Missing | None
    guild_connections: Missing | None


class RoleFlags(IntFlag):
    IN_PROMPT = 1 << 0


class Role(BaseModel):
    id: Snowflake
    name: str
    color: int
    hoist: bool
    icon: str | Missing | None
    unicode_emoji: str | Missing | None
    position: int
    permissions: str
    managed: bool
    mentionable: bool
    tags: RoleTags | Missing
    flags: RoleFlags
