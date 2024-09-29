import json
import time
from queue import Queue
from typing import Callable

from discord_interactions import verify_key, InteractionType
from pydantic import TypeAdapter

from .discord.interactions.application_commands.enums import ApplicationCommandType
from .discord.interactions.application_commands.models import ApplicationCommandOption
from .discord.interactions.receiving_and_responding.interaction import Interaction
from .discord.interactions.receiving_and_responding.interaction_callback import InteractionCallbackResponse
from .discord.interactions.receiving_and_responding.interaction_response import InteractionResponse, \
    MessagesInteractionCallbackData, InteractionCallbackType
from .discord.reference import Snowflake, Missing
from .discord.resources.application.models import Application
from .api import DiscordApi
from .discord.resources.channel.channel import Channel
from .discord.resources.channel.message import Message, MessageFlags
from .discord.resources.guild.guild import PartialGuild, Guild
from .discord.resources.user.user import User
from .error import DiscordApiException
from .log import logger
from .model.api import HttpResponse, HttpUnauthorized, HttpOk
from .model.commands import ApplicationCommand


class ApplicationClient:
    _public_key: str
    _api: DiscordApi
    _commands: dict[Snowflake, dict[str, ApplicationCommand]] = {}
    _command_callbacks: dict[str, Callable] = {}
    _component_callbacks: dict[str, Callable] = {}
    _application: Application = Missing()
    _guilds: list[Guild] = Missing()
    _deferred_queue: Queue[Interaction] = Queue()

    def __init__(self, public_key, bot_token):
        self._public_key = public_key
        self._api = DiscordApi(self, bot_token)

    def verified_interact(self, raw_request, signature, timestamp) -> HttpResponse:
        if signature is None or timestamp is None or not verify_key(
                raw_request, signature, timestamp, self._public_key):
            return HttpUnauthorized('Bad request signature')
        return self.interact(TypeAdapter(Interaction).validate_json(raw_request))

    def interact(self, interaction: Interaction) -> HttpResponse:
        match interaction.type:
            case InteractionType.PING:  # PING
                response_data = InteractionResponse.pong()  # PONG
            case InteractionType.APPLICATION_COMMAND:
                data = interaction.data
                command_name = data.name
                kwargs = {}
                for option in data.options or []:
                    kwargs[option.name] = option.value
                response_data = self._command_callbacks[command_name](interaction=interaction, **kwargs)
            case InteractionType.MESSAGE_COMPONENT:
                data = interaction.data
                response_data = self._component_callbacks[data.custom_id.split("$")[0]](interaction=interaction)
            case _:
                raise DiscordApiException(DiscordApiException.UNKNOWN_INTERACTION_TYPE.format(interaction.type))

        return HttpOk(json.loads(response_data.model_dump_json()), headers={"Content-Type": "application/json"})

    def verified_defer_interact(self, raw_request, signature, timestamp) -> HttpResponse:
        if signature is None or timestamp is None or not verify_key(
                raw_request, signature, timestamp, self._public_key):
            return HttpUnauthorized('Bad request signature')
        return self.defer_interact(TypeAdapter(Interaction).validate_json(raw_request))

    def defer_interact(self, interaction: Interaction) -> HttpResponse:
        match interaction.type:
            case InteractionType.PING:  # PING
                response_data = InteractionResponse.pong()  # PONG
            case _:
                self._deferred_queue.put(interaction)
                response_data = InteractionResponse(type=InteractionCallbackType.DEFERRED_CHANNEL_MESSAGE_WITH_SOURCE,
                                                    data=MessagesInteractionCallbackData(flags=MessageFlags.EPHEMERAL))
        return HttpOk(json.loads(response_data.model_dump_json()), headers={"Content-Type": "application/json"})

    def defer_queue_interact(self, sleep: int = 0):
        interaction = self._deferred_queue.get(timeout=5)
        logger.debug(f"DEFER QUEUE REQUEST: {interaction}")
        time.sleep(sleep)
        interact_http_response: HttpResponse = self.interact(interaction)
        logger.debug(f"DEFER QUEUE RESPONSE: {interact_http_response.body}")
        interact_response: MessagesInteractionCallbackData = (TypeAdapter(MessagesInteractionCallbackData)
                                                              .validate_python(interact_http_response.body["data"]))
        if interact_response.flags is None:
            interact_response.flags = 0
        self.edit_original_response(interaction.token, interact_response)

    def interaction_callback(self, interaction: Interaction,
                             interaction_response: InteractionResponse) -> InteractionCallbackResponse:
        return self._api.post(f"/interactions/{self.application.id}/{interaction.token}/callback",
                              interaction_response,
                              type_hint=InteractionCallbackResponse)

    def edit_original_response(self, interaction_token: Snowflake, response: MessagesInteractionCallbackData):
        self._api.patch(f"/webhooks/{self.application.id}/{interaction_token}/messages/@original", response,
                        type_hint=Message)

    def add_command(self, command: ApplicationCommand, callback: Callable):
        if self._commands.get(command.guild_id) is None:
            self._commands[command.guild_id] = {}
        self._command_callbacks[command.name] = callback
        self._commands.get(command.guild_id)[command.name] = command

    def command(self, *, name, description, dm_permission=True, nsfw=False, guild_ids: list[Snowflake] = None,
                options: list[ApplicationCommandOption] = None):
        if guild_ids is None:
            guild_ids = ["ALL"]

        def decorator(func):
            for guild_id in guild_ids:
                if guild_id == "ALL":
                    guild_id = None
                self.add_command(
                    ApplicationCommand(name=name, description=description,
                                       type=ApplicationCommandType.CHAT_INPUT,
                                       dm_permission=dm_permission, nsfw=nsfw,
                                       guild_id=guild_id, options=options, client=self), func)
            return func

        return decorator

    def component_callback(self, name: str):
        def decorator(func):
            self._component_callbacks[name] = func
            return func

        return decorator

    @property
    def application(self):
        if self._application is Missing():
            self._application = self.get_application()
        return self._application

    @property
    def guilds(self) -> list[Guild]:
        if self._guilds is Missing():
            self._guilds = self._get_guilds()
        return self._guilds

    def get_application(self):
        return self._api.get("/applications/@me", type_hint=Application)

    def sync_commands(self, guild_id: Snowflake = None, guild_ids: list[Snowflake] = None,
                      application_id: Snowflake = None):
        if guild_ids:
            for g_id in guild_ids:
                self.sync_commands(guild_id=g_id, application_id=application_id)

        registered_commands = self._get_commands(guild_id)
        client_commands = self._commands.get(guild_id)
        missing_commands = list(client_commands.values()) if client_commands else []
        for registered_command in registered_commands:
            if registered_command not in missing_commands:
                self._delete_commands(command_id=registered_command.id, guild_id=guild_id,
                                      application_id=registered_command.application_id)
            else:
                missing_commands.remove(registered_command)

        for missing_command in missing_commands:
            self._register_command(missing_command, guild_id=guild_id, application_id=application_id)

    def _get_commands(self, guild_id: Snowflake = None, application_id: Snowflake = None,
                      with_localizations: bool = None) -> list[ApplicationCommand]:
        endpoint = f"/applications/{application_id if application_id else self.application.id}"
        if guild_id:
            endpoint += f"/guilds/{guild_id}"

        params = {}
        if with_localizations is not None:
            params["with_localizations"] = with_localizations

        return self._api.get(f"{endpoint}/commands", params=params, type_hint=list[ApplicationCommand])

    def _delete_commands(self, command_id: Snowflake,
                         guild_id: Snowflake = None, application_id: Snowflake = None) -> None:
        endpoint = f"/applications/{application_id if application_id else self.application.id}"
        if guild_id:
            endpoint += f"/guilds/{guild_id}"

        self._api.delete(f"{endpoint}/commands/{command_id}")

    def _register_command(self, application_command: ApplicationCommand,
                          guild_id: Snowflake = None, application_id: Snowflake = None) -> ApplicationCommand:
        endpoint = f"/applications/{application_id if application_id else self.application.id}"
        if guild_id:
            endpoint += f"/guilds/{guild_id}"
        return self._api.post(f"{endpoint}/commands", application_command,
                              type_hint=ApplicationCommand)

    def get_user(self, user_id=None) -> User:
        return self._api.get(f"/users/{user_id if user_id else '@me'}", type_hint=User)

    def get_guild(self, guild_id) -> Guild:
        return self._api.get(f"/guilds/{guild_id}", type_hint=Guild)

    def _get_guilds(self) -> list[Guild]:
        return self._api.get("/users/@me/guilds", type_hint=list[PartialGuild])

    def get_channel(self, channel_id) -> list[Channel]:
        return self._api.get(f"/channels/{channel_id}", type_hint=list[Channel])
