import dislord
from kb2 import env
from kb2.log import logger

client = dislord.ApplicationClient(env.PUBLIC_KEY, env.BOT_TOKEN)

owner_group = dislord.CommandGroup(client, name="owner", description="KoalaBot Owner Commands", guild_id="1175756999040966656")


def serverless_handler(event, context):  # Not needed if using server
    logger.info(f"\nevent: {event}\ncontext: {context}")
    return dislord.server.serverless_handler(client, event, context, api_gateway_base_path=env.API_GATEWAY_BASE_PATH)


def sync_serverless_handler(event, context):
    client.sync_commands()
    client.sync_commands(guild_ids=[g.id for g in client.guilds])
    return {"statusCode": 200}


if __name__ == '__main__':  # Not needed if using serverless
    client.sync_commands()
    client.sync_commands(guild_ids=[g.id for g in client.guilds])
    dislord.server.start_server(client)
