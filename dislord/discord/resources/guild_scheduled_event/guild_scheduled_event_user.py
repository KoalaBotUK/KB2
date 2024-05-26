from dislord.discord.base import BaseModel
from dislord.discord.resources.guild.guild_member import GuildMember
from dislord.discord.resources.user.user import User
from dislord.discord.type import Snowflake, Missing


class GuildScheduledEventUser(BaseModel):
    guild_scheduled_event_id: Snowflake
    user: User
    member: GuildMember | Missing
