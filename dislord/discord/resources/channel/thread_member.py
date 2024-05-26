from enum import IntFlag

from dislord.discord.base import BaseModel
from dislord.discord.resources.guild.guild_member import GuildMember
from dislord.discord.type import Snowflake, Missing, ISOTimestamp


class ThreadMember(BaseModel):
    id: Snowflake | Missing
    user_id: Snowflake | Missing
    join_timestamp: ISOTimestamp
    flags: IntFlag  # Any user-thread settings, currently only used for notifications
    member: GuildMember | Missing
