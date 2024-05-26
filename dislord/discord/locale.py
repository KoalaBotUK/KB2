from enum import Enum


class Locale(Enum):
    INDONESIAN = 'id'
    DANISH = 'da'
    GERMAN = 'de'
    BRITISH_ENGLISH = 'en-GB'
    AMERICAN_ENGLISH = 'en-US'
    SPAIN_SPANISH = 'es-ES'
    FRENCH = 'fr'
    CROATIAN = 'hr'
    ITALIAN = 'it'
    LITHUANIAN = 'lt'
    HUNGARIAN = 'hu'
    DUTCH = 'nl'
    NORWEGIAN = 'no'
    POLISH = 'pl'
    BRAZIL_PORTUGUESE = 'pt-BR'
    ROMANIAN = 'ro'
    FINNISH = 'fi'
    SWEDISH = 'sv-SE'
    VIETNAMESE = 'vi'
    TURKISH = 'tr'
    CZECH = 'cs'
    GREEK = 'el'
    BULGARIAN = 'bg'
    RUSSIAN = 'ru'
    UKRAINIAN = 'uk'
    HINDI = 'hi'
    THAI = 'th'
    CHINESE = 'zh-CN'
    JAPANESE = 'ja'
    TAIWAN_CHINESE = 'zh-TW'
    KOREAN = 'ko'

    def __str__(self) -> str:
        return self.value