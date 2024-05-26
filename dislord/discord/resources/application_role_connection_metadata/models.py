from dislord.discord.base import BaseModel
from dislord.discord.locale import Locale
from dislord.discord.resources.application_role_connection_metadata.enums import ApplicationRoleConnectionMetadataType
from dislord.discord.type import Missing


class ApplicationRoleConnectionMetadata(BaseModel):
    type: ApplicationRoleConnectionMetadataType
    key: str
    name: str
    name_localizations: dict[Locale, str] | Missing | None
    description: str
    description_localizations: dict[Locale, str] | Missing | None