from fastapi import FastAPI, WebSocket

app = FastAPI()


@app.websocket("/ws")
async def websocket_endpoint(websocket: WebSocket):
    await websocket.accept()
    while True:
        msg = await websocket.receive_text()
        print(f"Message text was: {msg}")


if __name__ == '__main__':
    import uvicorn
    uvicorn.run(app, host="localhost", port=8765)
