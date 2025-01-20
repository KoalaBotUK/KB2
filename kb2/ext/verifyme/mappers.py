from kb2.ext.koala.dtos import GuildDto
from kb2.ext.verifyme.dtos import EmailDto
from kb2.ext.verifyme.models import Email


def email_to_dto(model: Email) -> EmailDto:
    return EmailDto(email=model.email, user_id=model.user_id, organization=model.organization, active=model.active, date_added=model.date_added, date_updated=model.date_updated)