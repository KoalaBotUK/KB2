from typing import Annotated

from fastapi import APIRouter, Depends
from pydantic import BaseModel
from starlette.responses import Response
from starlette.status import HTTP_401_UNAUTHORIZED, HTTP_201_CREATED

from dislord.discord.topics.oauth2.shared_resources import AuthorizationInformation
from kb2.api import app
from kb2.dtos import ErrorResponse
from kb2.errors import ErrorCode, ErrorCodeException
from kb2.ext.verifyme import core, mappers
from kb2.ext.verifyme.dtos import LinkEmailRequest, LinkEmailResponse
from kb2.ext.verifyme.errors import VerifymeErrorCode
from kb2.ext.verifyme.validators import ValidatorFactory
from kb2.security import discord_auth

router = APIRouter(prefix="/verify")


@router.get("/email")
async def get_emails(auth_info: Annotated[AuthorizationInformation, Depends(discord_auth)]):
    emails = core.get_emails(auth_info.user.id)
    return [mappers.email_to_dto(email) for email in emails]


@router.post("/email/link")
async def link_email(req: LinkEmailRequest, resp: Response,
                     auth_info: Annotated[AuthorizationInformation, Depends(discord_auth)]):
    user_id = auth_info.user.id

    validator = ValidatorFactory().get(req.organization)

    if not validator.validate(req.token):
        raise ErrorCodeException(VerifymeErrorCode.INVALID_TOKEN, "Invalid third party token")

    core.add_email(user_id, validator.email, req.organization, req.overwrite)
    resp.status_code = HTTP_201_CREATED
    return LinkEmailResponse(email=validator.email)


class UnlinkEmailRequest(BaseModel):
    email: str


@router.post("/email/unlink")
async def unlink_email(req: UnlinkEmailRequest, auth_info: Annotated[AuthorizationInformation, Depends(discord_auth)]):
    core.delete_email(auth_info.user.id, req.email)


class SendEmailRequest(BaseModel):
    email: str


@router.post("/email/send")
async def send_email(req: SendEmailRequest, auth_info: Annotated[AuthorizationInformation, Depends(discord_auth)]):
    core.send_email(auth_info.user.id, req.email)


app.include_router(router)
