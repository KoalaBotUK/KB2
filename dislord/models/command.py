from dataclasses import dataclass
from enum import Enum
from typing import Optional, Union

from .base import BaseModel
from .channel import Channel, ChannelType
from .locale import Locale
from .type import Snowflake, PartialOptional, Missing
from .user import User


class ApplicationCommandOptionType(Enum):
    SUB_COMMAND = 1
    SUB_COMMAND_GROUP = 2
    STRING = 3
    INTEGER = 4
    BOOLEAN = 5
    USER = 6
    CHANNEL = 7
    ROLE = 8
    MENTIONABLE = 9
    NUMBER = 10
    ATTACHMENT = 11

    def from_python_type(self, type_hint):
        python_mapping = {str: self.STRING, int: self.INTEGER, bool: self.BOOLEAN,
                          User: self.USER, Channel: self.CHANNEL,
                          # Role: self.ROLE, Mentionable: self.MENTIONABLE, FIXME
                          float: self.NUMBER,
                          # Attachment: self.ATTACHMENT FIXME
                          }
        par = python_mapping.get(type_hint)
        if par is None:
            raise RuntimeError(f"Unexpected command param type: {type_hint}")

    def __eq__(self, other):
        try:
            return self.value == other.value
        except Exception:
            return False


@dataclass
class ApplicationCommandOptionChoice(BaseModel):
    name: str
    value: [str, int, float]
    name_localizations: Optional[dict[Locale, str]]


@dataclass
class ApplicationCommandOption(BaseModel):
    type: ApplicationCommandOptionType
    name: str
    description: str

    name_localizations: Optional[dict[Locale, str]]
    description_localizations: Optional[dict[Locale, str]]
    required: Optional[bool]
    choices: Optional[list[ApplicationCommandOptionChoice]]
    options: Optional[list['ApplicationCommandOption']]
    channel_types: Optional[list[ChannelType]]
    min_value: Optional[Union[int, float]]
    max_value: Optional[Union[int, float]]
    min_length: Optional[int]
    max_length: Optional[int]
    autocomplete: Optional[bool]


# _ApplicationCommandOption = Union[ApplicationCommandOption]

class ApplicationCommandType(Enum):
    CHAT_INPUT = 1
    USER = 2
    MESSAGE = 3


@dataclass
class ApplicationCommand(BaseModel):
    id: PartialOptional[Snowflake]
    application_id: PartialOptional[Snowflake]
    guild_id: Optional[Snowflake]
    name: str
    description: Optional[str]
    version: PartialOptional[Snowflake]

    type: Optional[ApplicationCommandType]
    name_localizations: Optional[dict[Locale, str]]
    description_localizations: Optional[dict[Locale, str]]
    options: Optional[list[ApplicationCommandOption]]
    default_member_permissions: Optional[str]
    dm_permission: Optional[bool]
    default_permission: Optional[bool]
    nsfw: Optional[bool]

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
