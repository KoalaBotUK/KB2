from enum import IntFlag

from dislord.types import ObjDict
from dislord.discord.resources.guild.guild_member import GuildMember
from dislord.discord.reference import Snowflake, Missing, ISOTimestamp


class ThreadMember(ObjDict):
    id: Snowflake | Missing = None
    user_id: Snowflake | Missing = None
    join_timestamp: ISOTimestamp
    flags: IntFlag  # Any user-thread settings, currently only used for notifications
    member: GuildMember | Missing = None
