from datetime import datetime
from enum import StrEnum, auto

from pydantic import BaseModel


class Organization(StrEnum):
    MICROSOFT = auto()
    GOOGLE = auto()
    EMAIL = auto()


class EmailDto(BaseModel):
    email: str
    user_id: str
    organization: str
    active: bool
    date_added: datetime
    date_updated: datetime


class LinkEmailRequest(BaseModel):
    organization: Organization
    token: str
    overwrite: bool | None = None


class LinkEmailResponse(BaseModel):
    email: str
