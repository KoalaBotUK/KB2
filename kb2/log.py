import logging
import sys
from datetime import date
from pathlib import Path

from kb2.env import CONFIG_PATH, LOGGING_FILE

_LOG_LEVEL = logging.DEBUG
_FORMATTER = logging.Formatter("%(asctime)s %(levelname)-8s %(message)s")
_LOG_DIR = Path(CONFIG_PATH, "logs", str(date.today()))

Path(_LOG_DIR).mkdir(exist_ok=True, parents=True)


logging.basicConfig(filename=Path(_LOG_DIR, 'kb.log'),
                    level=logging.WARN,
                    format='%(asctime)s %(levelname)-8s %(message)s')


def _get_default_warn_log():
    koala_log = logging.FileHandler(filename=Path(_LOG_DIR, "warn.kb.log"), encoding="utf-8")
    koala_log.setFormatter(_FORMATTER)
    koala_log.setLevel(logging.WARN)
    return koala_log


def _get_file_handler(log_name, log_level):
    file_handler = logging.FileHandler(filename=Path(_LOG_DIR, log_name), encoding="utf-8")
    file_handler.setFormatter(_FORMATTER)
    file_handler.setLevel(log_level)
    return file_handler


def _get_stdout_stream_handler(log_level):
    stream_handler = logging.StreamHandler(sys.stdout)
    stream_handler.setFormatter(_FORMATTER)
    stream_handler.setLevel(log_level)
    return stream_handler


def get_logger(log_name, log_level=_LOG_LEVEL, file_name=None, file_handler=True, stdout_handler=True):
    new_logger = logging.getLogger(log_name)

    if file_handler and LOGGING_FILE:
        new_logger.addHandler(_get_file_handler(file_name if file_name else log_name, log_level))
        new_logger.addHandler(_get_default_warn_log())

    if stdout_handler:
        new_logger.addHandler(_get_stdout_stream_handler(log_level))

    new_logger.setLevel(log_level)

    return new_logger


logger = get_logger(__name__)
