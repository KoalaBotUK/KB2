from dislord.discord.base import BaseModel
from dislord.discord.type import Snowflake


class ForumTag(BaseModel):
    id: Snowflake
    name: str
    moderated: bool
    emoji_id: Snowflake | None
    emoji_name: str | None
