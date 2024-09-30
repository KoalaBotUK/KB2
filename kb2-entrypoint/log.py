import logging
import sys

_LOG_LEVEL = logging.DEBUG
_FORMATTER = logging.Formatter("%(asctime)s %(levelname)-8s %(message)s")

logger = logging.getLogger(__name__)
logger.setLevel(_LOG_LEVEL)

stdout_handler = logging.StreamHandler(sys.stdout)
stdout_handler.setFormatter(_FORMATTER)
stdout_handler.setLevel(_LOG_LEVEL)

logger.addHandler(stdout_handler)

