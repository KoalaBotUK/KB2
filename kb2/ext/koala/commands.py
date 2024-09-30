import asyncio
import threading

from dislord import CommandGroup
from dislord.discord.interactions.application_commands.enums import ApplicationCommandOptionType
from dislord.discord.interactions.application_commands.models import ApplicationCommandOption
from dislord.discord.interactions.components.enums import ComponentType, ButtonStyle
from dislord.discord.interactions.components.models import ActionRow, SelectMenu, SelectOption, Button
from dislord.discord.interactions.receiving_and_responding.interaction import Interaction
from dislord.discord.interactions.receiving_and_responding.interaction_response import InteractionResponse, \
    InteractionCallbackType, MessagesInteractionCallbackData
from dislord.discord.resources.channel.message import MessageFlags
from dislord.discord.resources.emoji.emoji import PartialEmoji
from kb2.ext.koala.models import Guilds
from kb2.main import client, owner_group
from kb2.ext.koala import core

koala_group = CommandGroup(client, name="koala", description="KoalaBot Base Commands")
# owner_koala_group = CommandGroup(client, name="koala", description="KoalaBot Base Owner Commands", parent=owner_group)
guild_owner_koala_group = CommandGroup(client, name="guild", description="Guild Controls", parent=owner_group)  # FIXME: parent not used


@koala_group.command(name="support", description="KoalaBot Support server link")
def support(interaction: Interaction):
    return InteractionResponse.message(
        content="Join our support server for more help! https://discord.gg/5etEjVd")


@owner_group.command(name="version", description="KoalaBot Version")
def version(interaction: Interaction):
    return InteractionResponse.message(
        content=core.get_version())


@guild_owner_koala_group.command(name="delete", description="Delete all data for a guild from KoalaBot",
                                 options=[
                                     ApplicationCommandOption(name="guild_id", description="Guild ID",
                                                              type=ApplicationCommandOptionType.STRING, required=True)])
def delete_guild(interaction: Interaction, guild_id: str):
    core.delete_guild(guild_id)
    return InteractionResponse.message(
        content="Deleted data for this guild")


def sync_task(interaction: Interaction):
    client.sync_commands()
    client.sync_commands(guild_ids=[guild.id for guild in client.guilds])
    client.edit_original_response(interaction.token,
                                  MessagesInteractionCallbackData(content="Synced commands for all guilds",
                                                                  flags=MessageFlags.EPHEMERAL))


@owner_group.command(name="sync", description="Sync commands for all guilds")
def sync(interaction: Interaction):
    threading.Thread(target=sync_task, args=(interaction,)).start()
    return InteractionResponse.message(content="Syncing commands for all guilds...",
                                       flags=MessageFlags.EPHEMERAL | MessageFlags.LOADING)


available_extensions = ["verify", "vote"]


@client.component_callback("extension_enable")
def extension_select(interaction: Interaction):
    modified_components = interaction.message.components
    extension_id = interaction.data.custom_id.split("$")[1]

    for action_row in modified_components:
        for button in action_row.components:
            if button.custom_id == interaction.data.custom_id:
                if button.style == ButtonStyle.SUCCESS:
                    button.style = ButtonStyle.SECONDARY
                    core.disable_extension(interaction.guild_id, extension_id)
                else:
                    button.style = ButtonStyle.SUCCESS
                    core.enable_extension(interaction.guild_id, extension_id)
                break

    return InteractionResponse(
        type=InteractionCallbackType.UPDATE_MESSAGE,
        data=MessagesInteractionCallbackData(
            components=modified_components
        ))


@client.command(name="extensions", description="KoalaBot Extensions", dm_permission=False)
def extensions(interaction: Interaction):
    extensions = Guilds.get_or_add(interaction.guild_id).extensions

    components = []
    for i in range(0, len(extensions), 5):
        row_buttons = [
            Button(
                type=ComponentType.BUTTON,
                style=ButtonStyle.SUCCESS if ext.enabled else ButtonStyle.SECONDARY,
                label=ext.name,
                custom_id=f"extension_enable${ext.id}",
                emoji=PartialEmoji(name=ext.emoji)
            ) for ext in extensions[i:i+5] if not ext.hidden
        ]
        components.append(ActionRow(type=ComponentType.ACTION_ROW, components=row_buttons))

    return InteractionResponse.message(
        content="Select extensions to enable/disable:",
        flags=MessageFlags.EPHEMERAL,
        components=components
    )