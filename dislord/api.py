import json
from time import sleep

from pydantic import BaseModel, TypeAdapter
import requests

from .discord.reference import DISCORD_URL
from .error import DiscordApiException


# WARNING: Average time to call and get response from API is 25ms, not great to call lots if you want quick processing


class DiscordApi:
    def __init__(self, client, bot_token):
        self.client = client
        self.bot_token = bot_token
        self.auth_header = {"Authorization": "Bot " + self.bot_token}

    def get(self, endpoint: str, params: dict = None, type_hint: type = None, **kwargs):
        print(f"📨 Sending to Discord API GET: {endpoint}, {params}")
        response = requests.get(DISCORD_URL + endpoint, params, **kwargs, headers=self.auth_header)
        if response.ok:
            print(f"📬 Response from Discord API: {response.content}")
            response_payload = json.loads(response.content)
            if type_hint:
                return TypeAdapter(type_hint).validate_json(response.content)
                # return cast(response_payload, type_hint, client=self.client)
            else:
                return response_payload
        elif response.status_code == 429:
            retry_after = response.json()["retry_after"]
            print(f"⚠️ Rate Limited, waiting {retry_after}s")
            sleep(retry_after)
            return self.get(endpoint, params, type_hint, **kwargs)
        else:
            raise DiscordApiException(f"{response.status_code} {response.text} error when calling discord API "
                                      f"URL: GET {endpoint} Params: {params}")

    def delete(self, endpoint: str, **kwargs):
        print(f"📨 Sending to Discord API DELETE: {endpoint}")
        response = requests.delete(DISCORD_URL + endpoint, **kwargs, headers=self.auth_header)
        if response.ok:
            print(f"📬 Response from Discord API: {response.content}")
            return
        elif response.status_code == 429:
            retry_after = response.json()["retry_after"]
            print(f"⚠️ Rate Limited, waiting {retry_after}s")
            sleep(retry_after)
            return self.delete(endpoint, **kwargs)
        else:
            raise DiscordApiException(f"{response.status_code} {response.text} error when calling discord API "
                                      f"URL: DELETE {endpoint}")

    def post(self, endpoint: str, body: BaseModel = None, type_hint: type = None, **kwargs):
        body_json = body.model_dump_json()
        print(f"📨 Sending to Discord API POST: {endpoint}, {body_json}")
        headers = self.auth_header
        headers["Content-Type"] = "application/json"
        response = requests.post(DISCORD_URL + endpoint, data=body_json,
                                 **kwargs, headers=headers)
        if response.ok:
            print(f"📬 Response from Discord API: {response.content}")
            return TypeAdapter(type_hint).validate_json(response.content)
            # return cast(json.loads(response.content), type_hint, client=self.client)
        elif response.status_code == 429:
            retry_after = response.json()["retry_after"]
            print(f"⚠️ Rate Limited, waiting {retry_after}s")
            sleep(retry_after)
            return self.post(endpoint, body, type_hint, **kwargs)
        else:
            raise DiscordApiException(f"{response.status_code} {response.text} error when calling discord API "
                                      f"URL: POST {endpoint} Body: {body_json}")

    def patch(self, endpoint: str, body: BaseModel = None, type_hint: type = None, **kwargs):
        body_json = body.model_dump_json()
        print(f"📨 Sending to Discord API PATCH: {endpoint}, {body_json}")
        headers = self.auth_header
        headers["Content-Type"] = "application/json"
        response = requests.patch(DISCORD_URL + endpoint, data=body_json,
                                  **kwargs, headers=headers)
        if response.ok:
            print(f"📬 Response from Discord API: {response.content}")
            return TypeAdapter(type_hint).validate_json(response.content)
            # return cast(json.loads(response.content), type_hint, client=self.client)
        elif response.status_code == 429:
            retry_after = response.json()["retry_after"]
            print(f"⚠️ Rate Limited, waiting {retry_after}s")
            sleep(retry_after)
            return self.post(endpoint, body, type_hint, **kwargs)
        else:
            raise DiscordApiException(f"{response.status_code} {response.text} error when calling discord API "
                                      f"URL: PATCH {endpoint} Body: {body_json}")
