from typing import Callable

from dislord import ApplicationClient
from dislord.discord.interactions.application_commands.enums import ApplicationCommandOptionType, ApplicationCommandType
from dislord.discord.interactions.application_commands.models import ApplicationCommandOption
from dislord.discord.interactions.receiving_and_responding.interaction import Interaction
from dislord.discord.interactions.receiving_and_responding.interaction_response import InteractionResponse
from dislord.discord.interactions.receiving_and_responding.message_interaction import InteractionType
from dislord.discord.reference import Snowflake
from dislord.model.commands import CallbackDTO, CommandCallbackDTO, ApplicationCommand, \
    GroupCallbackDTO


class CommandGroup:
    client: ApplicationClient
    name: str
    description: str
    dm_permission: bool
    nsfw: bool
    guild_id: Snowflake
    parent: 'CommandGroup' = None
    _callbacks: dict[InteractionType, dict[str, CallbackDTO]]
    command_callbacks: dict[str, Callable] = {}

    def __init__(self, client: ApplicationClient, name: str, description: str, *,
                 dm_permission: bool = None, nsfw: bool = False, guild_id: Snowflake = None,
                 parent: 'CommandGroup' = None):
        self.client = client
        self.name = name
        self.description = description
        self.parent = parent
        self.dm_permission = True if guild_id is None else dm_permission
        self.nsfw = nsfw
        self.guild_id = guild_id
        self._callbacks = {k: {} for k in InteractionType}

        # self.update_parent()

    def update_parent(self):
        options = [c.command for c in self._callbacks[InteractionType.APPLICATION_COMMAND].values()]

        if self.parent:
            # Add to parent group callbacks
            self.parent.add_callback(
                GroupCallbackDTO(
                    key=self.name,
                    command=ApplicationCommandOption(name=self.name,
                                                     description=self.description,
                                                     type=ApplicationCommandOptionType.SUB_COMMAND_GROUP,
                                                     options=options),
                    sub_command_callbacks=self._callbacks
                )
            )
        else:
            # Add to client callbacks
            self.client.add_callback(
                GroupCallbackDTO(key=self.name,
                                 command=ApplicationCommand(
                                     name=self.name,
                                     description=self.description,
                                     type=ApplicationCommandType.CHAT_INPUT,
                                     dm_permission=self.dm_permission,
                                     nsfw=self.nsfw,
                                     guild_id=self.guild_id,
                                     options=options),
                                 sub_command_callbacks=self._callbacks)
            )

    def add_callback(self, callback_dto: CallbackDTO):
        self._callbacks[callback_dto.interaction_type][callback_dto.key] = callback_dto
        self.update_parent()

    def command(self, *, name, description, options: list[ApplicationCommandOption] = None,
                defer: InteractionResponse | None = None):
        def decorator(func):
            self.add_callback(
                CommandCallbackDTO(
                    key=name,
                    command=ApplicationCommandOption(name=name,
                                                     description=description,
                                                     type=ApplicationCommandOptionType.SUB_COMMAND,
                                                     options=options),
                    callback=func,
                    defer=defer
                )
            )
            return func

        return decorator

    def callback(self, interaction: Interaction, depth=1, **kwargs):
        command_data = interaction.data
        for _ in range(depth):
            command_data = command_data.options[0]
        command_name = command_data.name
        kwargs = {}
        for option in command_data.options:
            kwargs[option.name] = option.value

        callback = self._callbacks[InteractionType.APPLICATION_COMMAND][command_name].callback
        if callback.__name__ == "callback":
            return callback(interaction,
                            depth=depth + 1,
                            **kwargs)
        else:
            return callback(interaction, **kwargs)
