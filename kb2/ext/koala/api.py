from fastapi import APIRouter, Depends

from kb2.api import app
from kb2.ext.koala import mappers
from kb2.jwt import auth
from kb2.ext.koala.models import Guild

router = APIRouter(prefix="/koala")


@router.get("/guild/{guild_id}", dependencies=[Depends(auth.is_owner)])
async def get_guild(guild_id: str):
    return mappers.guild_to_dto(Guild.get(guild_id))

app.include_router(router)
