from enum import IntFlag, IntEnum

from dislord.types import ObjDict
from dislord.discord.resources.channel.default_reaction import DefaultReaction
from dislord.discord.resources.channel.forum_tag import ForumTag
from dislord.discord.resources.channel.overwrite import Overwrite
from dislord.discord.resources.channel.thread_member import ThreadMember
from dislord.discord.resources.channel.thread_metadata import ThreadMetadata
from dislord.discord.resources.user.user import User
from dislord.discord.reference import Missing, Snowflake, ISOTimestamp


class ForumLayoutType(IntEnum):
    NOT_SET = 0
    LIST_VIEW = 1
    GALLERY_VIEW = 2


class SortOrderType(IntEnum):
    LATEST_ACTIVITY = 0
    CREATION_DATE = 1


class ChannelFlags(IntFlag):
    NONE = 0
    PINNED = 1 << 1
    REQUIRE_TAG = 1 << 4
    HIDE_MEDIA_DOWNLOAD_OPTIONS = 1 << 15


class VideoQualityMode(IntEnum):
    AUTO = 1
    FULL = 2


class ChannelType(IntEnum):
    GUILD_TEXT = 0
    DM = 1
    GUILD_VOICE = 2
    GROUP_DM = 3
    GUILD_CATEGORY = 4
    GUILD_ANNOUNCEMENT = 5
    ANNOUNCEMENT_THREAD = 10
    PUBLIC_THREAD = 11
    PRIVATE_THREAD = 12
    GUILD_STAGE_VOICE = 13
    GUILD_DIRECTORY = 14
    GUILD_FORUM = 15
    GUILD_MEDIA = 16


class PartialChannel(ObjDict):
    id: Snowflake
    type: ChannelType
    name: str | Missing | None = None
    permissions: str | Missing = None
    thread_metadata: ThreadMetadata | Missing = None
    parent_id: Snowflake | Missing | None = None


class Channel(PartialChannel):
    guild_id: Snowflake | Missing = None
    position: int | Missing = None
    permission_overwrites: list[Overwrite] | Missing = None
    topic: str | Missing | None = None
    nsfw: bool | Missing = None
    last_message_id: Snowflake | Missing | None = None
    bitrate: int | Missing = None
    user_limit: int | Missing = None
    rate_limit_per_user: int | Missing = None
    recipients: list[User] | Missing = None
    icon: str | Missing | None = None
    owner_id: Snowflake | Missing = None
    application_id: Snowflake | Missing = None
    managed: bool | Missing = None
    last_pin_timestamp: ISOTimestamp | Missing | None = None
    rtc_region: str | Missing | None = None
    video_quality_mode: int | Missing = None
    message_count: int | Missing = None
    member_count: int | Missing = None
    member: ThreadMember | Missing = None
    default_auto_archive_duration: int | Missing = None
    flags: ChannelFlags | Missing = ChannelFlags.NONE
    total_message_sent: int | Missing = None
    available_tags: list[ForumTag] | Missing = None
    applied_tags: list[Snowflake] | Missing = None
    default_reaction_emoji: DefaultReaction | Missing | None = None
    default_thread_rate_limit_per_user: int | Missing = None
    default_sort_order: int | Missing | None = None
    default_forum_layout: int | Missing = None
