from dislord.server import start_server
from kb2.client import client

if __name__ == '__main__':  # Not needed if using serverless
    # client.sync_commands()
    # client.sync_commands(guild_ids=[g.id for g in client.guilds])
    start_server(client, port=8123)
