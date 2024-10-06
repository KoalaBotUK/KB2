from typing import Callable, Self

from dislord.discord.interactions.application_commands.models import ApplicationCommand as ApplicationCommandPayload, \
    ApplicationCommandOption
from dislord.discord.interactions.receiving_and_responding.interaction_response import InteractionResponse, \
    InteractionCallbackType
from dislord.discord.interactions.receiving_and_responding.message_interaction import InteractionType
from dislord.discord.reference import Missing
from dislord.types import ObjDict


class ApplicationCommand(ApplicationCommandPayload):

    def __eq__(self, other):
        eq_list = ['guild_id', 'name', 'description', 'type', 'name_localization', 'description_localizations',
                   'options', 'default_member_permissions', 'dm_permission', 'default_permission', 'nsfw']
        result = True
        for eq_attr in eq_list:
            self_attr = getattr(self, eq_attr, None)
            other_attr = getattr(other, eq_attr, None)
            result = result and \
                     (self_attr == other_attr or self_attr is other_attr)  # compare_missing_none(self_attr, other_attr)
        return result

    def __post_init__(self):
        if self.guild_id is not None and self.guild_id is not Missing:
            self.dm_permission = None


class CallbackDTO(ObjDict):
    interaction_type: InteractionType
    key: str
    callback: Callable


class PingCallbackDTO(CallbackDTO):
    interaction_type: InteractionType = InteractionType.PING
    key: str = "ping"
    callback: Callable = lambda interaction: InteractionResponse.pong()


class CommandCallbackDTO(CallbackDTO):
    interaction_type: InteractionType = InteractionType.APPLICATION_COMMAND
    command: ApplicationCommand | ApplicationCommandOption
    defer: InteractionResponse | None = None


class GroupCallbackDTO(CallbackDTO):
    interaction_type: InteractionType = InteractionType.APPLICATION_COMMAND
    command: ApplicationCommand | ApplicationCommandOption
    sub_command_callbacks: dict[InteractionType, dict[str, CommandCallbackDTO | Self]] | None = None

    def raise_callback_not_implemented(*args, **kwargs):
        raise NotImplementedError("callback not implemented")

    callback: Callable = raise_callback_not_implemented


class ComponentCallbackDTO(CallbackDTO):
    interaction_type: InteractionType = InteractionType.MESSAGE_COMPONENT
    defer: InteractionResponse | None = InteractionResponse(type=InteractionCallbackType.DEFERRED_UPDATE_MESSAGE)
