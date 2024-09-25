import enum

from pynamodb.exceptions import DoesNotExist

import kb2
from .models import Guilds

# Constants


def get_version():
    return kb2.__version__

class Extension(enum.StrEnum):
    # KB2
    VERIFY = "Verify" # Name, Emoji, Version

    # KB1
    ANNOUNCE = "Announce"
    COLOUR_ROLE = "Colour Role"
    INSIGHTS = "Insights"
    REACT_FOR_ROLE = "React For Role"
    TEXT_FILTER = "Text Filter"
    TWITCH_ALERT = "Twitch Alert"
    VOTE = "Vote"


def set_extension(guild_id: str, extension ):
    Guilds.get(guild_id).enabled_extensions = extension


def get_extensions(guild_id: str):
    return {"enabled": Guilds.get(guild_id).enabled_extensions, "available": Guilds.get(guild_id).allowed_extensions}


def enable_extension(guild_id, extension_id):
    guild_model = Guilds.get_or_add(guild_id)

    extension_attr = guild_model.find_extension(extension_id)

    if extension_attr.hidden:
        raise ValueError(f"Extension {extension_id} is hidden")

    if extension_attr.enabled:
        raise ValueError(f"Extension {extension_id} is already enabled")

    extension_attr.enabled = True
    guild_model.save()


def disable_extension(guild_id, extension_id):
    guild_model = Guilds.get_or_add(guild_id)

    extension_attr = guild_model.find_extension(extension_id)

    if extension_attr.hidden:
        raise ValueError(f"Extension {extension_id} is hidden")

    if not extension_attr.enabled:
        raise ValueError(f"Extension {extension_id} is already disabled")

    extension_attr.enabled = False
    guild_model.save()


def delete_guild(guild_id):
    Guilds.get(guild_id).delete()
