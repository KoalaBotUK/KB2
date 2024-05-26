from dislord.discord.base import BaseModel
from dislord.discord.type import Snowflake


class AvatarDecorationData(BaseModel):
    asset: str
    sku_id: Snowflake
