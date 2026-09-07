from __future__ import annotations

from fastapi import APIRouter, Depends

from app.auth import require_session_token
from app.intent.hybrid_interpreter import HybridIntentInterpreter
from app.schemas.intents import IntentRequest, IntentResponse

router = APIRouter(prefix="/v1/intents", tags=["intents"])
interpreter = HybridIntentInterpreter()


@router.post(
    "/interpret",
    response_model=IntentResponse,
    dependencies=[Depends(require_session_token)],
)
async def interpret_intent(request: IntentRequest) -> IntentResponse:
    return await interpreter.interpret(request)
