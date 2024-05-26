from dislord.discord.base import BaseModel


class VoiceRegion(BaseModel):
    id: str
    name: str
    optimal: bool
    deprecated: bool
    custom: bool
