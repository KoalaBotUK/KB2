from dislord.discord.base import BaseModel
from dislord.discord.resources.sticker.sticker import Sticker
from dislord.discord.type import Snowflake, Missing


class StickerPack(BaseModel):
    id: Snowflake
    stickers: list[Sticker]
    name: str
    sku_id: Snowflake
    cover_sticker_id: Snowflake | Missing
    description: str
    banner_asset_id: Snowflake | Missing
