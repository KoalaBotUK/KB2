from typing import Optional
from dataclasses import dataclass

from .base import BaseModel
from .type import Snowflake, PartialOptional, Missing


@dataclass
class Guild(BaseModel):
    id: Snowflake
    name: str
    # features: list[GuildFeature] FIXME: Add Enum
    icon: Optional[str]
    owner: Optional[bool]
    permissions: Optional[str]
    approximate_member_count: Optional[int]
    approximate_presence_count: Optional[int]

    owner_id: PartialOptional[Snowflake]
    afk_timeout: PartialOptional[int]
    verification_level: PartialOptional[int]
    default_message_notifications: PartialOptional[int]
    explicit_content_filter: PartialOptional[int]
    # roles: PartialOptional[list[Role]] FIXME: Add Enum
    # emojis: PartialOptional[list[Emoji]] FIXME: Add Enum
    mfa_level: PartialOptional[int]
    system_channel_flags: PartialOptional[int]
    premium_tier: PartialOptional[int]
    public_updates_channel_id: PartialOptional[Snowflake]
    nsfw_level: PartialOptional[int]
    premium_progress_bar_enabled: PartialOptional[bool]
    safety_alerts_channel_id: Optional[Snowflake]
    icon_hash: Optional[str]
    splash: Optional[str]
    discovery_splash: Optional[str]
    region: Optional[str]
    afk_channel_id: Optional[Snowflake]
    widget_enabled: Optional[bool]
    widget_channel_id: Optional[Snowflake]
    application_id: Optional[Snowflake]
    system_channel_id: Optional[Snowflake]
    rules_channel_id: Optional[Snowflake]
    max_presences: Optional[int]
    max_members: Optional[int]
    vanity_url_code: Optional[str]
    description: Optional[str]
    banner: Optional[str]
    preferred_locale: Optional[str]
    max_video_channel_users: Optional[int]
    # welcome_screen: Optional[WelcomeScreen] FIXME
    # stickers: Optional[list[Sticker]] FIXME


@dataclass
class PartialGuild(Guild):
    def __getattribute__(self, item):
        attr = super().__getattribute__(item)
        if attr is Missing:
            self.__class__ = Guild
            guild = self.client.get_guild(self.id)
            for a in self.__annotations__:
                setattr(self, a, getattr(guild, a))
            return self.__getattribute__(item)
        else:
            return attr
