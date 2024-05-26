from dislord.discord.base import BaseModel
from dislord.discord.resources.application.enums import ApplicationIntegrationType
from dislord.discord.resources.application.flags import ApplicationFlags
from dislord.discord.type import Snowflake, Missing


class InstallParams(BaseModel):
    scopes: list[str]
    permissions: str


class ApplicationIntegrationTypeConfiguration(BaseModel):
    oauth2_install_params: InstallParams | Missing


class PartialApplication(BaseModel):
    id: Snowflake
    name: str
    icon: str | None
    description: str
    bot_public: bool
    bot_require_code_grant: bool
    summary: str  # depreciated v11
    verify_key: str
    # team: Team | None FIXME


class Application(PartialApplication):
    rpc_origins: list[str] | Missing
    # bot: User | Missing FIXME
    terms_of_service_url: str | Missing
    privacy_policy_url: str | Missing
    # owner: User | Missing FIXME
    guild_id: Snowflake | Missing
    # guild: PartialGuild | Missing FIXME
    primary_sku_id: Snowflake | Missing
    slug: str | Missing
    cover_image: str | Missing
    flags: ApplicationFlags | Missing
    approximate_guild_count: int | Missing
    redirect_uris: list[str] | Missing
    interactions_endpoint_url: str | Missing
    role_connections_verification_url: str | Missing
    tags: list[str] | Missing
    install_params: InstallParams | Missing
    integration_types_config: dict[ApplicationIntegrationType, ApplicationIntegrationTypeConfiguration] | Missing
    custom_install_url: str | Missing