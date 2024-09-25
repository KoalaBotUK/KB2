from enum import IntFlag

from dislord.types import ObjDict
from dislord.discord.reference import Snowflake, Missing


class AttachmentFlag(IntFlag):
    IS_REMIX = 1 << 2


class PartialAttachment(ObjDict):
    id: Snowflake
    filename: str
    description: str | Missing = None


class Attachment(PartialAttachment):
    content_type: str | Missing = None
    size: int
    url: str
    proxy_url: str
    height: int | Missing | None = None
    width: int | Missing | None = None
    ephemeral: bool | Missing = None
    duration_secs: float | Missing = None
    waveform: str | Missing = None
    flags: AttachmentFlag
