from dislord.discord.base import BaseModel
from dislord.discord.type import Snowflake


class GuildWidgetSettings(BaseModel):
    enabled: bool
    channel_id: Snowflake | None
