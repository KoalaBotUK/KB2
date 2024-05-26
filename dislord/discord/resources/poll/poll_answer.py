from dislord.discord.base import BaseModel
from dislord.discord.resources.poll.poll_media import PollMedia


class PollAnswer(BaseModel):
    answer_id: int
    poll_media: PollMedia
