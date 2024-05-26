from dislord.discord.base import BaseModel
from dislord.discord.resources.guild.guild_member import PartialGuildMember


class InviteStageInstance(BaseModel):
    members: list[PartialGuildMember]
    participant_count: int
    speaker_count: int
    topic: str
