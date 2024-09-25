from dislord.types import ObjDict
from dislord.discord.resources.emoji.emoji import Emoji
from dislord.discord.resources.guild.guild import GuildFeatures
from dislord.discord.resources.sticker.sticker import Sticker
from dislord.discord.reference import Snowflake


class GuildPreview(ObjDict):
    id: Snowflake
    name: str
    icon: str | None = None
    splash: str | None = None
    discovery_splash: str | None = None
    emojis: list[Emoji]
    features: list[GuildFeatures]
    approximate_member_count: int
    approximate_presence_count: int
    description: str | None = None
    stickers: list[Sticker]
