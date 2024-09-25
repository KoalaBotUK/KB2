from typing import Callable

from dislord import ApplicationClient
from dislord.discord.interactions.application_commands.enums import ApplicationCommandOptionType, ApplicationCommandType
from dislord.discord.interactions.application_commands.models import ApplicationCommandOption, ApplicationCommand
from dislord.discord.interactions.receiving_and_responding.interaction import Interaction
from dislord.discord.reference import Snowflake


class CommandGroup:
    client: ApplicationClient
    name: str
    description: str
    dm_permission: bool
    nsfw: bool
    guild_id: Snowflake
    parent: 'CommandGroup' = None
    commands: dict[str, ApplicationCommandOption] = {}
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
        self.commands = {}

        self.update_parent()

    def update_parent(self):
        if not self.parent:
            return
        self.parent.add_command(ApplicationCommandOption(name=self.name, description=self.description,
                                                         type=ApplicationCommandOptionType.SUB_COMMAND_GROUP,
                                                         options=list(self.commands.values())), self.callback)

    def add_command(self, command: ApplicationCommandOption, callback: Callable):
        self.commands[command.name] = command
        self.command_callbacks[command.name] = callback

        self.client.add_command(command=ApplicationCommand(
            name=self.name, application_id=self.client.application.id, description=self.description, type=ApplicationCommandType.CHAT_INPUT,
            dm_permission=self.dm_permission, nsfw=self.nsfw, guild_id=self.guild_id,
            options=list(self.commands.values())), callback=self.callback)
        self.update_parent()

    def command(self, *, name, description, options: list[ApplicationCommandOption] = None):
        def decorator(func):
            self.add_command(ApplicationCommandOption(name=name, description=description,
                                                      type=ApplicationCommandOptionType.SUB_COMMAND,
                                                      options=options), func)
            return func

        return decorator

    def callback(self, interaction: Interaction, depth=1, **kwargs):
        command_data = interaction.data
        for i in range(depth):
            command_data = command_data.options[0]
        command_name = command_data.name
        kwargs = {}
        for option in command_data.options:
            kwargs[option.name] = option.value

        if self.command_callbacks[command_name].__name__ == "callback":
            return self.command_callbacks[command_name](interaction, depth=depth+1, **kwargs)
        else:
            return self.command_callbacks[command_name](interaction, **kwargs)
