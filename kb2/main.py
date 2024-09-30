import dislord
import kb2.env as env
from kb2.client import client
from kb2.log import logger


logger.info("Starting main.py")
client.start_ws_client()


def serverless_handler(event, context):  # Not needed if using server
    from kb2 import ext
    ext.register_all()
    logger.info(f"\nevent: {event}\ncontext: {context}")
    response = dislord.server.serverless_handler(client, event, context, root_path=env.API_GATEWAY_BASE_PATH)
    logger.info(f"\nresponse: {response}")
    return response


def sync_serverless_handler(event, context):
    client.sync_commands()
    client.sync_commands(guild_ids=[g.id for g in client.guilds])
    return {"statusCode": 200}


if __name__ == '__main__':  # Not needed if using serverless
    # client.sync_commands()
    # client.sync_commands(guild_ids=[g.id for g in client.guilds])
    dislord.server.start_server(client, port=8123)
