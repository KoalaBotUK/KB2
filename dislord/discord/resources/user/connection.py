from enum import IntEnum

from dislord.discord.base import BaseModel
from dislord.discord.resources.guild.integration import Integration
from dislord.discord.type import Missing


class VisibilityType(IntEnum):
    NONE = 0
    EVERYONE = 1


class Connection(BaseModel):
    id: str
    name: str
    type: str
    revoked: bool | Missing
    integrations: list[Integration] | Missing
    verified: bool
    friend_sync: bool
    show_activity: bool
    two_way_link: bool
    visibility: VisibilityType
