from dislord.types import ObjDict
from dislord.discord.resources.user.user import User
from dislord.discord.reference import Snowflake, Missing


class PartialEmoji(ObjDict):
    id: Snowflake | None = None
    name: str | None = None
    animated: bool | Missing = None


class Emoji(PartialEmoji):
    roles: list[Snowflake] | Missing = None
    user: User
    require_colons: bool | Missing = None
    managed: bool | Missing = None
    available: bool | Missing = None
