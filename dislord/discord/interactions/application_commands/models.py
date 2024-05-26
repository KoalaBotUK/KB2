from typing import Self

from dislord.discord.base import BaseModel
from dislord.discord.interactions.application_commands.enums import ApplicationCommandType, \
    ApplicationCommandPermissionType, ApplicationCommandOptionType
from dislord.discord.interactions.receiving_and_responding.interaction import InteractionContextType
from dislord.discord.locale import Locale
from dislord.discord.resources.application.enums import ApplicationIntegrationType
from dislord.discord.resources.channel.channel import ChannelType
from dislord.discord.type import Snowflake, Missing


class ApplicationCommandPermissions(BaseModel):
    id: Snowflake
    type: ApplicationCommandPermissionType
    permission: bool


class GuildApplicationCommandPermissions(BaseModel):
    id: Snowflake
    application_id: Snowflake
    guild_id: Snowflake
    permissions: list[ApplicationCommandPermissions]


ApplicationCommandPermissionsObject = GuildApplicationCommandPermissions


class ApplicationCommandOptionChoice(BaseModel):
    name: str
    name_localizations: dict[Locale, str] | Missing | None
    value: str | int | float


class ApplicationCommandOption(BaseModel):
    type: ApplicationCommandOptionType
    name: str
    name_localizations: dict[Locale, str] | Missing | None
    description: str
    description_localizations: dict[Locale, str] | Missing | None
    required: bool | Missing
    choices: list[ApplicationCommandOptionChoice] | Missing
    options: list[Self] | Missing
    channel_types: list[ChannelType] | Missing
    min_value: int | float | Missing
    max_values: int | float | Missing
    min_length: int | Missing
    max_length: int | Missing
    autocomplete: bool | Missing


class ApplicationCommand(BaseModel):
    id: Snowflake
    type: ApplicationCommandType | Missing
    application_id: Snowflake
    guild_id: Snowflake | Missing
    name: str
    name_localizations: dict[Locale, str] | Missing| None
    description: str
    description_localizations: dict[Locale, str] | Missing | None
    options: list[ApplicationCommandOption] | Missing
    # default_member_permissions: Permissions | None FIXME
    dm_permission: bool | Missing   # Deprecated
    default_permission: bool | Missing | None
    nsfw: bool | Missing
    integration_types: list[ApplicationIntegrationType] | Missing
    contexts: list[InteractionContextType] | Missing | None
    version: Snowflake

    def __eq__(self, other):
        eq_list = ['guild_id', 'name', 'description', 'type', 'name_localization', 'description_localizations',
                   'options', 'default_member_permissions', 'dm_permission', 'default_permission', 'nsfw']
        result = True
        for eq_attr in eq_list:
            self_attr = getattr(self, eq_attr, None)
            other_attr = getattr(other, eq_attr, None)
            result = result and (self_attr == other_attr or self_attr is other_attr) # compare_missing_none(self_attr, other_attr)
        return result

    def __post_init__(self):
        if self.guild_id is not None and self.guild_id is not Missing:
            self.dm_permission = None
