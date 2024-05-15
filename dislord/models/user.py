from dataclasses import dataclass
from typing import Optional

from .base import BaseModel
from .type import Snowflake


@dataclass
class User(BaseModel):
    id: Snowflake
    username: str
    discriminator: str
    global_name: Optional[str]
    avatar: Optional[str]
    bot: Optional[bool]
    system: Optional[bool]
    mfa_enabled: Optional[bool]
    banner: Optional[str]
    accent_color: Optional[int]
    locale: Optional[str]
    verified: Optional[bool]
    email: Optional[str]
    flags: Optional[int]
    premium_type: Optional[int]
    public_flags: Optional[int]
    avatar_decoration: Optional[str]
