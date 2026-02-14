from pydantic import BaseModel


class ModuleOut(BaseModel):
    slug: str
    name: str
    coming_soon: bool
    enabled: bool
    capabilities: list[str]


class ModuleListResponse(BaseModel):
    modules: list[ModuleOut]
