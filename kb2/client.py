import threading
import time

from pydantic import TypeAdapter

import dislord
from dislord.discord.interactions.receiving_and_responding.interaction import Interaction
from dislord.discord.interactions.receiving_and_responding.interaction_response import InteractionResponse
from kb2 import env
from kb2.log import logger

RESPONSE_TIME_SLA_MS = 2000

logger.info("Starting client.py")


class KB2ApplicationClient(dislord.ApplicationClient):
    def __init__(self, *args, **kwargs):
        super().__init__(*args, **kwargs)

    class DeferThread(threading.Thread):
        def __init__(self, interaction: Interaction, defer_ts: int):
            super().__init__()
            self.interaction = interaction
            self.defer_ts = defer_ts
            self.respond = True

        def run(self):
            logger.debug("Auto Defer: %s", self.interaction)
            defer = client.get_callback_dto_and_args(self.interaction)[0].defer
            if defer:
                logger.debug("Auto Defer Sleeping: %s", self.defer_ts - time.time() * 1000)
                time.sleep((self.defer_ts - time.time() * 1000) / 1000)
                if self.respond:
                    client.interaction_callback(self.interaction, defer)  # DEFER()
            logger.debug("Auto Defer Done: %s", defer)

    def gateway_interact(self, interaction: Interaction, x_interact_ts: int):
        thread = None
        if x_interact_ts:
            defer_ts = x_interact_ts + RESPONSE_TIME_SLA_MS
            thread = KB2ApplicationClient.DeferThread(interaction, defer_ts)
            thread.start()

        interaction_resp = TypeAdapter(InteractionResponse).validate_python(self.interact(interaction).body)
        if thread is None or thread.is_alive():
            thread.respond = False
            self.interaction_callback(interaction, interaction_resp)
        else:
            self.edit_original_response(interaction.token, interaction_resp.data)


class OwnerCommandGroup(dislord.CommandGroup):
    def __init__(self, *args, **kwargs):
        self.guild_id = "1175756999040966656"
        super().__init__(*args, **kwargs)


client = KB2ApplicationClient(env.PUBLIC_KEY, env.BOT_TOKEN)
owner_group = OwnerCommandGroup(client, name="owner",
                                description="KoalaBot Owner Commands")  # TODO: set owner flag on dynamodb
