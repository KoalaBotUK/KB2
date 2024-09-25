from dislord.types import ObjDict
from dislord.discord.resources.guild.guild_member import GuildMember
from dislord.discord.reference import Snowflake, Missing, ISOTimestamp


class VoiceState(ObjDict):
    guild_id: Snowflake | Missing = None
    channel_id: Snowflake | None = None
    user_id: Snowflake
    member: GuildMember | Missing = None
    session_id: str
    deaf: bool
    mute: bool
    self_deaf: bool
    self_mute: bool
    self_stream: bool | Missing = None
    self_video: bool
    suppress: bool
    request_to_speak_timestamp: ISOTimestamp | None = None
