from .client import ApplicationClient
from fastapi import FastAPI, Header, Request, Response
from pydantic import BaseModel
from typing import Annotated

from mangum import Mangum

from .discord.interactions.receiving_and_responding.interaction import Interaction

app = FastAPI()


__application_client: ApplicationClient

class InteractionsHeaders(BaseModel):
    x_signature_ed25519: str
    x_signature_timestamp: str


@app.post("/interactions")
async def interactions_endpoint(interactions_headers: Annotated[InteractionsHeaders, Header()],
                                request: Request, response: Response):
    # raw_request = request.data
    # request_json = request.json
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


handler = Mangum(app)

def serverless_handler(application_client, event, context):
    global __application_client
    __application_client = application_client
    return handler(event, context)
    # if event['httpMethod'] == "POST":
    #     print(f"🫱 Full Event: {event}")
    #     raw_request = event["body"]
    #     print(f"👉 Request: {raw_request}")
    #     raw_headers = event["headers"]
    #     signature = raw_headers.get('x-signature-ed25519')
    #     timestamp = raw_headers.get('x-signature-timestamp')
    #     response = self.verified_interact(raw_request, signature, timestamp).as_serverless_response()
    #     print(f"🫴 Response: {response}")
    #     return response

