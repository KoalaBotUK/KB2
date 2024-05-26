from dislord.discord.base import BaseModel
from dislord.discord.type import ISOTimestamp, Missing


class ThreadMetadata(BaseModel):
    archived: bool
    auto_archive_duration: int
    archive_timestamp: ISOTimestamp
    locked: bool
    invitable: bool | Missing
    create_timestamp: ISOTimestamp | Missing | None
