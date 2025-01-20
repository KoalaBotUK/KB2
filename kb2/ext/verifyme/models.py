import datetime
from typing import Optional, Dict, Any

from pynamodb.attributes import UnicodeAttribute, BooleanAttribute, UTCDateTimeAttribute, DynamicMapAttribute
from pynamodb.expressions.condition import Condition
from pynamodb.models import Model

from kb2 import env


class Email(Model):
    """
    {
        "email": "jack@gmail.com",
        "user_id": "1234567890123456789",
        "organization": 1,
        "active": true,
        "token": "ABCDEFG12345678",
        "token_expiry": "2000-01-01T00:00:00Z",
        "date_added": "2000-01-01T00:00:00Z",
        "date_updated": "2000-01-01T00:00:00Z"
    }
    """
    email = UnicodeAttribute(hash_key=True)
    organization = UnicodeAttribute()
    user_id = UnicodeAttribute()
    active = BooleanAttribute(default_for_new=True)
    token = UnicodeAttribute(null=True)
    token_expiry = UTCDateTimeAttribute(null=True)
    date_added = UTCDateTimeAttribute()
    date_updated = UTCDateTimeAttribute()

    def save(self, condition: Optional[Condition] = None, *, add_version_condition: bool = True) -> Dict[str, Any]:
        self.date_updated = datetime.datetime.now()

        if self.date_added is None:
            self.date_added = self.date_updated

        return super().save(condition=condition, add_version_condition=add_version_condition)

    class Meta:
        table_name = f'{env.ENV_PREFIX}kb_emails'
        region = 'eu-west-2'

