from __future__ import annotations

from fastapi import APIRouter, Depends

from app.auth import require_session_token
from app.intent.local_matcher import LocalIntentMatcher
from app.schemas.intents import IntentRequest, IntentResponse

router = APIRouter(prefix="/v1/intents", tags=["intents"])
matcher = LocalIntentMatcher()


@router.post(
    "/interpret",
    response_model=IntentResponse,
    dependencies=[Depends(require_session_token)],
)
def interpret_intent(request: IntentRequest) -> IntentResponse:
    return matcher.match(request)

