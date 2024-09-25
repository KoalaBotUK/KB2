from dislord.discord.resources.guild.guild import PartialGuild
from dislord.discord.resources.user.user import User
from dislord.types import ObjDict
from dislord.discord.resources.application.enums import ApplicationIntegrationType
from dislord.discord.resources.application.flags import ApplicationFlags
from dislord.discord.reference import Snowflake, Missing


class InstallParams(ObjDict):
    scopes: list[str]
    permissions: str


class ApplicationIntegrationTypeConfiguration(ObjDict):
    oauth2_install_params: InstallParams | Missing = None


class PartialApplication(ObjDict):
    id: Snowflake
    name: str
    icon: str | None = None
    description: str
    bot_public: bool
    bot_require_code_grant: bool
    summary: str  # depreciated v11
    verify_key: str
    # team: Team | None FIXME


class Application(PartialApplication):
    rpc_origins: list[str] | Missing = None
    bot: User | Missing = None
    terms_of_service_url: str | Missing = None
    privacy_policy_url: str | Missing = None
    owner: User | Missing = None
    guild_id: Snowflake | Missing = None
    guild: PartialGuild | Missing = None
    primary_sku_id: Snowflake | Missing = None
    slug: str | Missing = None
    cover_image: str | Missing = None
    flags: ApplicationFlags | Missing = None
    approximate_guild_count: int | Missing = None
    redirect_uris: list[str] | Missing = None
    interactions_endpoint_url: str | Missing = None
    role_connections_verification_url: str | Missing = None
    tags: list[str] | Missing = None
    install_params: InstallParams | Missing = None
    integration_types_config: dict[ApplicationIntegrationType, ApplicationIntegrationTypeConfiguration] | Missing = None
    custom_install_url: str | Missing = None
