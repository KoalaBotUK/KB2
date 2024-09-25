from typing import Self

from dislord.discord.interactions.components.enums import ComponentType
from dislord.types import ObjDict
from dislord.discord.interactions.application_commands.enums import ApplicationCommandOptionType
from dislord.discord.interactions.components.models import  Component
from dislord.discord.topics.permissions.role import Role
from dislord.discord.resources.channel.attachment import Attachment
from dislord.discord.resources.channel.channel import PartialChannel
from dislord.discord.resources.channel.partial_message import PartialMessage
from dislord.discord.resources.guild.guild_member import PartialGuildMember
from dislord.discord.resources.user.user import User
from dislord.discord.reference import Snowflake, Missing


class ApplicationCommandInteractionDataOption(ObjDict):
    name: str
    type: ApplicationCommandOptionType
    value: str | int | float | bool | Missing = None
    options: list['ApplicationCommandInteractionDataOption'] | Missing = None
    focused: bool | Missing = None


class ModalSubmitData(ObjDict):
    custom_id: str
    components: list[Component]


class ResolvedData(ObjDict):
    users: dict[Snowflake, User] | Missing = None
    members: dict[Snowflake, PartialGuildMember] | Missing = None
    roles: dict[Snowflake, Role] | Missing = None
    channels: dict[Snowflake, PartialChannel] | Missing = None
    messages: dict[Snowflake, PartialMessage] | Missing = None
    attachments: dict[Snowflake, Attachment] | Missing = None


class MessageComponentData(ObjDict):
    custom_id: str
    component_type: ComponentType
    values: list[str] | Missing = None
    resolved: ResolvedData | Missing = None


class ApplicationCommandData(ObjDict):
    id: Snowflake
    name: str
    type: int
    resolved: ResolvedData | Missing = None
    options: list[ApplicationCommandInteractionDataOption] | Missing = None
    guild_id: Snowflake | Missing = None
    target: Snowflake | Missing = None


InteractionData = ApplicationCommandData | MessageComponentData | ModalSubmitData
