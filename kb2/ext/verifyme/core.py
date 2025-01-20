import datetime
import urllib.parse
import uuid

from pynamodb.exceptions import DoesNotExist

from kb2.errors import ErrorCodeException, KoalaErrorCode
from kb2.ext.verifyme.dtos import Organization
from kb2.ext.verifyme.errors import VerifymeErrorCode
from kb2.ext.verifyme.models import Email


def get_emails(user_id) -> list[Email]:
    return list(Email.scan(Email.user_id == user_id))  # FIXME: GSI instead


def add_email(user_id, email, organization, overwrite=False):
    try:
        email_model = Email.get(email)
    except DoesNotExist:
        email_model = None

    if email_model is not None and email_model.active:
        if email_model.user_id == user_id:
            raise ErrorCodeException(VerifymeErrorCode.LINK_EXISTS_SELF, f"{email} is already linked to this account")
        if not overwrite:
            raise ErrorCodeException(VerifymeErrorCode.LINK_EXISTS_OTHER, f"{email} is already linked to another account")

    email_model = Email(user_id=user_id, email=email, organization=organization)
    email_model.save()


def delete_email(user_id, email):
    email_model = Email.get(email)

    if email_model.user_id != user_id:
        raise ErrorCodeException(KoalaErrorCode.UNAUTHORIZED, "Email does not belong to user")

    email_model.active = False
    email_model.save()


def send_email(user_id, email):
    try:
        email_model = Email.get(email)
    except DoesNotExist:
        email_model = Email(user_id=user_id, email=email, organization=Organization.EMAIL, active=False)

    token = str(uuid.uuid4())
    email_model.token = token
    email_model.token_expiry = datetime.datetime.now() + datetime.timedelta(hours=1)
    email_model.save()
    callback_link = f"http://localhost:3000/verify/email/callback?{urllib.parse.urlencode({'token': token})}"

    # TODO: Send email
    print("Email Link: " + callback_link)
