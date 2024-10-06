from enum import IntEnum

from dislord.discord.reference import Missing, Snowflake
from dislord.discord.resources.guild.guild_member import PartialGuildMember
from dislord.discord.resources.user.user import User
from dislord.types import ObjDict


class InteractionType(IntEnum):
    PING = 1
    APPLICATION_COMMAND = 2
    MESSAGE_COMPONENT = 3
    APPLICATION_COMMAND_AUTOCOMPLETE = 4
    MODAL_SUBMIT = 5


class MessageInteraction(ObjDict):
    id: Snowflake
    type: InteractionType
    name: str
    user: User
    member: PartialGuildMember | Missing = None
