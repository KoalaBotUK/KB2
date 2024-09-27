from .client import ApplicationClient
from fastapi import FastAPI, Header, Request, Response
from pydantic import BaseModel
from typing import Annotated

from mangum import Mangum

app = FastAPI()
handler = None

__application_client: ApplicationClient


class InteractionsHeaders(BaseModel):
    x_signature_ed25519: str
    x_signature_timestamp: str


@app.post("/interactions") # FIXME: Add env overwrite
async def interactions_endpoint(interactions_headers: Annotated[InteractionsHeaders, Header()],
                                request: Request, response: Response):
    raw_request = await request.body()
    signature = interactions_headers.x_signature_ed25519
    timestamp = interactions_headers.x_signature_timestamp
    print(f"👉 Request: {raw_request}")
    response_data = __application_client.verified_interact(raw_request, signature, timestamp)
    response_data = response_data.as_server_response(response)
    print(f"🫴 Response: {response_data}")
    return response_data


def start_server(application_client, **kwargs):
    import uvicorn
    global __application_client
    __application_client = application_client
    uvicorn.run(app, host="0.0.0.0", port=8123)


def handler_singleton(**kwargs) -> Mangum:
    global handler
    if handler is None:
        app.root_path = kwargs.get("root_path")
        handler = Mangum(app)
    return handler


def serverless_handler(application_client, event, context, **kwargs):
    global __application_client
    __application_client = application_client
    return handler_singleton(**kwargs)(event, context)
