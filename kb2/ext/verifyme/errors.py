from enum import auto

from kb2.errors import ErrorCode


class VerifymeErrorCode(ErrorCode):
    INVALID_TOKEN = auto()
    INVALID_EMAIL = auto()
    LINK_EXISTS_SELF = auto()
    LINK_EXISTS_OTHER = auto()
