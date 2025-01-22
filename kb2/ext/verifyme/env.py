import os
from dotenv import load_dotenv

load_dotenv()

EMAIL_REDIRECT_URI = os.environ.get("EMAIL_REDIRECT_URI")
