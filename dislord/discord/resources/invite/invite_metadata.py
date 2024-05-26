from dislord.discord.base import BaseModel
from dislord.discord.type import ISOTimestamp


class InviteMetadata(BaseModel):
    uses: int
    max_uses: int
    max_age: int
    temporary: bool
    created_at: ISOTimestamp
