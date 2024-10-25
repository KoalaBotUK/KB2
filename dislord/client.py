import json
from queue import Queue
from types import NoneType
from typing import Callable

from discord_interactions import verify_key
from pydantic import TypeAdapter

from .api import DiscordApi
from .discord.interactions.application_commands.enums import ApplicationCommandType, ApplicationCommandOptionType
from .discord.interactions.application_commands.models import ApplicationCommandOption
from .discord.interactions.receiving_and_responding.interaction import Interaction
from .discord.interactions.receiving_and_responding.interaction_callback import InteractionCallbackResponse
from .discord.interactions.receiving_and_responding.interaction_response import InteractionResponse, \
    MessagesInteractionCallbackData
from .discord.interactions.receiving_and_responding.message_interaction import InteractionType
from .discord.reference import Snowflake, Missing
from .discord.resources.application.models import Application
from .discord.resources.channel.channel import Channel
from .discord.resources.channel.message import Message, MessageFlags
from .discord.resources.guild.guild import PartialGuild, Guild
from .discord.resources.user.user import User
from .error import DiscordApiException
from .log import logger
from .model.api import HttpResponse, HttpUnauthorized, HttpOk
from .model.commands import ApplicationCommand, PingCallbackDTO, CommandCallbackDTO, ComponentCallbackDTO, CallbackDTO


class ApplicationClient:
    _public_key: str
    _api: DiscordApi
    _callbacks: dict[InteractionType, dict[str, CallbackDTO]]
    _application: Application = Missing()
    _guilds: list[Guild] = Missing()
    _deferred_queue: Queue[Interaction] = Queue()

    def __init__(self, public_key, bot_token):
        self._public_key = public_key
        self._api = DiscordApi(self, bot_token)
        self._callbacks = {k: {} for k in InteractionType}
        self._callbacks[InteractionType.PING] = {"ping": PingCallbackDTO()}

    def verified_interact(self, raw_request, signature, timestamp) -> HttpResponse:
        if signature is None or timestamp is None or not verify_key(
                raw_request, signature, timestamp, self._public_key):
            return HttpUnauthorized('Bad request signature')
        return self.interact(TypeAdapter(Interaction).validate_json(raw_request))

    def get_callback_dto_and_args(self, interaction: Interaction) -> (CallbackDTO, dict[str, any]):
        interaction_options = []
        match interaction.type:
            case InteractionType.PING:  # PING
                key = "ping"
            case InteractionType.APPLICATION_COMMAND:
                key = interaction.data.name
                interaction_options = interaction.data.options
            case InteractionType.MESSAGE_COMPONENT:
                key = interaction.data.custom_id.split("$")[0]
            case _:
                raise DiscordApiException(DiscordApiException.UNKNOWN_INTERACTION_TYPE.format(interaction.type))

        callback_dto = self._callbacks[interaction.type][key]

        while (interaction_options and interaction_options[0].type in
               [ApplicationCommandOptionType.SUB_COMMAND_GROUP, ApplicationCommandOptionType.SUB_COMMAND]):
            key = interaction.data.options[0].name
            callback_dto = callback_dto.sub_command_callbacks[interaction.type][key]
            interaction_options = interaction_options[0].options

        kwargs = {}
        for option in interaction_options or []:
            kwargs[option.name] = option.value

        return callback_dto, kwargs

    def defer(self, interaction: Interaction) -> HttpResponse:
        callback_dto, _ = self.get_callback_dto_and_args(interaction)
        if callback_dto.defer:
            return HttpOk(json.loads(callback_dto.defer.model_dump_json()), headers={"Content-Type": "application/json"})
        else:
            return HttpOk("{}", headers={"Content-Type": "application/json"})

    def interact(self, interaction: Interaction) -> HttpResponse:
        callback_dto, kwargs = self.get_callback_dto_and_args(interaction)
        response_data = callback_dto.callback(interaction=interaction, **kwargs)

        return HttpOk(json.loads(response_data.model_dump_json()), headers={"Content-Type": "application/json"})

    def defer_queue_interact(self):
        interaction = self._deferred_queue.get()
        logger.debug(f"DEFER QUEUE REQUEST: {interaction}")
        interact_http_response: HttpResponse = self.interact(interaction)
        logger.debug(f"DEFER QUEUE RESPONSE: {interact_http_response.body}")
        interact_response: MessagesInteractionCallbackData = (TypeAdapter(MessagesInteractionCallbackData)
                                                              .validate_python(interact_http_response.body["data"]))
        if interact_response.flags is None:
            interact_response.flags = MessageFlags.NONE
        self.edit_original_response(interaction.token, interact_response)

    def interaction_callback(self, interaction: Interaction,
                             interaction_response: InteractionResponse) -> InteractionCallbackResponse:
        return self._api.post(f"/interactions/{interaction.id}/{interaction.token}/callback",
                              interaction_response,
                              type_hint=None)

    def edit_original_response(self, interaction_token: Snowflake, response: MessagesInteractionCallbackData):
        self._api.patch(f"/webhooks/{self.application.id}/{interaction_token}/messages/@original", response,
                        type_hint=Message)

    def add_callback(self, callback_dto: CallbackDTO):
        self._callbacks[callback_dto.interaction_type][callback_dto.key] = callback_dto

    def command(self, *, name, description, dm_permission=True, nsfw=False, guild_ids: list[Snowflake] = None,
                options: list[ApplicationCommandOption] = None, defer: InteractionResponse | None = None):
        if guild_ids is None:
            guild_ids = ["ALL"]

        def decorator(func):
            for guild_id in guild_ids:
                if guild_id == "ALL":
                    guild_id = None

                self.add_callback(
                    CommandCallbackDTO(
                        key=name,
                        command=ApplicationCommand(name=name, description=description,
                                                   type=ApplicationCommandType.CHAT_INPUT,
                                                   dm_permission=dm_permission, nsfw=nsfw,
                                                   guild_id=guild_id, options=options, client=self),
                        callback=func,
                        defer=defer
                    )
                )

            return func

        return decorator

    def component_callback(self, name: str, defer: InteractionResponse | None = None):
        def decorator(func):
            self.add_callback(ComponentCallbackDTO(key=name, callback=func, defer=defer))
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
        missing_commands = [dto.command for dto in self._callbacks[InteractionType.APPLICATION_COMMAND].values()
                            if dto.command.guild_id == guild_id]
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
