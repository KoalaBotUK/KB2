
from dislord.discord.reference import Snowflake
from dislord.types import ObjDict


class DefaultReaction(ObjDict):
    emoji_id: Snowflake | None = None
    emoji_name: str | None = None
