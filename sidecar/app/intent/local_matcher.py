from __future__ import annotations

import re
import shlex

from app.intent.catalog_matcher import DeterministicCatalogMatcher
from app.schemas.intents import CommandIntent, IntentRequest, IntentResponse


class LocalIntentMatcher:
    def __init__(self) -> None:
        self._command_catalog = DeterministicCatalogMatcher()

    _CHANGE_DIRECTORY = re.compile(
        r"^(?:take|go|move|change)(?:\s+me)?\s+(?:to|into)\s+(.+?)(?:\s+directory|\s+folder)?[.!]?$",
        re.IGNORECASE,
    )

    # "go up one directory" / "go back" / "move up a level" — folds into the
    # same change_directory action the adapter already renders, using ".."
    # as the target, so no new adapter action is needed for this one.
    _GO_UP_DIRECTORY = re.compile(
        r"^(?:go|move)\s+(?:back|up)(?:\s+(?:one|a)\s+(?:level|directory|folder))?[.!]?$",
        re.IGNORECASE,
    )

    _FIND_FILES_BY_EXTENSION = re.compile(
        r"\b(?:show|find|list)\b.*?\ball\b.*?\.([a-z0-9]{1,10})\b"
        r"(?:\s*files?\b)?",
        re.IGNORECASE,
    )

    _FIND_FILES_BY_NAME = re.compile(
        r"^(?:find|locate|search\s+for)(?:\s+me)?\s+(?:the\s+)?files?"
        r"(?:\s+(?:named|called))?\s+(?P<pattern>.+?)[.!]?$",
        re.IGNORECASE,
    )

    # A plain directory listing with no extension filter — "list the files
    # here" / "show me the files" — distinct from _FIND_FILES_BY_EXTENSION,
    # which requires "all" plus a literal extension.
    _LIST_DIRECTORY = re.compile(
        r"^(?:show|list)\s+(?:me\s+)?(?:the\s+)?files"
        r"(?:\s+(?:here|in\s+this\s+folder|in\s+this\s+directory))?[.!?]?$",
        re.IGNORECASE,
    )

    # "show hidden files with name and mode" / "list all files and folders,
    # including hidden items, and show their name and mode". Keep this ahead
    # of the broad read-file matcher so "show hidden files" is not treated as
    # a filename.
    _LIST_HIDDEN_ITEMS_WITH_DETAILS = re.compile(
        r"^(?:show|list)\s+(?:me\s+)?(?:"
        r"hidden\s+(?:items?|files?|folders?)|"
        r"(?:all\s+)?(?:files?\s+and\s+folders?|items?|files?)"
        r"(?:,?\s+including\s+hidden\s+(?:items?|files?|folders?))?"
        r")"
        r"(?:\s+with\s+|,?\s+and\s+show\s+(?:their\s+)?)name\s+and\s+mode[.!?]?$",
        re.IGNORECASE,
    )

    # A named path in the current folder — "list files .claude in this folder"
    # / "list .claude in this folder". The adapter validates whether the target
    # is a file or folder before choosing the output.
    _INSPECT_NAMED_PATH = re.compile(
        r"^(?:show|list)\s+(?:me\s+)?(?:the\s+)?"
        r"(?:(?:files?|contents?)\s+)?(?P<root>.+?)\s+"
        r"in\s+(?:this|the\s+current)\s+(?:folder|directory)[.!?]?$",
        re.IGNORECASE,
    )

    _WHATS_IN_THIS_FOLDER = re.compile(
        r"^what(?:'s|\s+is)\s+in\s+(?:this|the)\s+(?:folder|directory)[?.!]?$",
        re.IGNORECASE,
    )

    _LIST_DIRECTORIES = re.compile(
        r"^(?:show|list)\s+(?:me\s+)?(?:the\s+)?(?:folders?|directories?)"
        r"(?:\s+(?:here|in\s+this\s+(?:folder|directory)))?(?:\s+only)?[.!?]?$",
        re.IGNORECASE,
    )

    # "create a folder called drafts" / "make a new directory drafts"
    _MAKE_DIRECTORIES = re.compile(
        r"^(?:create|make)(?:\s+new)?\s+(?:folders|directories)\s+"
        r"(?P<paths>.+?)[.!]?$",
        re.IGNORECASE,
    )

    _MAKE_DIRECTORY = re.compile(
        r"^(?:create|make)(?:\s+a)?(?:\s+new)?\s+(?:folder|directory)\s+"
        r"(?:(?:called|named)\s+)?(?P<path>.+?)[.!]?$",
        re.IGNORECASE,
    )

    _CREATE_FILES = re.compile(
        r"^(?:create|make)(?:\s+new)?\s+files\s+(?P<paths>.+?)[.!]?$",
        re.IGNORECASE,
    )

    _CREATE_FILE = re.compile(
        r"^(?:create|make)(?:\s+a)?(?:\s+new)?\s+file\s+"
        r"(?:(?:called|named)\s+)?(?P<path>.+?)[.!]?$",
        re.IGNORECASE,
    )

    _DELETE_FILE = re.compile(
        r"^(?:delete|remove)(?:\s+the)?\s+file\s+"
        r"(?:(?:called|named)\s+)?(?P<path>.+?)[.!]?$",
        re.IGNORECASE,
    )

    _PATH_ARGUMENT = r'''(?:"[^"]+"|'[^']+'|[^\s,;]+)'''

    _BACKUP_AND_REPLACE_FILE = re.compile(
        rf"^(?:back\s+up|backup)(?:\s+the)?(?:\s+file)?\s+"
        rf"(?P<target>{_PATH_ARGUMENT})(?:\s+as)?\s+"
        rf"(?P<backup>{_PATH_ARGUMENT})\s*(?:,|;)?\s*"
        rf"(?:then|and(?:\s+then)?)\s+replace(?:\s+the)?\s+"
        rf"(?P<replace_target>original(?:\s+file)?|{_PATH_ARGUMENT})"
        rf"(?:\s+with)?\s+(?P<source>{_PATH_ARGUMENT})[.!]?$",
        re.IGNORECASE,
    )

    _BACKUP_FILE = re.compile(
        rf"^(?:back\s+up|backup)(?:\s+the)?(?:\s+file)?\s+"
        rf"(?P<source>{_PATH_ARGUMENT})(?:\s+as)?\s+"
        rf"(?P<backup>{_PATH_ARGUMENT})[.!]?$",
        re.IGNORECASE,
    )

    _REPLACE_FILE = re.compile(
        rf"^replace(?:\s+the)?(?:\s+file)?\s+"
        rf"(?P<target>{_PATH_ARGUMENT})(?:\s+with)?\s+"
        rf"(?P<source>{_PATH_ARGUMENT})[.!]?$",
        re.IGNORECASE,
    )

    _TRANSFER_FILE_FROM_DIRECTORY = re.compile(
        r"^(?P<operation>copy|cut|move)(?:\s+the)?\s+(?:file|fle)\s+"
        r"(?P<filename>.+?)\s+from\s+(?P<source_directory>.+?)\s+to\s+"
        r"(?P<destination>.+?)[.!]?$",
        re.IGNORECASE,
    )

    _TRANSFER_FILE_PATH = re.compile(
        r"^(?P<operation>copy|cut|move)(?:\s+the)?\s+(?:file|fle)\s+"
        r"(?P<source>.+?)\s+to\s+(?P<destination>.+?)[.!]?$",
        re.IGNORECASE,
    )

    # A grep-like content search — "search for TODO in this folder" / "find
    # the text TODO in files" — complements view_file_lines as the other half
    # of Phase 1's "grep-like" scope (search vs. read a specific range).
    _SEARCH_FILE_CONTENTS = re.compile(
        r"^(?:search|find)(?:\s+for)?\s+(?:the\s+text\s+)?[\"']?(?P<term>.+?)[\"']?\s+"
        r"in\s+(?:this\s+folder|the\s+files|files)[.!]?$",
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

    _VIEW_FILE_START = re.compile(
        r"^(?:show|read|display|preview)(?:\s+me)?\s+(?:the\s+)?(?:first|top)"
        r"(?:\s+(?P<count>\d+))?\s+lines?(?:\s+(?:of|from))?\s+"
        r"(?:the\s+file\s+)?(?P<path>.+?)[.!]?$",
        re.IGNORECASE,
    )

    _VIEW_FILE_END = re.compile(
        r"^(?:show|read|display|preview)(?:\s+me)?\s+(?:the\s+)?(?:last|bottom)"
        r"(?:\s+(?P<count>\d+))?\s+lines?\s+(?:of|from)\s+"
        r"(?:the\s+file\s+)?(?P<path>.+?)[.!]?$",
        re.IGNORECASE,
    )

    _VIEW_FILE_EDGES = re.compile(
        r"^(?:show|read|display|preview)(?:\s+me)?\s+(?:the\s+)?"
        r"(?:first\s*(?:/|and)\s*last|top\s*(?:/|and)\s*bottom)"
        r"(?:\s+(?P<count>\d+))?\s+lines?(?:\s+(?:of|from))?\s+"
        r"(?:the\s+file\s+)?(?P<path>.+?)[.!]?$",
        re.IGNORECASE,
    )

    _INCOMPLETE_FILE_PREVIEW = re.compile(
        r"^(?:show|read|display|preview)(?:\s+me)?\s+(?:the\s+)?"
        r"(?:(?:first|top)\s*(?:/|and)\s*(?:last|bottom)|"
        r"(?:first|top|last|bottom))"
        r"(?:\s+\d+)?\s+lines?[.!]?$",
        re.IGNORECASE,
    )

    _READ_FILE = re.compile(
        r"^(?:read|display|print|show)(?:\s+me)?\s+(?:the\s+)?"
        r"(?:(?:contents?|content)\s+of\s+)?(?:the\s+)?(?:file\s+)?"
        r"(?P<path>.+?)[.!]?$",
        re.IGNORECASE,
    )

    _INSPECT_FILE = re.compile(
        r"^(?:show|display|inspect)\s+(?:me\s+)?(?:the\s+)?"
        r"(?:file\s+)?(?:details|info|information|metadata)\s+(?:for|about|of)\s+"
        r"(?:the\s+file\s+)?(?P<path>.+?)[.!]?$",
        re.IGNORECASE,
    )

    _COUNT_FILES = re.compile(
        r"^(?:count|how\s+many)\s+(?:all\s+)?files"
        r"(?:\s+are\s+there)?(?:\s+(?:here|in\s+this\s+(?:folder|directory)))?"
        r"(?:\s+(?P<recursive>recursively|including\s+subfolders))?[.!?]?$",
        re.IGNORECASE,
    )

    _LIST_PROCESSES = re.compile(
        r"^(?:(?:show|list)\s+(?:me\s+)?(?:the\s+)?running\s+processes|"
        r"what\s+processes\s+are\s+running)[.!?]?$",
        re.IGNORECASE,
    )

    _INSPECT_PORT = re.compile(
        r"^(?:(?:inspect|check|find|locate|show)(?:\s+me)?\s+"
        r"(?:(?:a|the)\s+)?(?:specific\s+)?port\s+"
        r"(?P<direct_port>\d+)|"
        r"(?:what(?:'s|\s+is)|which\s+process\s+is|show\s+(?:me\s+)?what(?:'s|\s+is)?)"
        r"\s+(?:using|running\s+on|listening\s+on)\s+port\s+"
        r"(?P<usage_port>\d+)|"
        r"(?:which|find(?:\s+me)?(?:\s+the)?)\s+process\s+"
        r"(?:is\s+)?(?:using|uses|owns|listening\s+on)\s+port\s+"
        r"(?P<process_port>\d+))[.!?]?$",
        re.IGNORECASE,
    )

    _LIST_PORTS = re.compile(
        r"^(?:(?:show|list)\s+(?:me\s+)?(?:the\s+)?listening\s+ports|"
        r"which\s+ports\s+are\s+listening)[.!?]?$",
        re.IGNORECASE,
    )

    _INCOMPLETE_PORT_INSPECTION = re.compile(
        r"^(?:inspect|check|find|locate|show)(?:\s+me)?\s+(?:a|the)?\s*"
        r"(?:specific\s+)?port[.!?]?$",
        re.IGNORECASE,
    )

    _RUN_ALL_SCRIPTS = re.compile(
        r"^run\s+all(?:\s+the)?(?:\s+(?P<language>python|javascript|typescript|"
        r"powershell|shell|bash|ruby|php|perl|lua|julia|r))?\s+scripts?"
        r"(?:\s+in\s+(?:this|the\s+current)\s+(?:folder|directory))?"
        r"(?P<options>.*?)[.!?]?$",
        re.IGNORECASE,
    )

    _LIST_RUNNABLE_SCRIPTS = re.compile(
        r"^(?:(?:list|show)(?:\s+me)?(?:\s+the)?\s+"
        r"(?:runnable\s+scripts?|script\s+execution\s+order|scripts?\s+in\s+run\s+order))"
        r"(?:\s+in\s+(?:this|the\s+current)\s+(?:folder|directory))?"
        r"(?P<options>.*?)[.!?]?$",
        re.IGNORECASE,
    )

    _RUN_SCRIPT = re.compile(
        r"^run(?:\s+the)?(?:\s+script)?\s+"
        r"(?:(?P<interpreter>python3?|node|npx|powershell|pwsh|bash|sh|ruby|php|"
        r"perl|lua|rscript|julia)\s+)?"
        r"(?P<path>.+?\.(?:ps1|cmd|bat|py|js|ts|sh|rb|php|pl|lua|r|jl))"
        r"(?P<arguments>\s+.*?)?[.!?]?$",
        re.IGNORECASE,
    )

    _SCRIPT_EXTENSIONS = {
        "python": ".py",
        "javascript": ".js",
        "typescript": ".ts",
        "powershell": ".ps1",
        "shell": ".sh",
        "bash": ".sh",
        "ruby": ".rb",
        "php": ".php",
        "perl": ".pl",
        "lua": ".lua",
        "julia": ".jl",
        "r": ".r",
    }

    def match(self, request: IntentRequest) -> IntentResponse:
        message = request.message.strip()
        normalized = message.casefold().rstrip(".!?")

        gitleaks_intent = self._match_gitleaks(
            normalized, request.execution_profile.working_directory
        )
        if gitleaks_intent is not None:
            return gitleaks_intent

        cloud_intent = self._match_azure_terraform(message, normalized)
        if cloud_intent is not None:
            return cloud_intent

        catalog_intent = self._command_catalog.match(message)
        if catalog_intent is not None:
            return catalog_intent

        if normalized in {"where am i", "show current directory", "show me the current directory"}:
            return self._matched("show_current_directory", {})

        if self._GO_UP_DIRECTORY.match(message):
            return self._matched("change_directory", {"target": ".."})

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

        edges_match = self._VIEW_FILE_EDGES.match(message)
        if edges_match:
            return self._matched(
                "view_file_edges",
                {
                    "path": self._clean_path(edges_match.group("path")),
                    "line_count": int(edges_match.group("count") or 20),
                },
            )

        start_match = self._VIEW_FILE_START.match(message)
        if start_match:
            return self._matched(
                "view_file_start",
                {
                    "path": self._clean_path(start_match.group("path")),
                    "line_count": int(start_match.group("count") or 20),
                },
            )

        end_match = self._VIEW_FILE_END.match(message)
        if end_match:
            return self._matched(
                "view_file_end",
                {
                    "path": self._clean_path(end_match.group("path")),
                    "line_count": int(end_match.group("count") or 20),
                },
            )

        if self._INCOMPLETE_FILE_PREVIEW.match(message):
            return self._clarification(
                "Which file should I preview? For example: "
                '"Preview the first/last lines of app.log."'
            )

        run_all_match = self._RUN_ALL_SCRIPTS.match(message)
        if run_all_match:
            options = run_all_match.group("options") or ""
            language = (run_all_match.group("language") or "").casefold()
            return self._matched(
                "run_scripts",
                self._script_collection_parameters(options, language),
            )

        list_scripts_match = self._LIST_RUNNABLE_SCRIPTS.match(message)
        if list_scripts_match:
            return self._matched(
                "list_scripts",
                self._script_collection_parameters(
                    list_scripts_match.group("options") or "", ""
                ),
            )

        run_script_match = self._RUN_SCRIPT.match(message)
        if run_script_match:
            path = self._clean_path(run_script_match.group("path"))
            raw_arguments = (run_script_match.group("arguments") or "").strip()
            try:
                arguments = self._split_script_arguments(raw_arguments)
            except ValueError:
                return self._clarification(
                    "I could not parse the script arguments. Check that quoted values "
                    "have matching quotation marks, then try again."
                )
            parameters: dict[str, object] = {"path": path}
            if arguments:
                parameters["arguments"] = arguments
            interpreter = (run_script_match.group("interpreter") or "").casefold()
            if interpreter:
                parameters["interpreter"] = interpreter
            return self._matched("run_script", parameters)

        if self._LIST_PROCESSES.match(message):
            return self._matched("list_processes", {})

        port_match = self._INSPECT_PORT.match(message)
        if port_match:
            port = (
                port_match.group("direct_port")
                or port_match.group("usage_port")
                or port_match.group("process_port")
            )
            return self._matched("inspect_port", {"port": int(port)})

        if self._LIST_PORTS.match(message):
            return self._matched("list_listening_ports", {})

        if self._INCOMPLETE_PORT_INSPECTION.match(message):
            return self._clarification(
                "Which port should I inspect? For example: "
                '"Inspect port 3000."'
            )

        if self._LIST_DIRECTORIES.match(message):
            return self._matched("list_directories", {"root": "."})

        if self._LIST_HIDDEN_ITEMS_WITH_DETAILS.match(message):
            return self._matched("list_hidden_items_with_details", {"root": "."})

        if self._LIST_DIRECTORY.match(message) or self._WHATS_IN_THIS_FOLDER.match(message):
            return self._matched("list_directory", {"root": "."})

        named_path_match = self._INSPECT_NAMED_PATH.match(message)
        if named_path_match:
            path = self._clean_path(named_path_match.group("root"))
            return self._matched("inspect_path", {"path": path})

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

        name_match = self._FIND_FILES_BY_NAME.match(message)
        if name_match:
            pattern = self._clean_path(name_match.group("pattern"))
            return self._matched(
                "find_files",
                {
                    "root": ".",
                    "pattern": pattern,
                    "recursive": True,
                },
            )

        make_directories_match = self._MAKE_DIRECTORIES.match(message)
        if make_directories_match:
            paths = self._clean_workspace_relative_paths(
                make_directories_match.group("paths")
            )
            if paths is None:
                return self._clarification(
                    "Provide comma-separated folder paths inside the active workspace. "
                    "Absolute paths and '..' parent traversal are not allowed."
                )
            return self._matched("make_directories", {"paths": paths})

        make_directory_match = self._MAKE_DIRECTORY.match(message)
        if make_directory_match:
            path = self._clean_workspace_relative_path(
                make_directory_match.group("path")
            )
            if path is None:
                return self._clarification(
                    "Choose a folder path inside the active workspace. Absolute paths and "
                    "'..' parent traversal are not allowed for this guided action."
                )
            return self._matched("make_directory", {"path": path})

        create_files_match = self._CREATE_FILES.match(message)
        if create_files_match:
            paths = self._clean_workspace_relative_paths(
                create_files_match.group("paths")
            )
            if paths is None:
                return self._clarification(
                    "Provide comma-separated file paths inside the active workspace. "
                    "Absolute paths and '..' parent traversal are not allowed."
                )
            return self._matched("create_files", {"paths": paths})

        create_file_match = self._CREATE_FILE.match(message)
        if create_file_match:
            path = self._clean_workspace_relative_path(create_file_match.group("path"))
            if path is None:
                return self._clarification(
                    "Choose a file path inside the active workspace. Absolute paths and '..' "
                    "parent traversal are not allowed for this guided action."
                )
            return self._matched("create_file", {"path": path})

        delete_file_match = self._DELETE_FILE.match(message)
        if delete_file_match:
            path = self._clean_workspace_relative_path(delete_file_match.group("path"))
            if path is None:
                return self._clarification(
                    "Choose a file path inside the active workspace. Absolute paths and '..' "
                    "parent traversal are not allowed for this guided action."
                )
            return self._matched("delete_file", {"path": path})

        backup_and_replace_match = self._BACKUP_AND_REPLACE_FILE.match(message)
        if backup_and_replace_match:
            target = self._clean_workspace_relative_path(
                backup_and_replace_match.group("target")
            )
            if target is None:
                return self._workspace_file_clarification("original file")
            backup = self._clean_backup_path(
                target, backup_and_replace_match.group("backup")
            )
            if backup is None:
                return self._workspace_file_clarification("backup file")
            replace_target_text = backup_and_replace_match.group("replace_target")
            if not replace_target_text.casefold().startswith("original"):
                replace_target = self._clean_workspace_relative_path(replace_target_text)
                if replace_target != target:
                    return self._clarification(
                        "The combined workflow must replace the same file it backs up. "
                        "Use 'original' or repeat the original workspace-relative path."
                    )
            source = self._clean_external_source_path(
                backup_and_replace_match.group("source")
            )
            if source is None:
                return self._clarification("Provide one replacement source file path.")
            if backup == target:
                return self._clarification(
                    "Choose a backup path different from the original file."
                )
            return self._matched(
                "backup_and_replace_file",
                {"target": target, "backup": backup, "source": source},
            )

        backup_file_match = self._BACKUP_FILE.match(message)
        if backup_file_match:
            source = self._clean_workspace_relative_path(
                backup_file_match.group("source")
            )
            if source is None:
                return self._workspace_file_clarification("source file")
            backup = self._clean_backup_path(source, backup_file_match.group("backup"))
            if backup is None:
                return self._workspace_file_clarification("backup file")
            if backup == source:
                return self._clarification(
                    "Choose a backup path different from the source file."
                )
            return self._matched(
                "backup_file", {"source": source, "backup": backup}
            )

        replace_file_match = self._REPLACE_FILE.match(message)
        if replace_file_match:
            target = self._clean_workspace_relative_path(
                replace_file_match.group("target")
            )
            if target is None:
                return self._workspace_file_clarification("target file")
            source = self._clean_external_source_path(replace_file_match.group("source"))
            if source is None:
                return self._clarification("Provide one replacement source file path.")
            return self._matched(
                "replace_file", {"target": target, "source": source}
            )

        transfer_from_directory = self._TRANSFER_FILE_FROM_DIRECTORY.match(message)
        if transfer_from_directory:
            filename = self._clean_external_source_path(
                transfer_from_directory.group("filename")
            )
            source_directory = self._clean_external_source_path(
                transfer_from_directory.group("source_directory")
            )
            if (
                filename is None
                or source_directory is None
                or "/" in filename
                or "\\" in filename
            ):
                return self._clarification(
                    "Provide one file name after 'file' and its source folder after "
                    "'from'. For example: copy file report.py from D:\\Downloads to "
                    "\\scripts."
                )
            separator = (
                "\\"
                if "\\" in source_directory
                or re.match(r"^[a-zA-Z]:", source_directory)
                else "/"
            )
            source = f"{source_directory.rstrip('/\\')}{separator}{filename}"
            return self._match_file_transfer(
                transfer_from_directory.group("operation"),
                source,
                transfer_from_directory.group("destination"),
            )

        transfer_path = self._TRANSFER_FILE_PATH.match(message)
        if transfer_path:
            source = self._clean_external_source_path(transfer_path.group("source"))
            if source is None:
                return self._clarification("Provide the path of one source file to transfer.")
            return self._match_file_transfer(
                transfer_path.group("operation"),
                source,
                transfer_path.group("destination"),
            )

        search_match = self._SEARCH_FILE_CONTENTS.match(message)
        if search_match:
            term = search_match.group("term").strip().strip("\"'")
            return self._matched("search_file_contents", {"term": term, "root": "."})

        inspect_match = self._INSPECT_FILE.match(message)
        if inspect_match:
            return self._matched(
                "inspect_file",
                {"path": self._clean_path(inspect_match.group("path"))},
            )

        count_match = self._COUNT_FILES.match(message)
        if count_match:
            return self._matched(
                "count_files",
                {
                    "root": ".",
                    "recursive": count_match.group("recursive") is not None,
                },
            )

        read_match = self._READ_FILE.match(message)
        if read_match:
            return self._matched(
                "read_file",
                {"path": self._clean_path(read_match.group("path"))},
            )

        return IntentResponse(
            matched=False,
            source="local",
            intent=None,
            message="This request is not supported by the local Phase 1 matcher.",
            requires_clarification=False,
        )

    def _match_gitleaks(
        self, normalized: str, working_directory: str
    ) -> IntentResponse | None:
        repository_name = re.split(r"[\\/]", working_directory.rstrip("\\/"))[-1]
        repository_name = re.sub(r"[^a-zA-Z0-9._-]+", "-", repository_name).strip("-.")
        repository_name = repository_name or "repository"
        report = f'$env:TEMP\\{repository_name}-gitleaks-report.json'

        command: str | None = None
        if normalized in {
            "gitleaks version",
            "show gitleaks version",
            "check gitleaks version",
        }:
            command = "gitleaks version"
        elif normalized in {
            "check for any leaks",
            "check for leaks",
            "check repository for leaks",
            "check repository for secrets",
            "scan for leaks",
            "scan for secrets",
            "scan git history for secrets",
            "run gitleaks",
            "gitleaks scan",
        }:
            command = (
                'gitleaks git --redact=100 --log-opts="--all" '
                f'--report-format json --report-path "{report}" .'
            )
        elif normalized in {
            "check current files for leaks",
            "check working tree for leaks",
            "scan current files for secrets",
            "scan working tree for secrets",
            "gitleaks dir",
        }:
            command = (
                "gitleaks dir --redact=100 "
                f'--report-format json --report-path "{report}" .'
            )
        elif normalized in {
            "check latest commit for leaks",
            "scan latest commit for secrets",
        }:
            command = (
                'gitleaks git --redact=100 --log-opts="-1" '
                f'--report-format json --report-path "{report}" .'
            )
        elif normalized in {
            "show gitleaks report",
            "show leak report",
            "open gitleaks report",
        }:
            command = f'Get-Content -LiteralPath "{report}" -Raw'

        if command is None:
            return None
        return self._matched(
            "catalog_command",
            {
                "catalog_id": "security_gitleaks",
                "command": command,
                "risk": "SecuritySafe",
            },
        )

    def _script_collection_parameters(
        self, options: str, language: str
    ) -> dict[str, str | bool]:
        normalized = options.casefold()
        if any(
            phrase in normalized
            for phrase in ("by name", "by filename", "alphabetically")
        ):
            order = "name"
        elif "oldest first" in normalized:
            order = "modified_asc"
        else:
            order = "modified_desc"

        parameters: dict[str, str | bool] = {
            "root": ".",
            "order": order,
            "recursive": any(
                phrase in normalized
                for phrase in ("recursively", "including subfolders", "including subdirectories")
            ),
        }
        extension = self._SCRIPT_EXTENSIONS.get(language)
        if extension:
            parameters["extension"] = extension
        return parameters

    def _match_azure_terraform(
        self, message: str, normalized: str
    ) -> IntentResponse | None:
        aliases: dict[str, set[str]] = {
            "azure_cli_version": {
                "azure version",
                "az version",
                "check azure version",
                "check azure cli version",
                "show azure cli version",
            },
            "terraform_version": {
                "terraform version",
                "check terraform version",
                "show terraform version",
            },
            "azure_account_show": {
                "show azure account",
                "show current azure account",
                "which azure account am i using",
            },
            "azure_subscription_list": {
                "list azure subscriptions",
                "show azure subscriptions",
                "show my azure subscriptions",
            },
            "azure_login": {"azure login", "login to azure", "sign in to azure"},
            "azure_login_device_code": {
                "azure login with device code",
                "login to azure with device code",
                "sign in to azure with device code",
            },
            "azure_account_clear": {
                "clear my azure login",
                "clear azure login",
                "log me out of azure completely",
                "log out of azure completely",
            },
            "terraform_init": {"terraform init", "initialize terraform"},
            "terraform_fmt": {"terraform fmt", "format terraform files"},
            "terraform_validate": {"terraform validate", "validate terraform"},
            "terraform_plan": {"terraform plan", "plan terraform changes"},
            "terraform_apply": {"terraform apply", "apply terraform changes"},
            "terraform_destroy": {"terraform destroy", "destroy terraform infrastructure"},
            "terraform_state_list": {"terraform state list", "list terraform state"},
            "terraform_output": {"terraform output", "show terraform output", "show terraform outputs"},
            # No provider-less alias like "list virtual machines" — every
            # Azure phrase requires "az"/"azure" so a future "aws" equivalent
            # (e.g. "list aws vms") can never collide with this one.
            "azure_vm_list": {"list azure vms", "show azure vms"},
            "azure_nsg_list": {
                "list azure network security groups",
                "list azure nsgs",
            },
        }
        for action, phrases in aliases.items():
            if normalized in phrases:
                return self._matched(action, {})

        # Every pattern below requires a literal "az" or "azure" marker,
        # never optional — this is what lets a future AWS adapter use the
        # same verb/noun shape ("start aws vm X") without ever colliding
        # with these Azure patterns ("start az vm X" / "start azure vm X").
        patterns: list[tuple[str, str, tuple[str, ...]]] = [
            (
                "azure_subscription_set",
                r"(?:switch|set|change)\s+(?:az|azure)\s+subscription(?:\s+to)?\s+(?P<subscription>.+)",
                ("subscription",),
            ),
            (
                "azure_provider_show",
                r"(?:check|show)(?:\s+if)?\s+(?:the\s+)?(?:az|azure)\s+(?P<namespace>.+?)\s+provider(?:\s+is)?\s+registered\??"
                r"|is\s+(?:the\s+)?(?:az|azure)\s+(?P<namespace2>.+?)\s+provider\s+registered\??",
                ("namespace", "namespace2"),
            ),
            (
                "azure_provider_register",
                r"register(?:\s+the)?\s+(?:az|azure)\s+(?P<namespace>.+?)\s+provider",
                ("namespace",),
            ),
            (
                "generate_ssh_key",
                r"(?:generate|create)(?:\s+an?)?\s+ssh\s+key(?:\s+(?:named|called|for))?\s+(?P<key_name>[^\s]+)(?:\s+(?:with\s+)?comment\s+[\"']?(?P<comment>.+?)[\"']?)?",
                ("key_name", "comment"),
            ),
            (
                "show_ssh_public_key",
                r"(?:show|display|read)(?:\s+the)?\s+(?:ssh\s+)?public\s+key(?:\s+(?:named|for))?\s+(?P<key_name>[^\s]+)",
                ("key_name",),
            ),
            (
                "azure_resource_group_create",
                r"create(?:\s+an?)?\s+(?:az|azure)\s+resource\s+group(?:\s+(?:named|called))?\s+(?P<resource_group>[^\s]+)\s+in\s+(?P<location>[^\s]+)",
                ("resource_group", "location"),
            ),
            (
                "azure_storage_account_create",
                r"create(?:\s+an?)?\s+(?:az|azure)\s+storage\s+account(?:\s+(?:named|called))?\s+(?P<account_name>[^\s]+)\s+in(?:\s+resource\s+group)?\s+(?P<resource_group>[^\s]+)",
                ("account_name", "resource_group"),
            ),
            (
                "azure_storage_container_create",
                r"create(?:\s+an?)?\s+(?:az|azure)\s+storage\s+container(?:\s+(?:named|called))?\s+(?P<container_name>[^\s]+)\s+in(?:\s+storage\s+account)?\s+(?P<account_name>[^\s]+)",
                ("container_name", "account_name"),
            ),
            (
                "azure_storage_keys_list",
                r"(?:show|list|get)(?:\s+the)?\s+(?:az|azure)\s+storage\s+(?:account\s+)?keys(?:\s+for)?\s+(?P<account_name>[^\s]+)\s+in(?:\s+resource\s+group)?\s+(?P<resource_group>[^\s]+)",
                ("account_name", "resource_group"),
            ),
            (
                "terraform_state_show",
                r"(?:terraform\s+state\s+show|show\s+terraform\s+state)(?:\s+for)?\s+(?P<address>.+)",
                ("address",),
            ),
            (
                "terraform_import",
                r"(?:terraform\s+import|import\s+terraform\s+resource)\s+(?P<address>[^\s]+)\s+(?P<resource_id>.+)",
                ("address", "resource_id"),
            ),
            (
                "azure_vm_show",
                r"(?:show|inspect)\s+(?:az|azure)\s+(?:vm|virtual\s+machine)\s+(?P<vm_name>[^\s]+)\s+in(?:\s+resource\s+group)?\s+(?P<resource_group>[^\s]+)",
                ("vm_name", "resource_group"),
            ),
            (
                "azure_resource_list",
                r"(?:list|show)\s+(?:az|azure)\s+resources\s+in(?:\s+resource\s+group)?\s+(?P<resource_group>[^\s]+)",
                ("resource_group",),
            ),
            (
                "azure_vm_start",
                r"start\s+(?:az|azure)\s+(?:vm|virtual\s+machine)\s+(?P<vm_name>[^\s]+)\s+in(?:\s+resource\s+group)?\s+(?P<resource_group>[^\s]+)",
                ("vm_name", "resource_group"),
            ),
            (
                "azure_vm_stop",
                r"stop\s+(?:az|azure)\s+(?:vm|virtual\s+machine)\s+(?P<vm_name>[^\s]+)\s+in(?:\s+resource\s+group)?\s+(?P<resource_group>[^\s]+)",
                ("vm_name", "resource_group"),
            ),
            (
                "azure_vm_deallocate",
                r"deallocate\s+(?:az|azure)\s+(?:vm|virtual\s+machine)\s+(?P<vm_name>[^\s]+)\s+in(?:\s+resource\s+group)?\s+(?P<resource_group>[^\s]+)",
                ("vm_name", "resource_group"),
            ),
            (
                "ssh_vm",
                r"(?:ssh|connect)\s+(?:to\s+)?(?P<admin>[^@\s]+)@(?P<ip>[^\s]+)\s+(?:using|with)(?:\s+ssh)?\s+key\s+(?P<key_name>[^\s]+)",
                ("admin", "ip", "key_name"),
            ),
            (
                "azure_resource_group_delete",
                r"delete\s+(?:az|azure)\s+resource\s+group(?:\s+(?:named|called))?\s+(?P<resource_group>[^\s]+)",
                ("resource_group",),
            ),
        ]
        for action, pattern, parameter_names in patterns:
            match = re.fullmatch(pattern, message.strip().rstrip(".!?"), re.IGNORECASE)
            if match:
                parameters = {
                    name: value.strip().strip("\"'")
                    for name in parameter_names
                    if (value := match.groupdict().get(name))
                }
                if action == "generate_ssh_key" and "comment" not in parameters:
                    parameters["comment"] = "TerminalMate SSH key"
                if action in {"azure_provider_show", "azure_provider_register"}:
                    raw_namespace = parameters.pop("namespace2", None) or parameters.get("namespace")
                    parameters["namespace"] = self._resolve_provider_namespace(raw_namespace)
                return self._matched(action, parameters)

        incomplete: list[tuple[str, str]] = [
            (r"(?:switch|set|change)\s+(?:az|azure)\s+subscription", "Which Azure subscription ID or name should I use?"),
            (r"create(?:\s+an?)?\s+(?:az|azure)\s+resource\s+group", "Provide both the resource-group name and Azure region, for example: Create azure resource group demo-rg in australiaeast."),
            (r"(?:start|stop|deallocate)\s+(?:az|azure)\s+(?:vm|virtual\s+machine)", "Provide both the VM name and resource group."),
            (r"terraform\s+state\s+show", "Which Terraform resource address should I inspect?"),
            (r"terraform\s+import", "Provide the Terraform address and Azure resource ID."),
        ]
        for pattern, guidance in incomplete:
            if re.fullmatch(pattern, normalized, re.IGNORECASE):
                return self._clarification(guidance)

        return None

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
            requires_clarification=False,
        )

    @staticmethod
    def _clarification(message: str) -> IntentResponse:
        return IntentResponse(
            matched=False,
            source="local",
            intent=None,
            message=message,
            requires_clarification=True,
        )

    @staticmethod
    def _clean_directory_target(raw_target: str) -> str:
        target = raw_target.strip().strip("\"'")
        target = re.sub(r"^(?:the|this)\s+", "", target, flags=re.IGNORECASE)
        target = re.sub(r"^(?:folder|directory)\s+", "", target, flags=re.IGNORECASE)
        target = re.sub(r"\s+(?:folder|directory)$", "", target, flags=re.IGNORECASE)
        target = target.strip()

        # Guided Windows navigation is workspace-relative by default. Users
        # commonly write "\backend\tests\" when they mean a child of the
        # active workspace; keep UNC paths and Linux absolute paths intact.
        if target.startswith("\\") and not target.startswith("\\\\"):
            target = target[1:]

        if target not in {"/", "\\"}:
            target = target.rstrip("/\\")
        return target

    @staticmethod
    def _clean_path(raw_path: str) -> str:
        return raw_path.strip().strip("\"'")

    @staticmethod
    def _clean_workspace_relative_path(raw_path: str) -> str | None:
        path = raw_path.strip().strip("\"'")
        if not path or path.startswith(("\\\\", "//")) or re.match(r"^[a-zA-Z]:", path):
            return None

        path = path.replace("\\", "/").lstrip("/")
        while path.startswith("./"):
            path = path[2:]

        parts = path.split("/")
        if not path or any(part in {"", ".", ".."} for part in parts):
            return None
        return "/".join(parts)

    @classmethod
    def _clean_workspace_relative_paths(cls, raw_paths: str) -> list[str] | None:
        candidates = raw_paths.split(",")
        if not candidates:
            return None

        paths: list[str] = []
        seen: set[str] = set()
        for candidate in candidates:
            candidate = re.sub(r"^\s*and\s+", "", candidate, flags=re.IGNORECASE)
            path = cls._clean_workspace_relative_path(candidate)
            if path is None:
                return None
            key = path.casefold()
            if key not in seen:
                seen.add(key)
                paths.append(path)
        return paths or None

    @staticmethod
    def _clean_external_source_path(raw_path: str) -> str | None:
        path = raw_path.strip().strip("\"'")
        return path or None

    def _clean_backup_path(self, source: str, raw_backup: str) -> str | None:
        backup = self._clean_workspace_relative_path(raw_backup)
        if backup is None:
            return None
        if "/" not in backup and "/" in source:
            return f"{source.rsplit('/', 1)[0]}/{backup}"
        return backup

    def _workspace_file_clarification(self, label: str) -> IntentResponse:
        return self._clarification(
            f"Choose a {label} path inside the active workspace. Absolute paths and '..' "
            "parent traversal are not allowed for that path."
        )

    def _match_file_transfer(
        self, operation: str, source: str, raw_destination: str
    ) -> IntentResponse:
        destination_text = raw_destination.strip().strip("\"'")
        if destination_text.casefold() in {
            ".",
            "here",
            "this folder",
            "this directory",
            "current folder",
            "current directory",
        }:
            destination = "."
        else:
            destination = self._clean_workspace_relative_path(destination_text)
        if destination is None:
            return self._clarification(
                "Choose a destination folder inside the active workspace. Absolute paths "
                "and '..' parent traversal are not allowed for the destination."
            )
        action = "copy_file" if operation.casefold() == "copy" else "move_file"
        return self._matched(
            action,
            {"source": source, "destination": destination},
        )

    @staticmethod
    def _split_script_arguments(raw_arguments: str) -> list[str]:
        if not raw_arguments:
            return []
        lexer = shlex.shlex(raw_arguments, posix=True)
        lexer.whitespace_split = True
        lexer.commenters = ""
        # Backslashes are ordinary path characters in cross-platform requests.
        lexer.escape = ""
        return list(lexer)

    # Common short names for the Azure resource providers a developer is
    # likely to ask about, resolved to the full namespace `az provider`
    # expects. Anything already shaped like "Microsoft.X" passes through
    # unchanged; anything else falls back to a best-effort "Microsoft.<Name>".
    _AZURE_PROVIDER_ALIASES = {
        "storage": "Microsoft.Storage",
        "compute": "Microsoft.Compute",
        "network": "Microsoft.Network",
        "keyvault": "Microsoft.KeyVault",
        "key vault": "Microsoft.KeyVault",
        "sql": "Microsoft.Sql",
        "web": "Microsoft.Web",
        "insights": "Microsoft.Insights",
    }

    @staticmethod
    def _resolve_provider_namespace(raw_namespace: str) -> str:
        cleaned = raw_namespace.strip().strip("\"'")
        alias = LocalIntentMatcher._AZURE_PROVIDER_ALIASES.get(cleaned.lower())
        if alias:
            return alias
        if cleaned.lower().startswith("microsoft."):
            return cleaned
        return f"Microsoft.{cleaned[:1].upper()}{cleaned[1:]}" if cleaned else cleaned
