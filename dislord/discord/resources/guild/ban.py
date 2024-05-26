from dislord.discord.base import BaseModel
from dislord.discord.resources.user.user import User


class Ban(BaseModel):
    reason: str | None
    user: User
