from enum import IntEnum

from dislord.types import ObjDict
from dislord.discord.resources.user.user import User
from dislord.discord.reference import Snowflake, Missing


class StickerFormatType(IntEnum):
    PNG = 1
    APNG = 2
    LOTTIE = 3
    GIF = 4


class StickerType(IntEnum):
    STANDARD = 1
    GUILD = 2


class Sticker(ObjDict):
    id: Snowflake
    pack_id: Snowflake | Missing = None
    name: str
    description: str | None = None
    tags: str
    asset: str | Missing  # Deprecated
    type: StickerType
    format_type: StickerFormatType
    available: bool | Missing = None
    guild_id: Snowflake | Missing = None
    user: User | Missing = None
    sort_value: int | Missing = None
