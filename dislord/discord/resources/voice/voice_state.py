from dislord.discord.base import BaseModel
from dislord.discord.resources.guild.guild_member import GuildMember
from dislord.discord.type import Snowflake, Missing, ISOTimestamp


class VoiceState(BaseModel):
    guild_id: Snowflake | Missing
    channel_id: Snowflake | None
    user_id: Snowflake
    member: GuildMember | Missing
    session_id: str
    deaf: bool
    mute: bool
    self_deaf: bool
    self_mute: bool
    self_stream: bool | Missing
    self_video: bool
    suppress: bool
    request_to_speak_timestamp: ISOTimestamp | None
