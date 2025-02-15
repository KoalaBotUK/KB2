import datetime
from abc import ABC, abstractmethod

import httpx

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
    userinfo_endpoint: str

    def get_email(self, token):
        resp = httpx.get(self.userinfo_endpoint, headers={"Authorization": f"Bearer {token}"})
        return resp.json().get("email")

    def validate(self, token: str) -> bool:
        self._email = self.get_email(token)
        return self._email is not None

class MicrosoftValidator(OIDCValidator):
    userinfo_endpoint: str = "https://graph.microsoft.com/oidc/userinfo"


class GoogleValidator(OIDCValidator):
    userinfo_endpoint: str = "https://openidconnect.googleapis.com/v1/userinfo"

class EmailValidator(Validator):
    def validate(self, token: str) -> bool:
        email_models: list[Email] = list(Email.scan(Email.token == token)) # TODO: Use GSI

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