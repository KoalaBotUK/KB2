from typing import Annotated

from fastapi import FastAPI, Depends, Header
from starlette.responses import Response

from dislord.discord.interactions.receiving_and_responding.interaction import Interaction
from kb2.client import client
from kb2.jwt import auth
from kb2.log import logger

app = FastAPI()


@app.get("/ping", dependencies=[Depends(auth.is_owner)])
async def ping():
    return {"message": "Pong!"}


@app.post("/gateway-interactions", dependencies=[Depends(auth.is_owner)])
async def gateway_interactions(interaction: Interaction, x_interact_ts: Annotated[int | None, Header()] = None):
    logger.debug("Gateway Interact: %s %s", x_interact_ts, interaction)
    client.gateway_interact(interaction, x_interact_ts)
    return Response(status_code=200)
