from contextlib import asynccontextmanager

from .client import ApplicationClient
from fastapi import FastAPI, Header, Request, Response
from pydantic import BaseModel
from typing import Annotated

from mangum import Mangum

from .log import logger

__application_client: ApplicationClient
__handler = None

app = FastAPI()


class InteractionsHeaders(BaseModel):
    x_signature_ed25519: str
    x_signature_timestamp: str


@app.post("/interactions")
async def interactions_endpoint(interactions_headers: Annotated[InteractionsHeaders, Header()],
                                request: Request, response: Response):
    raw_request = await request.body()
    signature = interactions_headers.x_signature_ed25519
    timestamp = interactions_headers.x_signature_timestamp
    logger.debug(f"👉 Request: {raw_request}")
    response_data = __application_client.verified_interact(raw_request, signature, timestamp)
    response_data = response_data.as_server_response(response)
    logger.debug(f"🫴 Response: {response_data}")
    return response_data


@app.post("/deferred-interactions")
async def interactions_endpoint(interactions_headers: Annotated[InteractionsHeaders, Header()],
                                request: Request, response: Response):
    raw_request = await request.body()
    signature = interactions_headers.x_signature_ed25519
    timestamp = interactions_headers.x_signature_timestamp
    logger.debug(f"👉 Request: {raw_request}")
    response_data = await __application_client.verified_defer_interact(raw_request, signature, timestamp)
    response_data = response_data.as_server_response(response)
    logger.debug(f"🫴 Response: {response_data}")
    return response_data


def start_server(application_client, *, port: int = 8000, **kwargs):
    import uvicorn
    global __application_client
    __application_client = application_client
    uvicorn.run(app, host="0.0.0.0", port=port)


def handler_singleton(**kwargs) -> Mangum:
    global __handler
    if __handler is None:
        app.root_path = kwargs.get("root_path")
        __handler = Mangum(app)
    return __handler


def serverless_handler(application_client, event, context, **kwargs):
    global __application_client
    __application_client = application_client
    return handler_singleton(**kwargs)(event, context)
