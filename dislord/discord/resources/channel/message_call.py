from dislord.discord.base import BaseModel
from dislord.discord.type import Snowflake, ISOTimestamp, Missing


class MessageCall(BaseModel):
    participants: list[Snowflake]
    ended_timestamp: ISOTimestamp | Missing | None
