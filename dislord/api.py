import json
from time import sleep

from pydantic import BaseModel, TypeAdapter
import httpx

from .discord.reference import DISCORD_URL
from .error import DiscordApiException
from .log import logger


# WARNING: Average time to call and get response from API is 25ms, not great to call lots if you want quick processing


class DiscordApi:
    def __init__(self, client, token, is_bot=False, base_url=DISCORD_URL):
        self.client = client
        self.token = token
        self.auth_header = {"Authorization": "Bot " if is_bot else "Bearer " + self.token}
        self.base_url = base_url

    def get(self, endpoint: str, params: dict = None, type_hint: type = None, **kwargs):
        logger.debug(f"📨 Sending to Discord API GET: {endpoint}, {params}")
        response = httpx.get(self.base_url + endpoint, params=params, **kwargs, headers=self.auth_header)
        if response.is_success:
            logger.debug(f"📬 Response from Discord API: {response.content}")
            try:
                response_payload = json.loads(response.content)
                if type_hint:
                    return TypeAdapter(type_hint).validate_json(response.content)
                else:
                    return response_payload
            except json.JSONDecodeError as err:
                logger.error(f"{err} error when calling discord API URL: GET {endpoint} Params: {params} Response: {response.text}")
                raise err
        elif response.status_code == 429:
            retry_after = response.json()["retry_after"]
            logger.warning(f"⚠️ Rate Limited, waiting {retry_after}s")
            sleep(retry_after)
            return self.get(endpoint, params, type_hint, **kwargs)
        else:
            logger.error(f"{response.status_code} {response.text} error when calling discord API "
                  f"URL: GET {endpoint} Params: {params}")
            raise DiscordApiException(f"{response.status_code} {response.text} error when calling discord API "
                                      f"URL: GET {endpoint} Params: {params}")

    def delete(self, endpoint: str, **kwargs):
        logger.debug(f"📨 Sending to Discord API DELETE: {endpoint}")
        response = httpx.delete(self.base_url + endpoint, **kwargs, headers=self.auth_header)
        if response.is_success:
            logger.debug(f"📬 Response from Discord API: {response.content}")
        elif response.status_code == 429:
            retry_after = response.json()["retry_after"]
            logger.warning(f"⚠️ Rate Limited, waiting {retry_after}s")
            sleep(retry_after)
            return self.delete(endpoint, **kwargs)
        else:
            raise DiscordApiException(f"{response.status_code} {response.text} error when calling discord API "
                                      f"URL: DELETE {endpoint}")

    def post(self, endpoint: str, body: BaseModel = None, type_hint: type = None, **kwargs):
        body_json = body.model_dump_json()
        logger.debug(f"📨 Sending to Discord API POST: {endpoint}, {body_json}")
        headers = self.auth_header
        headers["Content-Type"] = "application/json"
        response = httpx.post(self.base_url + endpoint, content=body_json, **kwargs, headers=headers)
        if response.is_success:
            logger.debug(f"📬 Response from Discord API: {response.content}")
            if not type_hint:
                return
            return TypeAdapter(type_hint).validate_json(response.content)
        elif response.status_code == 429:
            retry_after = response.json()["retry_after"]
            logger.warning(f"⚠️ Rate Limited, waiting {retry_after}s")
            sleep(retry_after)
            return self.post(endpoint, body, type_hint, **kwargs)
        else:
            raise DiscordApiException(f"{response.status_code} {response.text} error when calling discord API "
                                      f"URL: POST {endpoint} Body: {body_json}")

    def patch(self, endpoint: str, body: BaseModel = None, type_hint: type = None, **kwargs):
        body_json = body.model_dump_json()
        logger.debug(f"📨 Sending to Discord API PATCH: {endpoint}, {body_json}")
        headers = self.auth_header
        headers["Content-Type"] = "application/json"
        response = httpx.patch(self.base_url + endpoint, content=body_json, **kwargs, headers=headers)
        if response.is_success:
            logger.debug(f"📬 Response from Discord API: {response.content}")
            return TypeAdapter(type_hint).validate_json(response.content)
        elif response.status_code == 429:
            retry_after = response.json()["retry_after"]
            logger.warning(f"⚠️ Rate Limited, waiting {retry_after}s")
            sleep(retry_after)
            return self.patch(endpoint, body, type_hint, **kwargs)
        else:
            raise DiscordApiException(f"{response.status_code} {response.text} error when calling discord API "
                                      f"URL: PATCH {endpoint} Body: {body_json}")
