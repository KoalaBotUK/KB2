from dislord.discord.resources.guild.guild import Guild as GuildPayload


class Guild(GuildPayload):

    @staticmethod
    def from_payload(payload: GuildPayload) -> 'Guild':
        return Guild(
            **payload
        )


