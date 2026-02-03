from pydantic import BaseModel


class ModuleOut(BaseModel):
    slug: str
    name: str
    coming_soon: bool


class ModuleListResponse(BaseModel):
    modules: list[ModuleOut]
