import json
import requests

from .error import DiscordApiException
from .models.base import cast, EnhancedJSONEncoder

DISCORD_API_VERSION = 10
DISCORD_URL = f"https://discord.com/api/v{DISCORD_API_VERSION}"


# WARNING: Average time to call and get response from API is 25ms, not great to call lots if you want quick processing


class DiscordApi:
    def __init__(self, client, bot_token):
        self.client = client
        self.bot_token = bot_token
        self.auth_header = {"Authorization": "Bot " + self.bot_token}

    def get(self, endpoint: str, params: dict = None, type_hint: type = None, **kwargs):
        print(f"📨 Sending to Discord API GET {endpoint}, {params}")
        response = requests.get(DISCORD_URL + endpoint, params, **kwargs, headers=self.auth_header)
        if response.ok:
            return cast(json.loads(response.content), type_hint, client=self.client)
        else:
            raise DiscordApiException(f"{response.status_code} {response.text} error when calling discord API "
                                      f"URL: GET {endpoint} Params: {params}")

    def delete(self, endpoint: str, **kwargs):
        print(f"📨 Sending to Discord API DELETE {endpoint}")
        response = requests.delete(DISCORD_URL + endpoint, **kwargs, headers=self.auth_header)
        if response.ok:
            return
        else:
            raise DiscordApiException(f"{response.status_code} {response.text} error when calling discord API "
                                      f"URL: DELETE {endpoint}")

    def post(self, endpoint: str, body: object = None, type_hint: type = None, **kwargs):
        body_json = json.dumps(body, cls=EnhancedJSONEncoder)
        print(f"📨 Sending to Discord API POST {endpoint}, {body_json}")
        headers = self.auth_header
        headers["Content-Type"] = "application/json"
        response = requests.post(DISCORD_URL + endpoint, data=body_json,
                                 **kwargs, headers=headers)
        if response.ok:
            return cast(json.loads(response.content), type_hint, client=self.client)
        else:
            raise DiscordApiException(f"{response.status_code} {response.text} error when calling discord API "
                                      f"URL: POST {endpoint} Body: {body_json}")
