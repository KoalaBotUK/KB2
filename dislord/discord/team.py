from enum import Enum

from dislord.discord.base import BaseModel
from dislord.discord.resources.user.user import PartialUser
from dislord.discord.type import Snowflake


class TeamMemberRoleType(Enum):
    OWNER = None
    ADMIN = "admin"
    DEVELOPER = "developer"
    READ_ONLY = "read_only"


class TeamMember(BaseModel):
    membership_state: int
    team_id: Snowflake
    user: PartialUser
    role: TeamMemberRoleType


class Team(BaseModel):
    icon: str | None
    id: Snowflake
    members: list[TeamMember]
    name: str
    owner_user_id: Snowflake
