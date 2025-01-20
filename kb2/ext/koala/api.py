from fastapi import APIRouter, Depends

from kb2.api import app
from kb2.client import client
from kb2.ext.koala import mappers
from kb2.security import jwt_auth
from kb2.ext.koala.models import Guild

router = APIRouter(prefix="/koala")


@router.get("/guild/{guild_id}", dependencies=[Depends(jwt_auth.is_owner)])
async def get_guild(guild_id: str):
    return mappers.guild_to_dto(Guild.get(guild_id))


@router.post("/sync", dependencies=[Depends(jwt_auth.is_owner)])
async def sync():
    client.sync_commands()
    client.sync_commands(guild_ids=[g.id for g in client.guilds])
    return {"message": "Synced commands for all guilds"}

app.include_router(router)
