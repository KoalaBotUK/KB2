import datetime
from abc import ABC, abstractmethod
from functools import cached_property

import httpx
from pynamodb.expressions.condition import Comparison
from pynamodb.pagination import ResultIterator

from kb2.ext.verifyme import env
from kb2.ext.verifyme.dtos import Organization
from kb2.ext.verifyme.models import Email


class Validator(ABC):
    _email: str | None = None

    @abstractmethod
    def validate(self, code: str) -> bool:
        """
        Validates a given token, returns true if valid
        :param code: oauth token for validation
        :return valid: bool if valid
        """
        pass

    @property
    def email(self) -> str | None:
        if self._email is None:
            raise AttributeError("Must call validate() first")
        return self._email

class OIDCValidator(Validator):
    token_endpoint: str
    client_id: str
    client_secret: str
    redirect_uri: str
    userinfo_endpoint: str

    def get_token(self, code):
        resp = httpx.post(self.token_endpoint, data={
            "grant_type": "authorization_code",
            "code": code,
            "scope": "openid email",
            "redirect_uri": self.redirect_uri,
            "client_id": self.client_id,
            "client_secret": self.client_secret
        })
        return resp.json().get("access_token")

    def get_email(self, token):
        resp = httpx.get(self.userinfo_endpoint, headers={"Authorization": f"Bearer {token}"})
        return resp.json().get("email")

    def validate(self, code: str) -> bool:
        self._email = self.get_email(self.get_token(code))
        return self._email is not None

class MicrosoftValidator(OIDCValidator):
    token_endpoint: str = "https://login.microsoftonline.com/common/oauth2/v2.0/token"
    client_id: str = env.MICROSOFT_CLIENT_ID
    client_secret: str = env.MICROSOFT_CLIENT_SECRET
    redirect_uri: str = env.MICROSOFT_REDIRECT_URI
    userinfo_endpoint: str = "https://graph.microsoft.com/oidc/userinfo"


class GoogleValidator(OIDCValidator):
    token_endpoint: str = "https://oauth2.googleapis.com/token"
    client_id: str = env.GOOGLE_CLIENT_ID
    client_secret: str = env.GOOGLE_CLIENT_SECRET
    redirect_uri: str = env.GOOGLE_REDIRECT_URI
    userinfo_endpoint: str = "https://openidconnect.googleapis.com/v1/userinfo"

class EmailValidator(Validator):
    def validate(self, code: str) -> bool:
        email_models: list[Email] = list(Email.scan(Email.token == code)) # TODO: Use GSI

        if len(email_models) != 1:
            return False
        else:
            email_model = email_models[0]

        if email_model.token_expiry is not None and email_model.token_expiry > datetime.datetime.now(tz=email_model.token_expiry.tzinfo):
            self._email = email_model.email

        email_model.token = None
        email_model.token_expiry = None
        email_model.save()

        return self._email is not None

class ValidatorFactory:
    validators = {
        Organization.MICROSOFT: MicrosoftValidator,
        Organization.GOOGLE: GoogleValidator,
        Organization.EMAIL: EmailValidator
    }

    def get(self, organization: Organization) -> Validator:
        return self.validators[organization]()