from dislord.discord.base import BaseModel


class PollAnswerCount(BaseModel):
    id: int
    count: int
    me_voted: bool


class PollResults(BaseModel):
    is_finalized: bool
    answer_counts: list[PollAnswerCount]
