from enum import Enum
from typing import Literal

from dislord.discord.base import BaseModel
from dislord.discord.interactions.receiving_and_responding.interaction_data import InteractionData, \
    ApplicationCommandData, MessageComponentData, ModalSubmitData
from dislord.discord.interactions.receiving_and_responding.message_interaction import InteractionType
from dislord.discord.locale import Locale
from dislord.discord.resources.application.enums import ApplicationIntegrationType
from dislord.discord.resources.application.models import ApplicationIntegrationTypeConfiguration
from dislord.discord.resources.channel.channel import PartialChannel
from dislord.discord.resources.channel.message import Message
from dislord.discord.resources.guild.guild import Guild
from dislord.discord.resources.guild.guild_member import GuildMember
from dislord.discord.resources.user.user import User
from dislord.discord.type import Snowflake, Missing


class InteractionContextType(Enum):
    GUILD = 0
    BOT_DM = 1
    PRIVATE_CHANNEL = 2


class __Interaction(BaseModel):
    id: Snowflake
    application_id: Snowflake
    type: InteractionType
    data: InteractionData | Missing
    guild: Guild | Missing
    guild_id: Snowflake | Missing
    channel: PartialChannel | Missing
    channel_id: Snowflake | Missing
    member: GuildMember | Missing
    user: User | Missing
    token: str
    version: int
    message: Message | Missing
    app_permissions: str
    locale: Locale | Missing
    guild_locale: Locale | Missing
    # entitlements: list[Entitlement] FIXME
    authorizing_integration_owners: dict[ApplicationIntegrationType, ApplicationIntegrationTypeConfiguration]
    context: InteractionContextType | Missing


class PingInteraction(__Interaction):
    type: 1
    data: Missing


class ApplicationCommandInteraction(__Interaction):
    type: (2, 4)
    type2: [2, 4]
    data: ApplicationCommandData


class MessageInteraction(__Interaction):
    type: 3
    data: MessageComponentData


class ModalSubmitInteraction(__Interaction):
    type: 5
    data: ModalSubmitData

Interaction = ApplicationCommandInteraction | MessageInteraction | ModalSubmitInteraction
