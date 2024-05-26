from dislord.discord.base import BaseModel


class PollCreateRequest(BaseModel):
    question: PollMedia
    answers: list[PollAnswer]
    duration: int
    allow_multiset: bool
    layout_type: LayoutType | Missing
