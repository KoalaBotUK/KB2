from dislord.discord.base import BaseModel
from dislord.discord.resources.poll.layout_type import LayoutType
from dislord.discord.resources.poll.poll_answer import PollAnswer
from dislord.discord.resources.poll.poll_media import PollMedia
from dislord.discord.resources.poll.poll_results import PollResults
from dislord.discord.type import ISOTimestamp, Missing


class Poll(BaseModel):
    question: PollMedia
    answers: list[PollAnswer]
    expiry: ISOTimestamp | None
    allow_multiset: bool
    layout_type: LayoutType
    results: PollResults | Missing