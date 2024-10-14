from fastapi import FastAPI, Depends

from kb2.jwt import auth

app = FastAPI()


@app.get("/ping", dependencies=[Depends(auth.is_owner)])
async def ping():
    return {"message": "Pong!"}


