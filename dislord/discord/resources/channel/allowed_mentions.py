from enum import StrEnum

from dislord.discord.base import BaseModel
from dislord.discord.type import Snowflake


class AllowedMentionType(StrEnum):
    ROLES = "roles"
    USERS = "users"
    EVERYONE = "everyone"


class AllowedMentions(BaseModel):
    parse: list[AllowedMentionType]
    role: list[Snowflake]
    users: list[Snowflake]
    replied_user: bool
