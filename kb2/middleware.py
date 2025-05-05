from starlette.middleware.base import BaseHTTPMiddleware, RequestResponseEndpoint
from starlette.requests import Request
from starlette.responses import Response
from starlette.status import HTTP_400_BAD_REQUEST

from kb2.dtos import ErrorResponse
from kb2.errors import ErrorCodeException
from kb2.log import logger

class ErrorCodeResponseMiddleware(BaseHTTPMiddleware):
    async def dispatch(self, request: Request, call_next: RequestResponseEndpoint) -> Response:
        try:
            return await call_next(request)
        except ErrorCodeException as e:
            logger.error("Error: %s", e, exc_info=e)
            return Response(ErrorResponse.from_exception(e).model_dump_json(), status_code=HTTP_400_BAD_REQUEST)
