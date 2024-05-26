from dislord.discord.base import BaseModel
from dislord.discord.type import Snowflake


class DefaultReaction(BaseModel):
    emoji_id: Snowflake | None
    emoji_name: str | None
