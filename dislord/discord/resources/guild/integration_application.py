from dislord.types import ObjDict
from dislord.discord.resources.user.user import User
from dislord.discord.reference import Snowflake, Missing


class IntegrationApplication(ObjDict):
    id: Snowflake
    name: str
    icon: str | None = None
    description: str
    bot: User | Missing = None
