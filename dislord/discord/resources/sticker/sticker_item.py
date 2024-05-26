from dislord.discord.base import BaseModel
from dislord.discord.resources.sticker.sticker import StickerFormatType
from dislord.discord.type import Snowflake


class StickerItem(BaseModel):
    id: Snowflake
    name: str
    format_type: StickerFormatType
