import os
from threading import Thread

import dislord
from dislord.aws import lambda_runtime
from dislord.defer import DeferredThread
from kb2 import env
from kb2.log import logger

client = dislord.ApplicationClient(env.PUBLIC_KEY, env.BOT_TOKEN)

owner_group = dislord.CommandGroup(client, name="owner", description="KoalaBot Owner Commands",
                                   guild_id="1175756999040966656")  # TODO: set owner flag on dynamodb


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


runtime_thread: DeferredThread
if os.environ.get('AWS_LAMBDA_RUNTIME_API'):
    runtime_thread = lambda_runtime.LambdaDeferredThread.instance(client)
else:
    runtime_thread = DeferredThread.instance(client)
runtime_thread.start()

if __name__ == '__main__':  # Not needed if using serverless
    # client.sync_commands()
    # client.sync_commands(guild_ids=[g.id for g in client.guilds])
    dislord.server.start_server(client, port=8123)
