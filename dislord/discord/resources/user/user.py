from enum import IntEnum, IntFlag

from dislord.types import ObjDict
from dislord.discord.reference import Snowflake, Missing, Locale


class PremiumType(IntEnum):
    NONE = 0
    NITRO_CLASSIC = 1
    NITRO = 2
    NITRO_BASIC = 3


class UserFlags(IntFlag):
    NONE = 0
    STAFF = 1 << 0
    PARTNER = 1 << 1
    HYPESQUAD = 1 << 2
    BUG_HUNTER_LEVEL_1 = 1 << 3
    HYPESQUAD_ONLINE_HOUSE_1 = 1 << 6
    HYPESQUAD_ONLINE_HOUSE_2 = 1 << 7
    HYPESQUAD_ONLINE_HOUSE_3 = 1 << 8
    PREMIUM_EARLY_SUPPORTER = 1 << 9
    TEAM_PSEUDO_USER = 1 << 10
    BUG_HUNTER_LEVEL_2 = 1 << 14
    VERIFIED_BOT = 1 << 16
    VERIFIED_DEVELOPER = 1 << 17
    CERTIFIED_MODERATOR = 1 << 18
    BOT_HTTP_INTERACTIONS = 1 << 19
    ACTIVE_DEVELOPER = 1 << 22


class PartialUser(ObjDict):
    id: Snowflake
    username: str
    discriminator: str
    avatar: str | None = None


class User(PartialUser):
    global_name: str | None = None
    bot: bool | Missing = None
    system: bool | Missing = None
    mfa_enabled: bool | Missing = None
    banner: str | Missing | None = None
    accent_color: int | Missing | None = None
    locale: Locale | Missing = None
    verified: bool | Missing = None
    email: str | Missing | None = None
    flags: UserFlags | Missing = UserFlags.NONE
    premium_type: PremiumType | Missing = None
    public_flags: UserFlags | Missing = UserFlags.NONE
    avatar_decoration: str | Missing | None = None
