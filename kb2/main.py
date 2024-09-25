import dislord
from kb2 import env

client = dislord.ApplicationClient(env.PUBLIC_KEY, env.BOT_TOKEN)

owner_group = dislord.CommandGroup(client, name="owner", description="KoalaBot Owner Commands", guild_id="1175756999040966656")


def serverless_handler(event, context):  # Not needed if using server
    return dislord.server.serverless_handler(client, event, context)


def sync_serverless_handler(event, context):
    client.sync_commands()
    client.sync_commands(guild_ids=[g.id for g in client.guilds])
    return {"statusCode": 200}


if __name__ == '__main__':  # Not needed if using serverless
    client.sync_commands()
    client.sync_commands(guild_ids=[g.id for g in client.guilds])
    dislord.server.start_server(client)
