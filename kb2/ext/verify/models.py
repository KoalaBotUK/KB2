from pynamodb.attributes import MapAttribute, ListAttribute, UnicodeAttribute, NumberAttribute, DynamicMapAttribute

from kb2.ext.koala.models import ExtensionAttr, Guild


class VerifyRoleAttr(MapAttribute):
    """
    {
        "role_id": "1234567890123456789",
        "type": 1,
        "regex": "((+44)|(07)).*",
        "blacklist_regex": "[a-z]*"
    }
    """
    role_id: str = UnicodeAttribute(hash_key=True)
    type: int = NumberAttribute()
    regex: str = UnicodeAttribute()
    blacklist_regex: str = UnicodeAttribute()


class VerifyDataAttr(DynamicMapAttribute):
    roles: list = ListAttribute(of=VerifyRoleAttr)


class VerifyExtensionAttr(ExtensionAttr, discriminator="verify"):
    data: VerifyDataAttr = VerifyDataAttr()

DEFAULT_EXT_DATA = VerifyExtensionAttr(id="verify", name="Verify", emoji="✅", version=1, enabled=False, hidden=True, data={})
