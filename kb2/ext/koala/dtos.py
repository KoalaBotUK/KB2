from pydantic import BaseModel


class ExtensionDto(BaseModel):
    cls: str
    id: str
    name: str
    emoji: str
    version: int
    enabled: bool
    hidden: bool
    data: dict | None = None


class GuildDto(BaseModel):
    guild_id: str
    extensions: list[ExtensionDto]
