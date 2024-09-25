from dislord.types import ObjDict
from dislord.discord.resources.application_role_connection_metadata.models import ApplicationRoleConnectionMetadata


class ApplicationRoleConnection(ObjDict):
    platform_name: str | None = None
    platform_username: str | None = None
    metadata: dict[ApplicationRoleConnectionMetadata, str]
