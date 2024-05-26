from dislord.discord.base import BaseModel
from dislord.discord.resources.channel.channel import PartialChannel
from dislord.discord.resources.user.user import PartialUser
from dislord.discord.type import Snowflake


class GuildWidget(BaseModel):
    id: Snowflake
    name: str
    instant_invite: str | None
    channels: list[PartialChannel]
    members: list[PartialUser]
    presence_count: int
