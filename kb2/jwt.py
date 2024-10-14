import httpx
from fastapi import HTTPException, Request
from fastapi.security import HTTPBearer, HTTPAuthorizationCredentials
from jose import jwt
from starlette.status import HTTP_403_FORBIDDEN

from kb2 import env
from kb2.log import logger


class JWTBearer(HTTPBearer):
    def __init__(self, jwks_url: str, auto_error: bool = True):
        super().__init__(auto_error=auto_error)
        logger.debug("Getting JWKS from %s", jwks_url)
        self.jwks = httpx.get(jwks_url).json()
        logger.debug("Got JWKS: %s", self.jwks)

    async def __call__(self, request: Request) -> dict | None:
        credentials: HTTPAuthorizationCredentials = await super().__call__(request)

        if not credentials:
            return

        if credentials.scheme != "Bearer":
            raise HTTPException(
                status_code=HTTP_403_FORBIDDEN, detail="Wrong authentication method"
            )
        try:
            return jwt.decode(credentials.credentials, self.jwks, audience="kb2", issuer="temp.auther.koalabot.uk")
        except (jwt.JWTError, jwt.ExpiredSignatureError, jwt.JWTClaimsError) as e:
            logger.exception("JWT Error: %s", e, exc_info=e)
            raise HTTPException(
                status_code=HTTP_403_FORBIDDEN, detail="Invalid token"
            )

    async def assert_scope(self, request: Request, scope: str):
        jwt_auth = await self(request)
        if not jwt_auth:
            return False
        if scope not in jwt_auth.get("scope").split(" "):
            raise HTTPException(
                status_code=HTTP_403_FORBIDDEN, detail="Incorrect scope"
            )

    async def is_owner(self, request: Request):
        await self.assert_scope(request, "owner")

auth = JWTBearer(env.JWKS_URL)