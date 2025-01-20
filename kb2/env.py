import os
from pathlib import Path

from dotenv import load_dotenv

load_dotenv()

PUBLIC_KEY = os.environ.get("PUBLIC_KEY")
BOT_TOKEN = os.environ.get("DISCORD_TOKEN")

API_PORT = os.environ.get("API_PORT", 8080)

# Logging
LOGGING_FILE = eval(os.environ.get("LOGGING_FILE", "False"))
LOG_PATH = None

# Config Path
if LOGGING_FILE:
    LOG_PATH = Path(os.environ.get("CONFIG_PATH", './../logs' if os.name == 'nt' else '/logs'))
    LOG_PATH.mkdir(exist_ok=True, parents=True)

ENV_PREFIX = os.environ.get("ENV_PREFIX", "local_")

API_GATEWAY_BASE_PATH = os.environ.get("API_GATEWAY_BASE_PATH", "/")

JWKS_URL = os.environ.get("JWKS_URL", "https://kb2.auther.jackdraper.co.uk/.well-known/jwks.json")
