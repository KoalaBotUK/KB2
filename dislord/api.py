import json
from time import sleep

from pydantic import BaseModel, TypeAdapter
import httpx

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
        response = httpx.get(DISCORD_URL + endpoint, params=params, **kwargs, headers=self.auth_header)
        if response.is_success:
            print(f"📬 Response from Discord API: {response.content}")
            response_payload = json.loads(response.content)
            if type_hint:
                return TypeAdapter(type_hint).validate_json(response.content)
            else:
                return response_payload
        elif response.status_code == 429:
            retry_after = response.json()["retry_after"]
            print(f"⚠️ Rate Limited, waiting {retry_after}s")
            sleep(retry_after)
            return self.get(endpoint, params, type_hint, **kwargs)
        else:
            print(f"{response.status_code} {response.text} error when calling discord API "
                  f"URL: GET {endpoint} Params: {params}")
            raise DiscordApiException(f"{response.status_code} {response.text} error when calling discord API "
                                      f"URL: GET {endpoint} Params: {params}")

    def delete(self, endpoint: str, **kwargs):
        print(f"📨 Sending to Discord API DELETE: {endpoint}")
        response = httpx.delete(DISCORD_URL + endpoint, **kwargs, headers=self.auth_header)
        if response.is_success:
            print(f"📬 Response from Discord API: {response.content}")
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
        response = httpx.post(DISCORD_URL + endpoint, data=json.loads(body_json), **kwargs, headers=headers)
        if response.is_success:
            print(f"📬 Response from Discord API: {response.content}")
            return TypeAdapter(type_hint).validate_json(response.content)
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
        response = httpx.patch(DISCORD_URL + endpoint, data=json.loads(body_json), **kwargs, headers=headers)
        if response.is_success:
            print(f"📬 Response from Discord API: {response.content}")
            return TypeAdapter(type_hint).validate_json(response.content)
        elif response.status_code == 429:
            retry_after = response.json()["retry_after"]
            print(f"⚠️ Rate Limited, waiting {retry_after}s")
            sleep(retry_after)
            return self.post(endpoint, body, type_hint, **kwargs)
        else:
            raise DiscordApiException(f"{response.status_code} {response.text} error when calling discord API "
                                      f"URL: PATCH {endpoint} Body: {body_json}")
