from dislord.discord.base import BaseModel
from dislord.discord.resources.channel.reaction_count_details import ReactionCountDetails
from dislord.discord.resources.emoji.emoji import PartialEmoji
from dislord.discord.type import HexColor


class Reaction(BaseModel):
    count: int
    count_details: ReactionCountDetails
    me: bool
    me_burst: bool
    emoji: PartialEmoji
    bust_colors: list[HexColor]
