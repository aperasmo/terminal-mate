from pydantic import BaseModel


class HealthResponse(BaseModel):
    status: str
    protocol_version: str
    service_version: str

