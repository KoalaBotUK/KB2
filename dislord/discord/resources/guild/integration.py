from enum import IntEnum

from dislord.types import ObjDict
from dislord.discord.resources.guild.integration_account import IntegrationAccount
from dislord.discord.resources.guild.integration_application import IntegrationApplication
from dislord.discord.resources.user.user import User
from dislord.discord.reference import Snowflake, Missing, ISOTimestamp


class IntegrationExpireBehavior(IntEnum):
    REMOVE_ROLE = 0
    KICK = 1


class Integration(ObjDict):
    id: Snowflake
    name: str
    type: str
    enabled: bool
    syncing: bool | Missing = None
    role_id: Snowflake | Missing = None
    enable_emoticons: bool | Missing = None
    expire_behavior: IntegrationExpireBehavior | Missing = None
    expire_grace_period: int | Missing = None
    user: User | Missing = None
    account: IntegrationAccount
    synced_at: ISOTimestamp | Missing = None
    subscriber_count: int | Missing = None
    revoked: bool | Missing = None
    application: IntegrationApplication | Missing = None
    # scopes: list[Oauth2Scopes] | Missing FIXME
