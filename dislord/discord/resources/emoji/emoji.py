from dislord.discord.base import BaseModel
from dislord.discord.resources.user.user import User
from dislord.discord.type import Snowflake, Missing


class PartialEmoji(BaseModel):
    id: Snowflake | None
    name: str | None
    animated: bool | Missing


class Emoji(PartialEmoji):
    roles: list[Snowflake] | Missing
    user: User
    require_colons: bool | Missing
    managed: bool | Missing
    available: bool | Missing
