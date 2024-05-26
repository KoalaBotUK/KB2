from dislord import CommandGroup
from dislord.discord.interactions.components.enums import ComponentType
from dislord.discord.interactions.components.models import ActionRow, SelectMenu, SelectOption
from dislord.discord.interactions.receiving_and_responding.interaction import Interaction
from dislord.discord.interactions.receiving_and_responding.interaction_response import InteractionResponse

extension_group = CommandGroup(name="extension", description="Extension configuration")


@extension_group.command(name="configure", description="Configure extensions")
def extension_configure(interaction: Interaction):
    action_row = ActionRow(type=ComponentType.ACTION_ROW, components=[
        SelectMenu(type=ComponentType.STRING_SELECT, custom_id=1, options=[
            SelectOption(label="a", value="a"),
            SelectOption(label="b", value="b")])])

    return InteractionResponse.message(components=[action_row], flags=0)
