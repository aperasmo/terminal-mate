from __future__ import annotations

import re
from datetime import date

from app.intent.command_catalog import COMMAND_CATALOG
from app.schemas.intents import CommandIntent, IntentResponse

_PLACEHOLDER = re.compile(r"<([a-zA-Z0-9][a-zA-Z0-9/-]*)>")
_LEADING_QUERY_VERB = re.compile(
    r"^(?:check|view|get|display|inspect)\s+", re.IGNORECASE
)
_EXTRA_ALIASES = {
    "aws_show_lambda_concurrent_executions_quota": ["show lambda quota", "check lambda quota"],
    "github_list_workflow_runs": [
        "list github workflows",
        "list github actions",
        "list github workflows for <owner/repo> workflow <workflow>",
    ],
    "github_trigger_workflow": [
        "trigger workflow <workflow> for <owner/repo> on <branch>"
    ],
    "github_list_github_actions_secret_names": [
        "list github actions secret names for <owner/repo>"
    ],
    "docker_show_docker_container_logs": ["show docker container logs <container>"],
    "docker_stop_docker_container": ["stop docker container <container>"],
    "docker_remove_docker_container": ["remove docker container <container>"],
    "git_stage_explicit_git_files": ["stage git files <files>"],
    "git_hard_reset_git": ["hard reset git to <target>"],
    "powershell_check_whether_path_exists": ["check whether path <path> exists"],
    "powershell_check_whether_command_exists": [
        "check whether command <command> exists"
    ],
    "powershell_test_tcp_port": ["test tcp port <host> <port>"],
}


def _parameter_name(value: str) -> str:
    return re.sub(r"[^a-zA-Z0-9]+", "_", value)


def _normalize(value: str) -> str:
    value = value.strip().rstrip(".!?")
    value = re.sub(r"\s+", " ", value)
    return _LEADING_QUERY_VERB.sub("show ", value)


def _alias_pattern(alias: str) -> tuple[re.Pattern[str], list[str]]:
    names: list[str] = []
    pieces: list[str] = []
    cursor = 0
    for match in _PLACEHOLDER.finditer(alias):
        pieces.append(re.escape(alias[cursor : match.start()]))
        name = _parameter_name(match.group(1))
        names.append(name)
        pieces.append(fr"(?P<{name}>.+?)")
        cursor = match.end()
    pieces.append(re.escape(alias[cursor:]))
    return re.compile("^" + "".join(pieces) + "$", re.IGNORECASE), names


def _render(entry: dict[str, object], values: dict[str, str]) -> str:
    command = str(entry["command"])
    command = _PLACEHOLDER.sub(
        lambda match: values.get(_parameter_name(match.group(1)), match.group(0)),
        command,
    )
    today = date.today()
    command = command.replace("<YYYY-MM-01>", today.replace(day=1).isoformat())
    command = command.replace("<YYYY-MM-DD>", today.isoformat())
    return command


class DeterministicCatalogMatcher:
    def match(self, message: str) -> IntentResponse | None:
        normalized = _normalize(message)
        for entry in COMMAND_CATALOG:
            aliases = [*entry["aliases"], *_EXTRA_ALIASES.get(str(entry["id"]), [])]
            command = str(entry["command"])
            if "<" not in command:
                aliases.append(command)
            for raw_alias in aliases:
                alias = _normalize(str(raw_alias))
                pattern, names = _alias_pattern(alias)
                match = pattern.fullmatch(normalized)
                if not match:
                    continue
                values = {
                    name: value.strip().strip("\"'")
                    for name, value in match.groupdict().items()
                    if value and value.strip()
                }
                required = [str(name) for name in entry["required_parameters"]]
                # Some friendly aliases intentionally use a collective name
                # (for example <files>) that is also the catalog parameter.
                missing = [name for name in required if name not in values]
                if missing:
                    labels = ", ".join(name.replace("_", "-") for name in missing)
                    return IntentResponse(
                        matched=False,
                        source="local",
                        intent=None,
                        message=f"Provide the required value(s): {labels}.",
                        requires_clarification=True,
                    )
                rendered = _render(entry, values)
                if _PLACEHOLDER.search(rendered):
                    unresolved = sorted(set(_PLACEHOLDER.findall(rendered)))
                    labels = ", ".join(unresolved)
                    return IntentResponse(
                        matched=False,
                        source="local",
                        intent=None,
                        message=f"Provide the required value(s): {labels}.",
                        requires_clarification=True,
                    )
                return IntentResponse(
                    matched=True,
                    source="local",
                    intent=CommandIntent(
                        schema_version=1,
                        action="catalog_command",
                        parameters={
                            "catalog_id": entry["id"],
                            "command": rendered,
                            "risk": entry["risk"],
                        },
                    ),
                    message=None,
                    requires_clarification=False,
                )
        return None
