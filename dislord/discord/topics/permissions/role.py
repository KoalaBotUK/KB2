from enum import IntFlag

from dislord.types import ObjDict
from dislord.discord.reference import Snowflake, Missing


class RoleTags(ObjDict):
    bot_id: Snowflake | Missing = None
    integration_id: Snowflake | Missing = None
    premium_subscriber: Missing | None = None
    subscription_listing_id: Snowflake | Missing = None
    available_for_purchase: Missing | None = None
    guild_connections: Missing | None = None


class RoleFlags(IntFlag):
    NONE = 0
    IN_PROMPT = 1 << 0


class Role(ObjDict):
    id: Snowflake
    name: str
    color: int
    hoist: bool
    icon: str | Missing | None = None
    unicode_emoji: str | Missing | None = None
    position: int
    permissions: str
    managed: bool
    mentionable: bool
    tags: RoleTags | Missing = None
    flags: RoleFlags = RoleFlags.NONE
