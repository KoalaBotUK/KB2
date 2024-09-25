import json
from http.client import OK, UNAUTHORIZED
from fastapi import Response

from dislord.model.base import EnhancedJSONEncoder


class HttpResponse:
    status_code: int
    body: [dict, str]
    headers: dict

    def __init__(self, body, *, headers=None):
        self.body = body
        self.headers = headers

    def as_serverless_response(self):
        return {"statusCode": int(self.status_code),
                "body": json.dumps(self.body, cls=EnhancedJSONEncoder),
                "headers": self.headers}

    def as_server_response(self, response: Response):
        response.status_code = int(self.status_code)
        return self.body


class HttpOk(HttpResponse):
    status_code = OK


class HttpUnauthorized(HttpResponse):
    status_code = UNAUTHORIZED
