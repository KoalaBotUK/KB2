from dislord.discord.base import BaseModel
from dislord.discord.type import Snowflake


class WelcomeScreenChannel(BaseModel):
    channel_id: Snowflake
    description: str
    emoji_id: Snowflake | None
    emoji_name: str | None


class WelcomeScreen(BaseModel):
    description: str | None
    welcome_channel: list[WelcomeScreenChannel]
