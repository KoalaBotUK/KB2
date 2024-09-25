from pynamodb.exceptions import DoesNotExist
from pynamodb.models import Model, MetaModel
from pynamodb.attributes import UnicodeAttribute, MapAttribute, NumberAttribute, BooleanAttribute, \
    ListAttribute, DynamicMapAttribute, DiscriminatorAttribute

from dislord.discord.reference import Snowflake
from kb2 import env

#
# class DynModelMeta(MetaModel):
#     def __new__(cls, name, bases, namespace, discriminator=None):
#         new_class = super().__new__(cls, name, bases, namespace, discriminator=None)
#
#         # Define the Meta class if not defined
#         if not hasattr(new_class, 'Meta'):
#             class Meta:
#                 region = 'eu-west-2'
#             new_class.Meta = Meta
#
#         class_name = name.lower()
#         new_class.Meta.table_name = f'{env.ENV_PREFIX}kb_{class_name.lower()}'
#         return new_class


class ExtensionAttr(MapAttribute):
    cls = DiscriminatorAttribute()
    id: str = UnicodeAttribute(hash_key=True)
    name: str = UnicodeAttribute()
    emoji: str = UnicodeAttribute()
    version: int = NumberAttribute()
    enabled: bool = BooleanAttribute()
    hidden: bool = BooleanAttribute()
    data: dict = DynamicMapAttribute()


class LegacyExtension(ExtensionAttr, discriminator="legacy"):
    pass


class Guilds(Model):
    """
    {
        "guild_id": "1234567890123456789", # guild_id DEFAULT is default
        "extensions": [
            {
                "id": "announce",
                "name": "Announce",
                "emoji": "📢",
                "version": 1,
                "enabled": True,
                "hidden": False,
                "data": {}
            },
            {
                "id": "verify",
                "name": "Verify",
                "emoji": "✅",
                "version": 2,
                "enabled": False,
                "hidden": True,
                "data": {
                    "roles": [
                        {
                            "role_id": "1234567890123456789",
                            "type": 0,
                            "regex": ".*@gmail.com",
                            "blacklist_regex": "jack@gmail.com"
                        },
                        {
                            "role_id": "1234567890123456789",
                            "type": 1,
                            "regex": "((+44)|(07)).*"
                        }
                    ]
                }
            }
        ]
    }
    """
    class Meta:
        table_name = f'{env.ENV_PREFIX}kb_guilds'
        region = 'eu-west-2'
        billing_mode = 'PAY_PER_REQUEST'
    guild_id: Snowflake = UnicodeAttribute(hash_key=True)
    extensions: list[ExtensionAttr] = ListAttribute(of=ExtensionAttr, default=list)

    def append_extension(self, extension, overwrite=False):
        if extension.id in [e.id for e in self.extensions] and not overwrite:
            raise ValueError(f"Extension {extension.id} already exists")
        else:
            self.extensions.append(extension)

    def find_extension(self, extension_id):
        return next((e for e in self.extensions if e.id == extension_id), None)

    @staticmethod
    def get_or_add(guild_id):
        try:
            return Guilds.get(guild_id)
        except DoesNotExist:
            return Guilds(guild_id, extensions=Guilds.get("DEFAULT").extensions)


DEFAULT_LEGACY_EXTENSIONS = [
    LegacyExtension(id="announce", name="Announce", emoji="📢", version=1, enabled=False, hidden=False, data={}),
    LegacyExtension(id="colour_roles", name="Colour Role", emoji="🎨", version=1, enabled=False, hidden=False, data={}),
    LegacyExtension(id="rfr", name="React for Role", emoji="👍", version=1, enabled=False, hidden=False, data={}),
    LegacyExtension(id="filter", name="Text Filter", emoji="🔎", version=1, enabled=False, hidden=False, data={}),
    LegacyExtension(id="twitch_alerts", name="Twitch Alert", emoji="🔔", version=1, enabled=False, hidden=False, data={}),
    LegacyExtension(id="verify", name="Verify", emoji="✅", version=1, enabled=False, hidden=False, data={})
]

if __name__ == '__main__':
    print(Guilds.Meta.table_name)
    Guilds.create_table(wait=True)
    new_guild = Guilds('DEFAULT')
    new_guild.extensions = DEFAULT_LEGACY_EXTENSIONS
    new_guild.save()
    new_guild = Guilds.get('DEFAULT')
    print(Guilds.get('DEFAULT').extensions)
    print(new_guild.extensions)