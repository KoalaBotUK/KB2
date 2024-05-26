from dislord.discord.base import BaseModel
from dislord.discord.resources.user.user import User
from dislord.discord.type import Snowflake, Missing


class IntegrationApplication(BaseModel):
    id: Snowflake
    name: str
    icon: str | None
    description: str
    bot: User | Missing
