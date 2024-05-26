from enum import IntFlag

from dislord.discord.base import BaseModel
from dislord.discord.type import Snowflake, Missing


class AttachmentFlag(IntFlag):
    IS_REMIX = 1 << 2


class PartialAttachment(BaseModel):
    id: Snowflake
    filename: str
    description: str | Missing


class Attachment(PartialAttachment):
    content_type: str | Missing
    size: int
    url: str
    proxy_url: str
    height: int | Missing | None
    width: int | Missing | None
    ephemeral: bool | Missing
    duration_secs: float | Missing
    waveform: str | Missing
    flags: AttachmentFlag
