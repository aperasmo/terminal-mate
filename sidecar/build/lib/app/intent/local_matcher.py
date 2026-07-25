from __future__ import annotations

import re

from app.schemas.intents import CommandIntent, IntentRequest, IntentResponse


class LocalIntentMatcher:
    _CHANGE_DIRECTORY = re.compile(
        r"^(?:take|go|move|change)(?:\s+me)?\s+(?:to|into)\s+(.+?)(?:\s+directory|\s+folder)?[.!]?$",
        re.IGNORECASE,
    )

    _FIND_FILES_BY_EXTENSION = re.compile(
        r"\b(?:show|find|list)\b.*?\ball\b.*?\.([a-z0-9]{1,10})\b\s*files?\b",
        re.IGNORECASE,
    )

    # "show me the content of sample.txt lines 20-100" / "sample.txt's lines 20 to 100"
    _VIEW_FILE_LINES_PATH_FIRST = re.compile(
        r"^(?:show|view)(?:\s+me)?(?:\s+the)?(?:\s+content(?:s)?\s+of)?\s+"
        r"(?:the\s+file\s+)?(?P<path>\S+?)(?:'s)?\s+lines?(?:\s+number)?\s+"
        r"(?P<start>\d+)\s*(?:-|to|through)\s*(?P<end>\d+)[.!]?$",
        re.IGNORECASE,
    )

    # "show me lines 20-100 of sample.txt" / "view lines 20 to 100 of the file sample.txt"
    _VIEW_FILE_LINES_RANGE_FIRST = re.compile(
        r"^(?:show|view)(?:\s+me)?\s+lines?(?:\s+number)?\s+"
        r"(?P<start>\d+)\s*(?:-|to|through)\s*(?P<end>\d+)\s+of\s+"
        r"(?:the\s+file\s+)?(?P<path>\S+?)[.!]?$",
        re.IGNORECASE,
    )

    def match(self, request: IntentRequest) -> IntentResponse:
        message = request.message.strip()
        normalized = message.casefold().rstrip(".!?")

        if normalized in {"where am i", "show current directory", "show me the current directory"}:
            return self._matched("show_current_directory", {})

        directory_match = self._CHANGE_DIRECTORY.match(message)
        if directory_match:
            target = self._clean_directory_target(directory_match.group(1))
            return self._matched("change_directory", {"target": target})

        view_lines_match = self._VIEW_FILE_LINES_PATH_FIRST.match(
            message
        ) or self._VIEW_FILE_LINES_RANGE_FIRST.match(message)
        if view_lines_match:
            return self._matched(
                "view_file_lines",
                {
                    "path": view_lines_match.group("path"),
                    "start_line": int(view_lines_match.group("start")),
                    "end_line": int(view_lines_match.group("end")),
                },
            )

        extension_match = self._FIND_FILES_BY_EXTENSION.search(message)
        if extension_match:
            extension = extension_match.group(1).lower()
            return self._matched(
                "find_files",
                {
                    "root": ".",
                    "pattern": f"*.{extension}",
                    "recursive": True,
                },
            )

        return IntentResponse(
            matched=False,
            source="local",
            intent=None,
            message="This request is not supported by the local Phase 1 matcher.",
        )

    @staticmethod
    def _matched(action: str, parameters: dict[str, object]) -> IntentResponse:
        return IntentResponse(
            matched=True,
            source="local",
            intent=CommandIntent(
                schema_version=1,
                action=action,
                parameters=parameters,
            ),
            message=None,
        )

    @staticmethod
    def _clean_directory_target(raw_target: str) -> str:
        target = raw_target.strip().strip("\"'")
        target = re.sub(r"^(?:the|this)\s+", "", target, flags=re.IGNORECASE)
        target = re.sub(r"^(?:folder|directory)\s+", "", target, flags=re.IGNORECASE)
        target = re.sub(r"\s+(?:folder|directory)$", "", target, flags=re.IGNORECASE)
        return target.strip()
