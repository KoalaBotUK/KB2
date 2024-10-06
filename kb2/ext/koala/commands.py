from dislord import CommandGroup
from dislord.discord.interactions.application_commands.enums import ApplicationCommandOptionType
from dislord.discord.interactions.application_commands.models import ApplicationCommandOption
from dislord.discord.interactions.components.enums import ComponentType, ButtonStyle
from dislord.discord.interactions.components.models import ActionRow, Button
from dislord.discord.interactions.receiving_and_responding.interaction import Interaction
from dislord.discord.interactions.receiving_and_responding.interaction_response import InteractionResponse, \
    InteractionCallbackType, MessagesInteractionCallbackData
from dislord.discord.resources.channel.message import MessageFlags
from dislord.discord.resources.emoji.emoji import PartialEmoji
from kb2.client import client, owner_group, OwnerCommandGroup
from kb2.ext.koala import core
from kb2.ext.koala.models import Guilds
from kb2.log import logger

koala_group = CommandGroup(client, name="koala", description="KoalaBot Base Commands")
owner_koala_group = OwnerCommandGroup(client, name="owner-koala", description="KoalaBot Base Owner Commands")
guild_owner_koala_group = CommandGroup(client, name="guild", description="Guild Controls", parent=owner_koala_group)


@koala_group.command(name="support", description="KoalaBot Support server link",
                     defer=InteractionResponse(
                         type=InteractionCallbackType.DEFERRED_CHANNEL_MESSAGE_WITH_SOURCE))
def support(interaction: Interaction):
    return InteractionResponse.message(
        content="Join our support server for more help! https://discord.gg/5etEjVd")


@owner_group.command(name="version", description="KoalaBot Version",
                     defer=InteractionResponse(
                         type=InteractionCallbackType.DEFERRED_CHANNEL_MESSAGE_WITH_SOURCE))
def version(interaction: Interaction):
    return InteractionResponse.message(
        content=core.get_version())


@guild_owner_koala_group.command(name="delete", description="Delete all data for a guild from KoalaBot",
                                 options=[
                                     ApplicationCommandOption(name="guild_id", description="Guild ID",
                                                              type=ApplicationCommandOptionType.STRING, required=True)],
                                 defer=InteractionResponse(
                                     type=InteractionCallbackType.DEFERRED_CHANNEL_MESSAGE_WITH_SOURCE))
def delete_guild(interaction: Interaction, guild_id: str):
    core.delete_guild(guild_id)
    return InteractionResponse.message(
        content="Deleted data for this guild")


@owner_group.command(name="sync", description="Sync commands for all guilds",
                     defer=InteractionResponse(
                         type=InteractionCallbackType.CHANNEL_MESSAGE_WITH_SOURCE,
                         data=MessagesInteractionCallbackData(content="Syncing commands for all guilds...",
                                                              flags=MessageFlags.EPHEMERAL | MessageFlags.LOADING)))
def sync(interaction: Interaction):
    client.sync_commands()
    client.sync_commands(guild_ids=[guild.id for guild in client.guilds])
    return InteractionResponse.message(content="Synced commands for all guilds",
                                       flags=MessageFlags.EPHEMERAL)


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


@client.command(name="extensions", description="KoalaBot Extensions", dm_permission=False,
                defer=InteractionResponse(type=InteractionCallbackType.DEFERRED_CHANNEL_MESSAGE_WITH_SOURCE,
                                          flags=MessageFlags.EPHEMERAL))
def extensions(interaction: Interaction):
    logger.debug("Getting Guild")
    k_extensions = Guilds.get_or_add(interaction.guild_id).extensions
    logger.debug("Got Guild")

    components = []
    for i in range(0, len(k_extensions), 5):
        row_buttons = [
            Button(
                type=ComponentType.BUTTON,
                style=ButtonStyle.SUCCESS if ext.enabled else ButtonStyle.SECONDARY,
                label=ext.name,
                custom_id=f"extension_enable${ext.id}",
                emoji=PartialEmoji(name=ext.emoji)
            ) for ext in k_extensions[i:i+5] if not ext.hidden
        ]
        components.append(ActionRow(type=ComponentType.ACTION_ROW, components=row_buttons))

    return InteractionResponse.message(
        content="Select extensions to enable/disable:",
        flags=MessageFlags.EPHEMERAL,
        components=components
    )
