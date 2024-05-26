from dislord.discord.base import BaseModel
from dislord.discord.resources.emoji.emoji import PartialEmoji
from dislord.discord.type import Missing


class PollMedia(BaseModel):
    text: str | Missing
    emoji: PartialEmoji | Missing
