from dislord.discord.base import BaseModel
from dislord.discord.type import Snowflake


class RoleSubscriptionData(BaseModel):
    role_subscription_listing_id = Snowflake
    tier_name: str
    total_months_subscribed = int
    is_renewal = bool

