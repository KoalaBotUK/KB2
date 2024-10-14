from typing import Annotated

from fastapi import FastAPI, Depends
from mangum import Mangum

from kb2 import env
from kb2.jwt import JWTBearer

app = FastAPI()

auth = JWTBearer(env.JWKS_URL)


async def auth_has_scope(scope: str, jwt_auth: Annotated[dict, Depends(auth)]):
    for s in scope.split(" "):
        assert s in jwt_auth.get("scope").split(" ")


async def auth_is_owner(jwt_auth: Annotated[dict, Depends(auth)]):
    await auth_has_scope("owner", jwt_auth)


@app.get("/ping", dependencies=[Depends(auth_is_owner)])
async def ping():
    return {"message": "Pong!"}


handler = Mangum(app, api_gateway_base_path=env.API_GATEWAY_BASE_PATH)

if __name__ == '__main__':
    import uvicorn

    uvicorn.run(app, host="0.0.0.0", port=8000)
