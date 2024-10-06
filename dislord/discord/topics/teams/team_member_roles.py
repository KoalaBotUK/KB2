from enum import StrEnum


class TeamMemberRoleType(StrEnum):
    OWNER = None
    ADMIN = "admin"
    DEVELOPER = "developer"
    READ_ONLY = "read_only"
