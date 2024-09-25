from enum import IntFlag

from dislord.types import ObjDict
from dislord.discord.resources.user.avatar_decoration_data import AvatarDecorationData
from dislord.discord.resources.user.user import User
from dislord.discord.reference import Missing, Snowflake, ISOTimestamp


class GuildMemberFlags(IntFlag):
    DID_REJOIN = 1 << 0
    COMPLETED_ONBOARDING = 1 << 1
    BYPASSES_VERIFICATION = 1 << 2
    STARTED_ONBOARDING = 1 << 3


class PartialGuildMember(ObjDict):
    nick: str | Missing | None = None
    avatar: str | Missing | None = None
    roles: list[Snowflake]
    joined_at: ISOTimestamp
    premium_since: ISOTimestamp | Missing | None = None
    flags: GuildMemberFlags
    pending: bool | Missing = None
    permissions: str | Missing = None
    communication_disabled_until: ISOTimestamp | Missing | None = None
    avatar_decoration_data: AvatarDecorationData | Missing | None = None


class GuildMember(PartialGuildMember):
    user: User | Missing = None
    deaf: bool
    mute: bool
