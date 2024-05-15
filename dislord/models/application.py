from dataclasses import dataclass
from typing import Optional

from .base import BaseModel
from .guild import PartialGuild
from .type import Snowflake
from .user import User


@dataclass
class Application(BaseModel):
    id: Snowflake
    name: str
    icon: Optional[str]
    description: str
    rpc_origins: Optional[list[str]]
    bot_public: bool
    bot_require_code_grant: bool
    bot: Optional[User]
    terms_of_service_url: Optional[str]
    privacy_policy_url: Optional[str]
    owner: Optional[User]
    summary: str  # depreciated v11
    verify_key: str
    # team: Optional[Team] FIXME
    guild: Optional[PartialGuild]
    primary_sku_id: Optional[Snowflake]
    slug: Optional[str]
    cover_image: Optional[str]
    flags: Optional[int]
    approximate_guild_count: Optional[int]
    redirect_uris: Optional[list[str]]
    interactions_endpoint_url: Optional[str]
    role_connections_verification_url: Optional[str]
    tags: Optional[list[str]]
    # install_params: Optional[InstallParams] FIXME
    custom_install_url: Optional[str]
