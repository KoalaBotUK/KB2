import dislord
from kb2 import env
from kb2.log import logger

logger.info("Starting client.py")

client = dislord.ApplicationClient(env.PUBLIC_KEY, env.BOT_TOKEN)

owner_group = dislord.CommandGroup(client, name="owner", description="KoalaBot Owner Commands",
                                   guild_id="1175756999040966656")  # TODO: set owner flag on dynamodb

