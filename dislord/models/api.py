import json
from dataclasses import dataclass
from http.client import OK, UNAUTHORIZED

from .base import EnhancedJSONEncoder


@dataclass
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

    def as_server_response(self):
        return json.loads(json.dumps(self.body, cls=EnhancedJSONEncoder)), int(self.status_code)


class HttpOk(HttpResponse):
    status_code = OK


class HttpUnauthorized(HttpResponse):
    status_code = UNAUTHORIZED
