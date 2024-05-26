from dislord.discord.base import BaseModel
from dislord.discord.resources.guild.guild import PartialGuild
from dislord.discord.resources.user.user import User
from dislord.discord.type import Snowflake, ISOTimestamp


class GuildTemplate(BaseModel):
    code: str
    name: str
    description: str | None
    usage_count: int
    creator_id: Snowflake
    creator: User
    created_at: ISOTimestamp
    updated_at: ISOTimestamp
    source_guild_id: Snowflake
    serialized_source_guild: PartialGuild
    is_dirty: bool | None
