

from dislord.discord.interactions.application_commands.models import ApplicationCommand as ApplicationCommandPayload
from dislord.discord.reference import  Missing


class ApplicationCommand(ApplicationCommandPayload):

    def __eq__(self, other):
        eq_list = ['guild_id', 'name', 'description', 'type', 'name_localization', 'description_localizations',
                   'options', 'default_member_permissions', 'dm_permission', 'default_permission', 'nsfw']
        result = True
        for eq_attr in eq_list:
            self_attr = getattr(self, eq_attr, None)
            other_attr = getattr(other, eq_attr, None)
            result = result and (self_attr == other_attr or self_attr is other_attr) # compare_missing_none(self_attr, other_attr)
        return result

    def __post_init__(self):
        if self.guild_id is not None and self.guild_id is not Missing:
            self.dm_permission = None
