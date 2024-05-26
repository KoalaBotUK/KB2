from enum import IntEnum

from dislord.discord.base import BaseModel
from dislord.discord.resources.guild.integration_account import IntegrationAccount
from dislord.discord.resources.guild.integration_application import IntegrationApplication
from dislord.discord.resources.user.user import User
from dislord.discord.type import Snowflake, Missing, ISOTimestamp


class IntegrationExpireBehavior(IntEnum):
    REMOVE_ROLE = 0
    KICK = 1


class Integration(BaseModel):
    id: Snowflake
    name: str
    type: str
    enabled: bool
    syncing: bool | Missing
    role_id: Snowflake | Missing
    enable_emoticons: bool | Missing
    expire_behavior: IntegrationExpireBehavior | Missing
    expire_grace_period: int | Missing
    user: User | Missing
    account: IntegrationAccount
    synced_at: ISOTimestamp | Missing
    subscriber_count: int | Missing
    revoked: bool | Missing
    application: IntegrationApplication | Missing
    # scopes: list[Oauth2Scopes] | Missing FIXME
