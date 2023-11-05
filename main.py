import os
import random
import string

import dislord
from dislord.models.command import ApplicationCommandOption, ApplicationCommandOptionType
from dislord.models.interaction import Interaction, InteractionResponse

PUBLIC_KEY = os.environ.get("PUBLIC_KEY")
BOT_TOKEN = os.environ.get("BOT_TOKEN")

client = dislord.ApplicationClient(PUBLIC_KEY, BOT_TOKEN)


@client.command(name="verify", description="Verify your email with Koala",
                options=[
                    ApplicationCommandOption.from_kwargs(name="email", description="Your email to be verified",
                                                         type=ApplicationCommandOptionType.STRING, required=True,
                                                         client=client)])
def verify(interaction: Interaction, email: str):
    token = ''.join(random.choice(string.ascii_letters) for _ in range(8))
    print(token)
    # db.save(email, token)
    return InteractionResponse.message(content="Please verify yourself using the command you have been emailed")


@client.command(name="confirm", description="Send token to complete email verification",
                options=[
                    ApplicationCommandOption.from_kwargs(name="token", description="Token you have been emailed",
                                                         type=ApplicationCommandOptionType.STRING, required=True,
                                                         client=client)])
def confirm(interaction: Interaction, token: str):
    token = ''.join(random.choice(string.ascii_letters) for _ in range(8))
    print(token)
    success = True
    # success = db.check(email, token)
    if success:
        content = "Verification Successful"
    else:
        content = "Could not verify with token provided"
    return InteractionResponse.message(content=content)


def serverless_handler(event, context):  # Not needed if using server
    return client.serverless_handler(event, context)


def sync_serverless_handler(event, context):
    client.sync_commands()
    client.sync_commands(guild_ids=[g.id for g in client.guilds])
    return {"statusCode": 200}


if __name__ == '__main__':  # Not needed if using serverless
    client.sync_commands()
    dislord.server.start_server(client, host='0.0.0.0', debug=True, port=8123)
