from typing import Annotated

from fastapi import FastAPI, Depends, Header
from starlette.middleware.cors import CORSMiddleware
from starlette.middleware import Middleware
from starlette.responses import Response

from dislord.discord.interactions.receiving_and_responding.interaction import Interaction
from kb2.client import client
from kb2.middleware import ErrorCodeResponseMiddleware
from kb2.security import jwt_auth
from kb2.log import logger

middleware = [
    Middleware(CORSMiddleware,
               allow_origins=["*"],
               allow_credentials=True,
               allow_methods=["*"],
               allow_headers=["*"]),
    Middleware(ErrorCodeResponseMiddleware)
]

app = FastAPI(middleware=middleware)


@app.get("/ping", dependencies=[Depends(jwt_auth.is_owner)])
async def ping():
    return {"message": "Pong!"}


@app.post("/gateway-interactions", dependencies=[Depends(jwt_auth.is_owner)])
async def gateway_interactions(interaction: Interaction, x_interact_ts: Annotated[int | None, Header()] = None):
    logger.debug("Gateway Interact: %s %s", x_interact_ts, interaction)
    client.gateway_interact(interaction, x_interact_ts)
    return Response(status_code=200)
