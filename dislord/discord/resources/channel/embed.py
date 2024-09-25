from dislord.types import ObjDict
from dislord.discord.reference import Missing, ISOTimestamp


class EmbedField(ObjDict):
    name: str
    value: str
    inline: bool | Missing = None


class EmbedFooter(ObjDict):
    text: str
    icon_url: str | Missing = None
    proxy_icon_url: str | Missing = None


class EmbedAuthor(ObjDict):
    name: str
    url: str | Missing = None
    icon_url: str | Missing = None
    proxy_icon_url: str | Missing = None


class EmbedProvider(ObjDict):
    name: str | Missing = None
    url: str | Missing = None


class EmbedImage(ObjDict):
    url: str
    proxy_url: str | Missing = None
    height: int | Missing = None
    width: int | Missing = None


class EmbedVideo(ObjDict):
    url: str | Missing = None
    proxy_url: str | Missing = None
    height: int | Missing = None
    width: int | Missing = None


class EmbedThumbnail(ObjDict):
    url: str
    proxy_url: str | Missing = None
    height: int | Missing = None
    width: int | Missing = None


class Embed(ObjDict):
    title: str | Missing = None
    type: str | Missing = None
    description: str | Missing = None
    url: str | Missing = None
    timestamp: ISOTimestamp | Missing = None
    color: int | Missing = None
    footer: EmbedFooter | Missing = None
    image: EmbedImage | Missing = None
    thumbnail: EmbedThumbnail | Missing = None
    video: EmbedVideo | Missing = None
    provider: EmbedProvider | Missing = None
    author: EmbedAuthor | Missing = None
    fields: list[EmbedField] | Missing = None
