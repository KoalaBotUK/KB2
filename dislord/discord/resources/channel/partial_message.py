from enum import IntEnum

from dislord.discord.base import BaseModel
from dislord.discord.permissions import Role
from dislord.discord.resources.channel.attachment import Attachment
from dislord.discord.resources.channel.embed import Embed
from dislord.discord.resources.channel.reaction import Reaction
from dislord.discord.resources.user.user import User
from dislord.discord.type import Snowflake, ISOTimestamp


class MessageType(IntEnum):
    DEFAULT = 0
    RECIPIENT_ADD = 1
    RECIPIENT_REMOVE = 2
    CALL = 3
    CHANNEL_NAME_CHANGE = 4
    CHANNEL_ICON_CHANGE = 5
    CHANNEL_PINNED_MESSAGE = 6
    USER_JOIN = 7
    GUILD_BOOST = 8
    GUILD_BOOST_TIER_1 = 9
    GUILD_BOOST_TIER_2 = 10
    GUILD_BOOST_TIER_3 = 11
    CHANNEL_FOLLOW_ADD = 12
    GUILD_DISCOVERY_DISQUALIFIED = 14
    GUILD_DISCOVERY_REQUALIFIED = 15
    GUILD_DISCOVERY_GRACE_PERIOD_INITIAL_WARNING = 16
    GUILD_DISCOVERY_GRACE_PERIOD_FINAL_WARNING = 17
    THREAD_CREATED = 18
    REPLY = 19
    CHAT_INPUT_COMMAND = 20
    THREAD_STARTER_MESSAGE = 21
    GUILD_INVITE_REMINDER = 22
    CONTEXT_MENU_COMMAND = 23
    AUTO_MODERATION_ACTION = 24
    ROLE_SUBSCRIPTION_PURCHASE = 25
    INTERACTION_PREMIUM_UPSELL = 26
    STAGE_START = 27
    STAGE_END = 28
    STAGE_SPEAKER = 29
    STAGE_TOPIC = 31
    GUILD_APPLICATION_PREMIUM_SUBSCRIPTION = 32


class PartialMessage(BaseModel):
    id: Snowflake
    channel_id: Snowflake
    author: User
    content: str
    timestamp: ISOTimestamp
    edited_timestamp: ISOTimestamp | None
    tts: bool
    mention_everyone: bool
    mentions: list[User]
    mention_roles: list[Role]
    attachments: list[Attachment]
    embeds: list[Embed]
    reactions: list[Reaction]
    pinned: bool
    type: MessageType
