from pydantic import BaseModel, ConfigDict


class ObjDict(BaseModel):
    model_config = ConfigDict()
