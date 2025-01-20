from enum import StrEnum, auto


class ErrorCode(StrEnum):
    pass

class KoalaErrorCode(ErrorCode):
    UNAUTHORIZED = auto()

class ErrorCodeException(Exception):
    def __init__(self, code: ErrorCode, message: str):
        self.code = code
        self.message = message
