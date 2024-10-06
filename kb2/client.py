import dislord
from kb2 import env
from kb2.log import logger

logger.info("Starting client.py")

client = dislord.ApplicationClient(env.PUBLIC_KEY, env.BOT_TOKEN)


class OwnerCommandGroup(dislord.CommandGroup):
    def __init__(self, *args, **kwargs):
        self.guild_id = "1175756999040966656"
        super().__init__(*args, **kwargs)


owner_group = OwnerCommandGroup(client, name="owner",
                                description="KoalaBot Owner Commands")  # TODO: set owner flag on dynamodb
