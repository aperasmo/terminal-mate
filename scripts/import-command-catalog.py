from __future__ import annotations

import argparse
import json
import re
from pathlib import Path


def slug(value: str) -> str:
    return re.sub(r"[^a-z0-9]+", "_", value.casefold()).strip("_")


def parse_catalog(source: str) -> list[dict[str, object]]:
    sections = re.split(r"(?m)^### ", source)[1:]
    entries: list[dict[str, object]] = []
    provider = "general"
    for section in sections:
        heading, _, body = section.partition("\n")
        before = source[: source.find(f"### {heading}")]
        provider_headers = re.findall(r"(?m)^# (Azure|AWS|GitHub CLI|Docker|Git|PowerShell / Windows|Waypoint project shortcuts)\s*$", before)
        if provider_headers:
            provider = slug(provider_headers[-1].replace(" CLI", "").replace(" / Windows", ""))

        risk_match = re.search(r"(?m)^\*\*([^*\n]+)\*\*", body)
        command_match = re.search(
            r"(?is)(?:Command(?: template)?|Resolve:[\s\S]*?Command template):\s*```(?:powershell)?\s*\n(.*?)\n```",
            body,
        )
        if not risk_match or not command_match:
            continue
        command = " ".join(line.strip() for line in command_match.group(1).splitlines()).strip()
        if not command or command.startswith("start-date ="):
            continue

        aliases: list[str] = []
        alias_match = re.search(r"(?is)Aliases:\s*```text\s*\n(.*?)\n```", body)
        if alias_match:
            aliases.extend(line.strip() for line in alias_match.group(1).splitlines() if line.strip())
        aliases.append(heading)

        parameters: list[str] = []
        for name in re.findall(r"<([a-zA-Z0-9][a-zA-Z0-9/-]*)>", command):
            normalized = re.sub(r"[^a-zA-Z0-9]+", "_", name)
            if normalized not in parameters and not name.startswith("YYYY"):
                parameters.append(normalized)

        entries.append(
            {
                "id": f"{provider}_{slug(heading)}",
                "provider": provider,
                "title": heading,
                "risk": risk_match.group(1).strip(),
                "command": command,
                "aliases": aliases,
                "required_parameters": parameters,
            }
        )
    return entries


def render_python(entries: list[dict[str, object]], source_name: str) -> str:
    payload = json.dumps(entries, indent=2, ensure_ascii=False)
    return (
        "# Generated from " + source_name + "; edit the source catalog and regenerate.\n"
        "# ruff: noqa: E501\n"
        "from __future__ import annotations\n\n"
        "COMMAND_CATALOG: list[dict[str, object]] = " + payload.replace("true", "True").replace("false", "False").replace("null", "None") + "\n"
    )


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("source", type=Path)
    parser.add_argument("output", type=Path)
    args = parser.parse_args()
    source = args.source.read_text(encoding="utf-8")
    entries = parse_catalog(source)
    args.output.write_text(render_python(entries, args.source.name), encoding="utf-8", newline="\n")
    print(f"Generated {len(entries)} deterministic command entries in {args.output}.")


if __name__ == "__main__":
    main()
