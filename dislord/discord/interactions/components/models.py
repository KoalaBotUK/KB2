from dislord.discord.base import BaseModel
from dislord.discord.interactions.components.enums import ButtonStyle, TextInputStyle, ComponentType
from dislord.discord.resources.channel.channel import ChannelType
from dislord.discord.type import Snowflake, Missing


class TextInput(BaseModel):
    type: ComponentType = ComponentType.TEXT_INPUT
    custom_id: str
    style: TextInputStyle
    label: str
    min_length: int | Missing
    max_length: int | Missing
    required: bool | Missing
    value: str | Missing
    placeholder: str | Missing


class SelectDefaultValue(BaseModel):
    id: Snowflake
    type: str


class SelectOption(BaseModel):
    label: str
    value: str
    description: str | Missing
    # emoji: PartialEmoji | Missing FIXME
    default: bool | Missing


class SelectMenu(BaseModel):
    type: ComponentType
    custom_id: str
    options: list[SelectOption] | Missing
    channel_types: list[ChannelType] | Missing
    placeholder: str | Missing
    default_values: list[SelectDefaultValue] | Missing
    min_values: int | Missing
    max_values: int | Missing
    disabled: bool | Missing


class Button(BaseModel):
    type: ComponentType = ComponentType.BUTTON
    style: ButtonStyle
    label: str | None
    # emoji: PartialEmoji | Missing FIXME
    custom_id: str | Missing
    url: str | Missing
    disabled: bool | Missing


class ActionRow(BaseModel):
    type: ComponentType = ComponentType.ACTION_ROW
    components: list[Button | SelectMenu | TextInput]


Component = ActionRow | Button | SelectMenu | TextInput
