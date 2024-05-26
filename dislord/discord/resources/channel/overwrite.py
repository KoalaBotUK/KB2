from enum import IntEnum

from dislord.discord.base import BaseModel
from dislord.discord.type import Snowflake


class OverwriteType(IntEnum):
    ROLE = 0
    MEMBER = 1


class Overwrite(BaseModel):
    id: Snowflake
    type: OverwriteType
    allow: str
    deny: str
