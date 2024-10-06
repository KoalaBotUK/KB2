from enum import IntEnum
from typing import Literal

from dislord.discord.interactions.receiving_and_responding.interaction_data import InteractionData, \
    ApplicationCommandData, MessageComponentData, ModalSubmitData
from dislord.discord.interactions.receiving_and_responding.message_interaction import InteractionType
from dislord.discord.reference import Snowflake, Missing, Locale
from dislord.discord.resources.application.enums import ApplicationIntegrationType
from dislord.discord.resources.channel.channel import PartialChannel
from dislord.discord.resources.channel.message import Message
from dislord.discord.resources.guild.guild import PartialGuild
from dislord.discord.resources.guild.guild_member import GuildMember
from dislord.discord.resources.user.user import User
from dislord.types import ObjDict


class InteractionContextType(IntEnum):
    GUILD = 0
    BOT_DM = 1
    PRIVATE_CHANNEL = 2


class __Interaction(ObjDict):
    id: Snowflake
    application_id: Snowflake
    type: InteractionType
    data: InteractionData | Missing = None
    guild: PartialGuild | Missing = None
    guild_id: Snowflake | Missing = None
    channel: PartialChannel | Missing = None
    channel_id: Snowflake | Missing = None
    member: GuildMember | Missing = None
    user: User | Missing = None
    token: str
    version: int
    message: Message | Missing = None
    app_permissions: str
    locale: Locale | Missing = None
    guild_locale: Locale | Missing = None
    # entitlements: list[Entitlement] FIXME
    authorizing_integration_owners: dict[ApplicationIntegrationType, Snowflake]
    context: InteractionContextType | Missing = None


class PingInteraction(__Interaction):
    type: Literal[InteractionType.PING]
    data: Missing = None


class ApplicationCommandInteraction(__Interaction):
    type: Literal[InteractionType.APPLICATION_COMMAND, InteractionType.APPLICATION_COMMAND_AUTOCOMPLETE]
    data: ApplicationCommandData


class MessageInteraction(__Interaction):
    type: Literal[InteractionType.MESSAGE_COMPONENT]
    data: MessageComponentData


class ModalSubmitInteraction(__Interaction):
    type: Literal[InteractionType.MODAL_SUBMIT]
    data: ModalSubmitData


Interaction = PingInteraction | ApplicationCommandInteraction | MessageInteraction | ModalSubmitInteraction
