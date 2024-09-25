from dislord.discord.interactions.application_commands.enums import ApplicationCommandType, \
    ApplicationCommandPermissionType, ApplicationCommandOptionType
from dislord.discord.interactions.receiving_and_responding.interaction import InteractionContextType
from dislord.discord.resources.application.enums import ApplicationIntegrationType
from dislord.discord.resources.channel.channel import ChannelType
from dislord.discord.reference import Snowflake, Missing, Locale
from dislord.discord.topics.permissions.permissions import Permission
from dislord.types import ObjDict


class ApplicationCommandPermissions(ObjDict):
    id: Snowflake
    type: ApplicationCommandPermissionType
    permission: bool


class GuildApplicationCommandPermissions(ObjDict):
    id: Snowflake
    application_id: Snowflake
    guild_id: Snowflake
    permissions: list[ApplicationCommandPermissions]


ApplicationCommandPermissionsObject = GuildApplicationCommandPermissions


class ApplicationCommandOptionChoice(ObjDict):
    name: str
    name_localizations: dict[Locale, str] | Missing | None = None
    value: str | int | float


class ApplicationCommandOption(ObjDict):
    type: ApplicationCommandOptionType
    name: str
    name_localizations: dict[Locale, str] | Missing | None = None
    description: str
    description_localizations: dict[Locale, str] | Missing | None = None
    required: bool | Missing = None
    choices: list[ApplicationCommandOptionChoice] | Missing = None
    options: list['ApplicationCommandOption'] | Missing = None
    channel_types: list[ChannelType] | Missing = None
    min_value: int | float | Missing = None
    max_values: int | float | Missing = None
    min_length: int | Missing = None
    max_length: int | Missing = None
    autocomplete: bool | Missing = None


class ApplicationCommand(ObjDict):
    id: Snowflake = None
    type: ApplicationCommandType | Missing = None
    application_id: Snowflake
    guild_id: Snowflake | Missing = None
    name: str
    name_localizations: dict[Locale, str] | Missing | None = None
    description: str
    description_localizations: dict[Locale, str] | Missing | None = None
    options: list[ApplicationCommandOption] | Missing = None
    default_member_permissions: Permission | None = None
    dm_permission: bool | Missing = None   # Deprecated
    default_permission: bool | Missing | None = True
    nsfw: bool | Missing = False
    integration_types: list[ApplicationIntegrationType] | Missing = [ApplicationIntegrationType.GUILD_INSTALL]
    contexts: list[InteractionContextType] | Missing | None = None
    version: Snowflake = None
