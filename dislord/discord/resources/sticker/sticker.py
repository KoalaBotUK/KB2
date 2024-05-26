from enum import IntEnum

from dislord.discord.base import BaseModel
from dislord.discord.resources.user.user import User
from dislord.discord.type import Snowflake, Missing


class StickerFormatType(IntEnum):
    PNG = 1
    APNG = 2
    LOTTIE = 3
    GIF = 4


class StickerType(IntEnum):
    STANDARD = 1
    GUILD = 2


class Sticker(BaseModel):
    id: Snowflake
    pack_id: Snowflake | Missing
    name: str
    description: str | None
    tags: str
    asset: str | Missing  # Deprecated
    type: StickerType
    format_type: StickerFormatType
    available: bool | Missing
    guild_id: Snowflake | Missing
    user: User | Missing
    sort_value: int | Missing
