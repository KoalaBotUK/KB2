from dislord.types import ObjDict
from dislord.discord.interactions.components.enums import ButtonStyle, TextInputStyle, ComponentType
from dislord.discord.resources.channel.channel import ChannelType
from dislord.discord.reference import Snowflake, Missing
from dislord.discord.resources.emoji.emoji import PartialEmoji


class TextInput(ObjDict):
    type: ComponentType = ComponentType.TEXT_INPUT
    custom_id: str
    style: TextInputStyle
    label: str
    min_length: int | Missing = None
    max_length: int | Missing = None
    required: bool | Missing = None
    value: str | Missing = None
    placeholder: str | Missing = None


class SelectDefaultValue(ObjDict):
    id: Snowflake
    type: str


class SelectOption(ObjDict):
    label: str
    value: str
    description: str | Missing = None
    emoji: PartialEmoji | Missing = None
    default: bool | Missing = None


class SelectMenu(ObjDict):
    type: ComponentType
    custom_id: str
    options: list[SelectOption] | Missing = None
    channel_types: list[ChannelType] | Missing = None
    placeholder: str | Missing = None
    default_values: list[SelectDefaultValue] | Missing = None
    min_values: int | Missing = None
    max_values: int | Missing = None
    disabled: bool | Missing = None


class Button(ObjDict):
    type: ComponentType = ComponentType.BUTTON
    style: ButtonStyle
    label: str | None = None
    emoji: PartialEmoji | Missing = None
    custom_id: str | Missing = None
    url: str | Missing = None
    disabled: bool | Missing = None


class ActionRow(ObjDict):
    type: ComponentType = ComponentType.ACTION_ROW
    components: list[Button | SelectMenu | TextInput]


Component = ActionRow | Button | SelectMenu | TextInput
