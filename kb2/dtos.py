from pydantic import BaseModel

from kb2.errors import ErrorCodeException
from kb2.log import logger


class ErrorResponse(BaseModel):
    error: str
    message: str

    @staticmethod
    def from_exception(exception: Exception) -> "ErrorResponse":
        if isinstance(exception, ErrorCodeException):
            return ErrorResponse(error=exception.code, message=exception.message)
        else:
            logger.error("Unknown Exception for response", exc_info=exception)
            raise exception
