from enum import Enum

from dislord.discord.base import BaseModel
from dislord.discord.resources.guild.guild_member import PartialGuildMember
from dislord.discord.resources.user.user import User
from dislord.discord.type import Missing, Snowflake


class InteractionType(Enum):
    PING = 1
    APPLICATION_COMMAND = 2
    MESSAGE_COMPONENT = 3
    APPLICATION_COMMAND_AUTOCOMPLETE = 4
    MODAL_SUBMIT = 5


class MessageInteraction(BaseModel):
    id: Snowflake
    type: InteractionType
    name: str
    user: User
    member: PartialGuildMember | Missing
