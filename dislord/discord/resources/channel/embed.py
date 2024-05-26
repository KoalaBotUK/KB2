from dislord.discord.base import BaseModel
from dislord.discord.type import Missing, ISOTimestamp


class EmbedField(BaseModel):
    name: str
    value: str
    inline: bool | Missing


class EmbedFooter(BaseModel):
    text: str
    icon_url: str | Missing
    proxy_icon_url: str | Missing


class EmbedAuthor(BaseModel):
    name: str
    url: str | Missing
    icon_url: str | Missing
    proxy_icon_url: str | Missing


class EmbedProvider(BaseModel):
    name: str | Missing
    url: str | Missing


class EmbedImage(BaseModel):
    url: str
    proxy_url: str | Missing
    height: int | Missing
    width: int | Missing


class EmbedVideo(BaseModel):
    url: str | Missing
    proxy_url: str | Missing
    height: int | Missing
    width: int | Missing


class EmbedThumbnail(BaseModel):
    url: str
    proxy_url: str | Missing
    height: int | Missing
    width: int | Missing


class Embed(BaseModel):
    title: str | Missing
    type: str | Missing
    description: str | Missing
    url: str | Missing
    timestamp: ISOTimestamp | Missing
    color: int | Missing
    footer: EmbedFooter | Missing
    image: EmbedImage | Missing
    thumbnail: EmbedThumbnail | Missing
    video: EmbedVideo | Missing
    provider: EmbedProvider | Missing
    author: EmbedAuthor | Missing
    fields: list[EmbedField] | Missing
