from dislord.discord.base import BaseModel
from dislord.discord.resources.application_role_connection_metadata.models import ApplicationRoleConnectionMetadata


class ApplicationRoleConnection(BaseModel):
    platform_name: str | None
    platform_username: str | None
    metadata: dict[ApplicationRoleConnectionMetadata, str]
