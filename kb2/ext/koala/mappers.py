from kb2.ext.koala.dtos import GuildDto, ExtensionDto
from kb2.ext.koala.models import Guild, ExtensionAttr


def extension_to_dto(extension: ExtensionAttr) -> ExtensionDto:
    return ExtensionDto(
        cls=extension.__class__.__name__,
        id=extension.id,
        name=extension.name,
        emoji=extension.emoji,
        version=extension.version,
        enabled=extension.enabled,
        hidden=extension.hidden,
        # data=extension.data FIXME: Generic pynamodb to pydantic mapper
    )


def guild_to_dto(guild: Guild) -> GuildDto:
    return GuildDto(guild_id=guild.guild_id, extensions=[extension_to_dto(ext) for ext in guild.extensions])