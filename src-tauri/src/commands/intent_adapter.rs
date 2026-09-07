use crate::models::intent::CommandIntent;

/// Renders a sidecar-matched intent into the literal PowerShell command that
/// will actually run. The result still goes through the same
/// classify/execute boundary as any directly typed command — this only
/// decides *what* runs, never whether it is allowed to.
pub fn render_intent(intent: &CommandIntent) -> Result<String, String> {
    if intent.action == "catalog_command" {
        return render_catalog_command(intent);
    }
    if let Some(rendered) = render_cloud_intent(intent, false) {
        return rendered;
    }
    match intent.action.as_str() {
        "show_current_directory" => Ok("Get-Location".to_owned()),
        "change_directory" => {
            let target = string_param(intent, "target")?;
            Ok(render_change_directory(target))
        }
        "find_files" => {
            let root = intent
                .parameters
                .get("root")
                .and_then(|value| value.as_str())
                .unwrap_or(".");
            let pattern = string_param(intent, "pattern")?;
            let recursive = intent
                .parameters
                .get("recursive")
                .and_then(|value| value.as_bool())
                .unwrap_or(false);
            Ok(render_find_files(root, pattern, recursive))
        }
        "view_file_lines" => {
            let path = string_param(intent, "path")?;
            let start = int_param(intent, "start_line")?;
            let end = int_param(intent, "end_line")?;
            render_view_file_lines(path, start, end)
        }
        "view_file_start" => {
            let path = string_param(intent, "path")?;
            let count = int_param(intent, "line_count")?;
            render_file_preview(path, count, false)
        }
        "view_file_end" => {
            let path = string_param(intent, "path")?;
            let count = int_param(intent, "line_count")?;
            render_file_preview(path, count, true)
        }
        "view_file_edges" => {
            let path = string_param(intent, "path")?;
            let count = int_param(intent, "line_count")?;
            render_file_edges(path, count)
        }
        "read_file" => {
            let path = string_param(intent, "path")?;
            Ok(format!("Get-Content -LiteralPath '{}'", escape_single_quoted(path)))
        }
        "list_directory" => {
            let root = intent
                .parameters
                .get("root")
                .and_then(|value| value.as_str())
                .unwrap_or(".");
            Ok(format!("Get-ChildItem -LiteralPath '{}'", escape_single_quoted(root)))
        }
        "list_hidden_items_with_details" => {
            let root = optional_string_param(intent, "root", ".");
            Ok(format!(
                "Get-ChildItem -LiteralPath '{}' -Force | Select-Object Name, Mode",
                escape_single_quoted(root)
            ))
        }
        "list_directories" => {
            let root = optional_string_param(intent, "root", ".");
            Ok(format!(
                "Get-ChildItem -LiteralPath '{}' -Directory",
                escape_single_quoted(root)
            ))
        }
        "make_directory" => {
            let path = string_param(intent, "path")?;
            Ok(format!(
                "New-Item -ItemType Directory -Path '{}'",
                escape_single_quoted(path)
            ))
        }
        "make_directories" => render_multiple_creations(intent, false, false),
        "create_file" => {
            let path = string_param(intent, "path")?;
            Ok(format!(
                "New-Item -ItemType File -Path '{}' -ErrorAction Stop",
                escape_single_quoted(path)
            ))
        }
        "create_files" => render_multiple_creations(intent, false, true),
        "delete_file" => {
            let path = escape_single_quoted(string_param(intent, "path")?);
            Ok(format!(
                "$target = Get-Item -LiteralPath '{path}' -ErrorAction Stop; if ($target.PSIsContainer) {{ throw 'The selected path is a folder. Use an explicit folder-deletion workflow instead.' }}; Remove-Item -LiteralPath $target.FullName -ErrorAction Stop"
            ))
        }
        "copy_file" => render_file_transfer(intent, false, false),
        "move_file" => render_file_transfer(intent, false, true),
        "backup_file" => render_file_backup(intent, false),
        "replace_file" => render_file_replace(intent, false),
        "backup_and_replace_file" => render_backup_and_replace(intent, false),
        "search_file_contents" => {
            let term = string_param(intent, "term")?;
            let root = intent
                .parameters
                .get("root")
                .and_then(|value| value.as_str())
                .unwrap_or(".");
            Ok(format!(
                "Get-ChildItem -LiteralPath '{}' -Recurse -File -ErrorAction SilentlyContinue | Select-String -Pattern '{}'",
                escape_single_quoted(root),
                escape_single_quoted(term)
            ))
        }
        "inspect_file" => {
            let path = string_param(intent, "path")?;
            Ok(format!(
                "Get-Item -LiteralPath '{}' | Format-List FullName,Length,LastWriteTime,Attributes",
                escape_single_quoted(path)
            ))
        }
        "inspect_path" => {
            let path = string_param(intent, "path")?;
            Ok(render_inspect_path(path))
        }
        "count_files" => {
            let root = optional_string_param(intent, "root", ".");
            let recursive = bool_param(intent, "recursive", false);
            let recurse = if recursive { " -Recurse" } else { "" };
            Ok(format!(
                "Get-ChildItem -LiteralPath '{}' -File{} | Measure-Object | Select-Object -ExpandProperty Count",
                escape_single_quoted(root),
                recurse
            ))
        }
        "list_processes" => Ok(
            "Get-Process | Sort-Object CPU -Descending | Select-Object -First 25 Name,Id,CPU,WorkingSet"
                .to_owned(),
        ),
        "list_listening_ports" => Ok(
            "Get-NetTCPConnection -State Listen | Sort-Object LocalPort | Select-Object LocalAddress,LocalPort,OwningProcess"
                .to_owned(),
        ),
        "inspect_port" => {
            let port = int_param(intent, "port")?;
            if !(1..=65_535).contains(&port) {
                return Err(format!("Invalid TCP port: {port}."));
            }
            Ok(format!(
                "Get-NetTCPConnection -LocalPort {port} | Select-Object LocalAddress,LocalPort,RemoteAddress,RemotePort,State,OwningProcess"
            ))
        }
        "run_script" | "run_scripts" | "list_scripts" => {
            render_script_intent(intent, false)
        }
        other => Err(format!("Unsupported intent action: {other}")),
    }
}

fn render_catalog_command(intent: &CommandIntent) -> Result<String, String> {
    let catalog_id = string_param(intent, "catalog_id")?;
    if catalog_id.is_empty()
        || !catalog_id
            .chars()
            .all(|character| character.is_ascii_lowercase() || character.is_ascii_digit() || character == '_')
    {
        return Err("Invalid deterministic catalog identifier.".to_owned());
    }
    let command = string_param(intent, "command")?;
    if command.trim().is_empty() || command.contains(['\r', '\n']) {
        return Err("Catalog commands must be a non-empty single line.".to_owned());
    }
    Ok(command.to_owned())
}

pub fn render_intent_for_shell(
    intent: &CommandIntent,
    shell: &str,
) -> Result<String, String> {
    if shell.eq_ignore_ascii_case("bash") {
        render_bash_intent(intent)
    } else {
        render_intent(intent)
    }
}

fn render_bash_intent(intent: &CommandIntent) -> Result<String, String> {
    if let Some(rendered) = render_cloud_intent(intent, true) {
        return rendered;
    }
    match intent.action.as_str() {
        "show_current_directory" => Ok("pwd".to_owned()),
        "change_directory" => {
            let target = string_param(intent, "target")?;
            if target.eq_ignore_ascii_case("home") {
                Ok("cd -- \"$HOME\"".to_owned())
            } else {
                Ok(format!("cd -- {}", bash_literal(target)))
            }
        }
        "find_files" => {
            let root = optional_string_param(intent, "root", ".");
            let pattern = string_param(intent, "pattern")?;
            let depth = if bool_param(intent, "recursive", false) {
                ""
            } else {
                " -maxdepth 1"
            };
            Ok(format!(
                "find {}{} -type f -name {} -print",
                bash_literal(root),
                depth,
                bash_literal(pattern)
            ))
        }
        "view_file_lines" => {
            let path = string_param(intent, "path")?;
            let start = int_param(intent, "start_line")?;
            let end = int_param(intent, "end_line")?;
            if start < 1 || end < start {
                return Err(format!(
                    "Invalid line range: start {start} must be at least 1 and no greater than end {end}."
                ));
            }
            Ok(format!("sed -n '{start},{end}p' -- {}", bash_literal(path)))
        }
        "view_file_start" => {
            let path = string_param(intent, "path")?;
            let count = validated_preview_count(intent)?;
            Ok(format!("head -n {count} -- {}", bash_literal(path)))
        }
        "view_file_end" => {
            let path = string_param(intent, "path")?;
            let count = validated_preview_count(intent)?;
            Ok(format!("tail -n {count} -- {}", bash_literal(path)))
        }
        "view_file_edges" => {
            let path = string_param(intent, "path")?;
            let count = validated_preview_count(intent)?;
            let path = bash_literal(path);
            Ok(format!(
                "printf '%s\\n' '--- First {count} lines ---'; head -n {count} -- {path}; printf '%s\\n' '--- Last {count} lines ---'; tail -n {count} -- {path}"
            ))
        }
        "read_file" => Ok(format!(
            "cat -- {}",
            bash_literal(string_param(intent, "path")?)
        )),
        "list_directory" => Ok(format!(
            "ls -la -- {}",
            bash_literal(optional_string_param(intent, "root", "."))
        )),
        "list_hidden_items_with_details" => Ok(format!(
            "ls -la -- {}",
            bash_literal(optional_string_param(intent, "root", "."))
        )),
        "list_directories" => Ok(format!(
            "find {} -mindepth 1 -maxdepth 1 -type d -print",
            bash_literal(optional_string_param(intent, "root", "."))
        )),
        "make_directory" => Ok(format!(
            "mkdir -- {}",
            bash_literal(string_param(intent, "path")?)
        )),
        "make_directories" => render_multiple_creations(intent, true, false),
        "create_file" => {
            let path = bash_literal(string_param(intent, "path")?);
            Ok(format!(
                "if [ -e {path} ] || [ -L {path} ]; then printf 'Path already exists: %s\\n' {path} >&2; exit 1; fi; touch -- {path}"
            ))
        }
        "create_files" => render_multiple_creations(intent, true, true),
        "delete_file" => {
            let path = bash_literal(string_param(intent, "path")?);
            Ok(format!(
                "if [ -d {path} ]; then printf 'The selected path is a folder. Use an explicit folder-deletion workflow instead.\\n' >&2; exit 1; fi; rm -- {path}"
            ))
        }
        "copy_file" => render_file_transfer(intent, true, false),
        "move_file" => render_file_transfer(intent, true, true),
        "backup_file" => render_file_backup(intent, true),
        "replace_file" => render_file_replace(intent, true),
        "backup_and_replace_file" => render_backup_and_replace(intent, true),
        "search_file_contents" => Ok(format!(
            "grep -RIn -- {} {}",
            bash_literal(string_param(intent, "term")?),
            bash_literal(optional_string_param(intent, "root", "."))
        )),
        "inspect_file" => Ok(format!(
            "stat -- {}",
            bash_literal(string_param(intent, "path")?)
        )),
        "inspect_path" => {
            let path = bash_literal(string_param(intent, "path")?);
            Ok(format!(
                "if [ -d {path} ]; then printf 'Target: %s\\nType: Folder\\nContents:\\n' \"$(realpath -- {path})\"; ls -la -- {path}; elif [ -f {path} ]; then printf 'Target: %s\\nType: File\\n' \"$(realpath -- {path})\"; stat -- {path}; else printf 'Path not found: %s\\n' {path} >&2; exit 1; fi"
            ))
        }
        "count_files" => {
            let root = bash_literal(optional_string_param(intent, "root", "."));
            let depth = if bool_param(intent, "recursive", false) {
                ""
            } else {
                " -maxdepth 1"
            };
            Ok(format!("find {root}{depth} -type f -print | wc -l"))
        }
        "list_processes" => Ok(
            "ps -eo pid,comm,%cpu,%mem --sort=-%cpu | head -n 26".to_owned(),
        ),
        "list_listening_ports" => Ok("ss -ltnp".to_owned()),
        "inspect_port" => {
            let port = int_param(intent, "port")?;
            if !(1..=65_535).contains(&port) {
                return Err(format!("Invalid TCP port: {port}."));
            }
            Ok(format!("ss -ltnp 'sport = :{port}'"))
        }
        "run_script" | "run_scripts" | "list_scripts" => {
            render_script_intent(intent, true)
        }
        other => Err(format!("Unsupported intent action: {other}")),
    }
}

fn render_script_intent(intent: &CommandIntent, bash: bool) -> Result<String, String> {
    if intent.action == "run_script" {
        let path = string_param(intent, "path")?;
        let arguments = string_array_param(intent, "arguments")?;
        return if bash {
            render_bash_single_script(path, &arguments)
        } else {
            render_powershell_single_script(path, &arguments)
        };
    }

    let root = optional_string_param(intent, "root", ".");
    let order = optional_string_param(intent, "order", "modified_desc");
    if !matches!(order, "modified_desc" | "modified_asc" | "name") {
        return Err(format!("Unsupported script order: {order}."));
    }
    let recursive = bool_param(intent, "recursive", false);
    let extension = intent
        .parameters
        .get("extension")
        .and_then(|value| value.as_str())
        .filter(|value| !value.trim().is_empty());
    let list_only = intent.action == "list_scripts";

    if bash {
        render_bash_script_collection(root, order, recursive, extension, list_only)
    } else {
        render_powershell_script_collection(root, order, recursive, extension, list_only)
    }
}

fn render_powershell_single_script(path: &str, arguments: &[&str]) -> Result<String, String> {
    const EXTENSIONS: &[&str] = &[
        ".ps1", ".cmd", ".bat", ".py", ".js", ".ts", ".rb", ".php", ".pl",
        ".lua", ".r", ".jl",
    ];
    validate_script_path(path, EXTENSIONS, "PowerShell")?;
    let path = escape_single_quoted(normalize_powershell_script_path(path));
    Ok(format!(
        "$script = Get-Item -LiteralPath '{path}' -ErrorAction Stop; if ($script.PSIsContainer) {{ throw \"Expected a script file but found a folder: $($script.FullName)\" }}; {}",
        powershell_run_script("$script", arguments)
    ))
}

fn normalize_powershell_script_path(path: &str) -> &str {
    for separator in ['\\', '/'] {
        if let Some(relative) = path.strip_prefix(separator) {
            if !relative.starts_with('\\') && !relative.starts_with('/') {
                return relative;
            }
        }
    }

    path
}

fn render_powershell_script_collection(
    root: &str,
    order: &str,
    recursive: bool,
    extension: Option<&str>,
    list_only: bool,
) -> Result<String, String> {
    const EXTENSIONS: &[&str] = &[
        ".ps1", ".cmd", ".bat", ".py", ".js", ".ts", ".rb", ".php", ".pl",
        ".lua", ".r", ".jl",
    ];
    let extensions = validated_script_extensions(extension, EXTENSIONS, "PowerShell")?;
    let extension_list = extensions
        .iter()
        .map(|value| format!("'{}'", escape_single_quoted(value)))
        .collect::<Vec<_>>()
        .join(",");
    let recurse = if recursive { " -Recurse" } else { "" };
    let sort = match order {
        "name" => "Sort-Object Name,FullName",
        "modified_asc" => {
            "Sort-Object @{Expression='LastWriteTime';Descending=$false},FullName"
        }
        _ => "Sort-Object @{Expression='LastWriteTime';Descending=$true},FullName",
    };
    let discovery = format!(
        "$scripts = @(Get-ChildItem -LiteralPath '{}' -File{} | Where-Object {{ @({extension_list}) -contains $_.Extension.ToLowerInvariant() }} | {sort})",
        escape_single_quoted(root),
        recurse,
    );
    if list_only {
        return Ok(format!(
            "{discovery}; if ($scripts.Count -eq 0) {{ Write-Output 'No supported scripts found.' }} else {{ $scripts | Select-Object FullName,Extension,LastWriteTime }}"
        ));
    }
    Ok(format!(
        "{discovery}; if ($scripts.Count -eq 0) {{ Write-Output 'No supported scripts found.'; exit 0 }}; foreach ($script in $scripts) {{ {} }}",
        powershell_run_script("$script", &[])
    ))
}

fn powershell_run_script(variable: &str, arguments: &[&str]) -> String {
    let arguments = arguments
        .iter()
        .map(|value| format!(" '{}'", escape_single_quoted(value)))
        .collect::<String>();
    format!(
        "Write-Output ('>>> Running: ' + {variable}.FullName); $global:LASTEXITCODE = 0; switch ({variable}.Extension.ToLowerInvariant()) {{ '.ps1' {{ & {variable}.FullName{arguments} }} '.cmd' {{ & {variable}.FullName{arguments} }} '.bat' {{ & {variable}.FullName{arguments} }} '.py' {{ & python -u {variable}.FullName{arguments} }} '.js' {{ & node {variable}.FullName{arguments} }} '.ts' {{ & npx --no-install tsx {variable}.FullName{arguments} }} '.rb' {{ & ruby {variable}.FullName{arguments} }} '.php' {{ & php {variable}.FullName{arguments} }} '.pl' {{ & perl {variable}.FullName{arguments} }} '.lua' {{ & lua {variable}.FullName{arguments} }} '.r' {{ & Rscript {variable}.FullName{arguments} }} '.jl' {{ & julia {variable}.FullName{arguments} }} default {{ throw \"Unsupported script type: $({variable}.Extension)\" }} }}; if (-not $? -or $LASTEXITCODE -ne 0) {{ $code = if ($LASTEXITCODE -ne 0) {{ $LASTEXITCODE }} else {{ 1 }}; Write-Error (\"Script failed: $({variable}.FullName)\"); exit $code }}"
    )
}

fn render_bash_single_script(path: &str, arguments: &[&str]) -> Result<String, String> {
    const EXTENSIONS: &[&str] = &[
        ".sh", ".py", ".js", ".ts", ".rb", ".php", ".pl", ".lua", ".r", ".jl",
    ];
    validate_script_path(path, EXTENSIONS, "WSL Bash")?;
    let path = bash_literal(path);
    let arguments = arguments
        .iter()
        .map(|value| format!(" {}", bash_literal(value)))
        .collect::<String>();
    Ok(format!(
        "script=$(realpath -- {path}) || exit 1; [ -f \"$script\" ] || {{ printf 'Expected a script file: %s\\n' \"$script\" >&2; exit 1; }}; {}; tm_run_script \"$script\"{arguments}",
        bash_script_runner()
    ))
}

fn render_bash_script_collection(
    root: &str,
    order: &str,
    recursive: bool,
    extension: Option<&str>,
    list_only: bool,
) -> Result<String, String> {
    const EXTENSIONS: &[&str] = &[
        ".sh", ".py", ".js", ".ts", ".rb", ".php", ".pl", ".lua", ".r", ".jl",
    ];
    let extensions = validated_script_extensions(extension, EXTENSIONS, "WSL Bash")?;
    let expression = extensions
        .iter()
        .map(|value| format!("-iname {}", bash_literal(&format!("*{value}"))))
        .collect::<Vec<_>>()
        .join(" -o ");
    let depth = if recursive { "" } else { " -maxdepth 1" };
    let find = if order == "name" {
        format!(
            "find {}{depth} -type f \\( {expression} \\) -print0 | sort -z",
            bash_literal(root)
        )
    } else {
        let direction = if order == "modified_asc" { "-n" } else { "-nr" };
        format!(
            "find {}{depth} -type f \\( {expression} \\) -printf '%T@ %p\\0' | sort -z {direction} | cut -z -d ' ' -f 2-",
            bash_literal(root)
        )
    };
    if list_only {
        return Ok(format!(
            "found=0; while IFS= read -r -d '' script; do found=1; printf '%s\\n' \"$script\"; done < <({find}); [ \"$found\" -eq 1 ] || printf '%s\\n' 'No supported scripts found.'"
        ));
    }
    Ok(format!(
        "{}; found=0; while IFS= read -r -d '' script; do found=1; tm_run_script \"$script\" || exit $?; done < <({find}); [ \"$found\" -eq 1 ] || printf '%s\\n' 'No supported scripts found.'",
        bash_script_runner()
    ))
}

fn bash_script_runner() -> &'static str {
    "tm_run_script() { script=$1; shift; printf '>>> Running: %s\\n' \"$script\"; case \"${script,,}\" in *.sh) bash \"$script\" \"$@\" ;; *.py) python3 -u \"$script\" \"$@\" ;; *.js) node \"$script\" \"$@\" ;; *.ts) npx --no-install tsx \"$script\" \"$@\" ;; *.rb) ruby \"$script\" \"$@\" ;; *.php) php \"$script\" \"$@\" ;; *.pl) perl \"$script\" \"$@\" ;; *.lua) lua \"$script\" \"$@\" ;; *.r) Rscript \"$script\" \"$@\" ;; *.jl) julia \"$script\" \"$@\" ;; *) printf 'Unsupported script type: %s\\n' \"$script\" >&2; return 2 ;; esac; code=$?; [ \"$code\" -eq 0 ] || printf 'Script failed: %s (exit %s)\\n' \"$script\" \"$code\" >&2; return \"$code\"; }"
}

fn validated_script_extensions<'a>(
    extension: Option<&'a str>,
    supported: &'a [&'a str],
    runtime: &str,
) -> Result<Vec<&'a str>, String> {
    match extension {
        Some(value) => supported
            .iter()
            .copied()
            .find(|item| item.eq_ignore_ascii_case(value))
            .map(|matched| vec![matched])
            .ok_or_else(|| {
                format!(
                    "Script type {value} is not supported in {runtime}. Switch runtime or choose a supported script."
                )
            }),
        None => Ok(supported.to_vec()),
    }
}

fn validate_script_path(path: &str, supported: &[&str], runtime: &str) -> Result<(), String> {
    let normalized = path.to_lowercase();
    if supported
        .iter()
        .any(|extension| normalized.ends_with(extension))
    {
        return Ok(());
    }

    Err(format!(
        "Script {path} is not supported in {runtime}. Switch runtime or choose a supported script."
    ))
}

fn render_cloud_intent(
    intent: &CommandIntent,
    bash: bool,
) -> Option<Result<String, String>> {
    let quote = |value: &str| {
        if bash {
            bash_literal(value)
        } else {
            format!("'{}'", escape_single_quoted(value))
        }
    };
    let required = |name| string_param(intent, name).map(&quote);

    let command = match intent.action.as_str() {
        "azure_cli_version" => Ok("az --version".to_owned()),
        "terraform_version" => Ok("terraform --version".to_owned()),
        "azure_account_show" => Ok("az account show".to_owned()),
        "azure_subscription_list" => Ok("az account list --output table".to_owned()),
        "azure_login" => Ok("az login".to_owned()),
        "azure_login_device_code" => Ok("az login --use-device-code".to_owned()),
        "azure_account_clear" => Ok("az account clear".to_owned()),
        "azure_provider_show" => required("namespace").map(|namespace| {
            format!("az provider show --namespace {namespace} --query registrationState")
        }),
        "azure_provider_register" => required("namespace")
            .map(|namespace| format!("az provider register --namespace {namespace}")),
        "azure_subscription_set" => required("subscription").map(|subscription| {
            format!("az account set --subscription {subscription}")
        }),
        "generate_ssh_key" => {
            let key_name = string_param(intent, "key_name");
            let comment = required("comment");
            match (key_name, comment) {
                (Ok(key_name), Ok(comment)) if bash => Ok(format!(
                    "ssh-keygen -t ed25519 -f \"$HOME/{}\" -C {comment}",
                    key_name.replace(['/', '\\'], "_")
                )),
                (Ok(key_name), Ok(comment)) => Ok(format!(
                    "ssh-keygen -t ed25519 -f (Join-Path $HOME '{}') -C {comment}",
                    escape_single_quoted(&key_name.replace(['/', '\\'], "_"))
                )),
                (Err(error), _) | (_, Err(error)) => Err(error),
            }
        }
        "show_ssh_public_key" => string_param(intent, "key_name").map(|key_name| {
            let key_name = key_name.replace(['/', '\\'], "_");
            if bash {
                format!("cat -- \"$HOME/{key_name}.pub\"")
            } else {
                format!(
                    "Get-Content -LiteralPath (Join-Path $HOME '{}.pub')",
                    escape_single_quoted(&key_name)
                )
            }
        }),
        "azure_resource_group_create" => {
            match (required("resource_group"), required("location")) {
                (Ok(group), Ok(location)) => Ok(format!(
                    "az group create --name {group} --location {location}"
                )),
                (Err(error), _) | (_, Err(error)) => Err(error),
            }
        }
        "azure_storage_account_create" => {
            match (required("account_name"), required("resource_group")) {
                (Ok(account), Ok(group)) => Ok(format!(
                    "az storage account create --name {account} --resource-group {group} --sku Standard_LRS --encryption-services blob"
                )),
                (Err(error), _) | (_, Err(error)) => Err(error),
            }
        }
        "azure_storage_container_create" => {
            match (required("container_name"), required("account_name")) {
                (Ok(container), Ok(account)) => Ok(format!(
                    "az storage container create --name {container} --account-name {account}"
                )),
                (Err(error), _) | (_, Err(error)) => Err(error),
            }
        }
        "azure_storage_keys_list" => {
            match (required("account_name"), required("resource_group")) {
                (Ok(account), Ok(group)) => Ok(format!(
                    "az storage account keys list --account-name {account} --resource-group {group}"
                )),
                (Err(error), _) | (_, Err(error)) => Err(error),
            }
        }
        "terraform_init" => Ok("terraform init".to_owned()),
        "terraform_fmt" => Ok("terraform fmt".to_owned()),
        "terraform_validate" => Ok("terraform validate".to_owned()),
        "terraform_plan" => Ok("terraform plan".to_owned()),
        // -auto-approve is deliberate, not a shortcut around review: Terraform's
        // own interactive "type yes to confirm" prompt can never be answered by
        // TerminalMate (commands run with stdin closed, there is no way to send
        // input to a running process), so without this flag the command would
        // hang forever after printing its plan. TerminalMate's own typed-`RUN`
        // High Risk confirmation is the approval gate instead.
        "terraform_apply" => Ok("terraform apply -auto-approve".to_owned()),
        "terraform_destroy" => Ok("terraform destroy -auto-approve".to_owned()),
        "terraform_state_list" => Ok("terraform state list".to_owned()),
        "terraform_state_show" => required("address")
            .map(|address| format!("terraform state show {address}")),
        "terraform_output" => Ok("terraform output".to_owned()),
        "terraform_import" => match (required("address"), required("resource_id")) {
            (Ok(address), Ok(resource_id)) => {
                Ok(format!("terraform import {address} {resource_id}"))
            }
            (Err(error), _) | (_, Err(error)) => Err(error),
        },
        "azure_vm_list" => Ok("az vm list --show-details --output table".to_owned()),
        "azure_vm_show" => render_azure_named_resource(intent, &quote, "az vm show"),
        "azure_nsg_list" => Ok("az network nsg list --output table".to_owned()),
        "azure_resource_list" => required("resource_group").map(|group| {
            format!("az resource list --resource-group {group} --output table")
        }),
        "azure_vm_start" => render_azure_named_resource(intent, &quote, "az vm start"),
        "azure_vm_stop" => render_azure_named_resource(intent, &quote, "az vm stop"),
        "azure_vm_deallocate" => {
            render_azure_named_resource(intent, &quote, "az vm deallocate")
        }
        "ssh_vm" => match (
            string_param(intent, "admin"),
            string_param(intent, "ip"),
            string_param(intent, "key_name"),
        ) {
            (Ok(admin), Ok(ip), Ok(key_name)) => {
                let destination = quote(&format!("{admin}@{ip}"));
                let key_name = key_name.replace(['/', '\\'], "_");
                // BatchMode disables password/passphrase prompts (fail fast
                // instead of hanging on one TerminalMate cannot answer);
                // StrictHostKeyChecking=accept-new auto-trusts a new host's
                // key (the same TOFU model `git clone` over SSH already
                // uses) instead of hanging on the "are you sure?" prompt.
                // This still leaves a bare interactive login, which
                // `policy::is_unactionable_ssh_login` blocks with an
                // "open a new terminal" alternative — this render just
                // ensures that terminal opens cleanly instead of hanging.
                if bash {
                    Ok(format!(
                        "ssh -o BatchMode=yes -o StrictHostKeyChecking=accept-new -i \"$HOME/{key_name}\" {destination}"
                    ))
                } else {
                    Ok(format!(
                        "ssh -o BatchMode=yes -o StrictHostKeyChecking=accept-new -i (Join-Path $HOME '{}') {destination}",
                        escape_single_quoted(&key_name)
                    ))
                }
            }
            (Err(error), _, _) | (_, Err(error), _) | (_, _, Err(error)) => Err(error),
        },
        "azure_resource_group_delete" => required("resource_group").map(|group| {
            format!("az group delete --name {group} --yes --no-wait")
        }),
        "aws_identity_show" => Ok("aws sts get-caller-identity".to_owned()),
        "aws_budget_create" => match (
            string_param(intent, "account_id"),
            string_param(intent, "budget_name"),
            string_param(intent, "amount"),
        ) {
            (Ok(account_id), Ok(budget_name), Ok(amount)) => {
                let currency = optional_string_param(intent, "currency", "USD").to_uppercase();
                let time_unit = optional_string_param(intent, "time_unit", "MONTHLY").to_uppercase();
                let budget = serde_json::json!({
                    "BudgetName": budget_name,
                    "BudgetLimit": {"Amount": amount, "Unit": currency},
                    "TimeUnit": time_unit,
                    "BudgetType": "COST"
                })
                .to_string();
                Ok(format!(
                    "aws budgets create-budget --account-id {} --budget {}",
                    quote(account_id),
                    quote(&budget)
                ))
            }
            (Err(error), _, _) | (_, Err(error), _) | (_, _, Err(error)) => Err(error),
        },
        "aws_budget_alert_create" => match (
            string_param(intent, "account_id"),
            string_param(intent, "budget_name"),
            string_param(intent, "amount"),
            string_param(intent, "threshold"),
            string_param(intent, "notification_email"),
        ) {
            (Ok(account_id), Ok(budget_name), Ok(amount), Ok(threshold), Ok(email)) => {
                let threshold = match threshold.parse::<f64>() {
                    Ok(value) => value,
                    Err(_) => {
                        return Some(Err(
                            "Parameter \"threshold\" must be a number.".to_owned()
                        ));
                    }
                };
                let currency = optional_string_param(intent, "currency", "USD").to_uppercase();
                let time_unit = optional_string_param(intent, "time_unit", "MONTHLY").to_uppercase();
                let notification_type =
                    optional_string_param(intent, "notification_type", "ACTUAL").to_uppercase();
                let threshold_type =
                    optional_string_param(intent, "threshold_type", "PERCENTAGE").to_uppercase();
                let comparison_operator = optional_string_param(
                    intent,
                    "comparison_operator",
                    "GREATER_THAN",
                )
                .to_uppercase();
                let budget = serde_json::json!({
                    "BudgetName": budget_name,
                    "BudgetLimit": {"Amount": amount, "Unit": currency},
                    "TimeUnit": time_unit,
                    "BudgetType": "COST"
                })
                .to_string();
                let notifications = serde_json::json!([{
                    "Notification": {
                        "NotificationType": notification_type,
                        "ComparisonOperator": comparison_operator,
                        "Threshold": threshold,
                        "ThresholdType": threshold_type
                    },
                    "Subscribers": [{
                        "SubscriptionType": "EMAIL",
                        "Address": email
                    }]
                }])
                .to_string();
                Ok(format!(
                    "aws budgets create-budget --account-id {} --budget {} --notifications-with-subscribers {}",
                    quote(account_id),
                    quote(&budget),
                    quote(&notifications)
                ))
            }
            (Err(error), _, _, _, _)
            | (_, Err(error), _, _, _)
            | (_, _, Err(error), _, _)
            | (_, _, _, Err(error), _)
            | (_, _, _, _, Err(error)) => Err(error),
        },
        "aws_ec2_instance_list" => {
            let mut command = "aws ec2 describe-instances".to_owned();
            append_optional_argument(intent, "region", "--region", &quote, &mut command);
            Ok(command)
        }
        "aws_ec2_instance_create" => {
            let image = required("image_id");
            let instance_type = required("instance_type");
            let key_name = required("key_name");
            let subnet = required("subnet_id");
            let security_groups = string_array_param(intent, "security_group_ids");
            match (image, instance_type, key_name, subnet, security_groups) {
                (Ok(image), Ok(instance_type), Ok(key_name), Ok(subnet), Ok(groups))
                    if !groups.is_empty() =>
                {
                    let count = intent
                        .parameters
                        .get("count")
                        .and_then(|value| value.as_i64())
                        .unwrap_or(1);
                    let groups = groups
                        .iter()
                        .map(|group| quote(group))
                        .collect::<Vec<_>>()
                        .join(" ");
                    let mut command = format!(
                        "aws ec2 run-instances --image-id {image} --count {count} --instance-type {instance_type} --key-name {key_name} --security-group-ids {groups} --subnet-id {subnet}"
                    );
                    append_optional_argument(intent, "region", "--region", &quote, &mut command);
                    if let Some(name) = intent.parameters.get("name").and_then(|value| value.as_str()) {
                        let tags = format!("ResourceType=instance,Tags=[{{Key=Name,Value={name}}}]");
                        command.push_str(&format!(" --tag-specifications {}", quote(&tags)));
                    }
                    Ok(command)
                }
                (Ok(_), Ok(_), Ok(_), Ok(_), Ok(_)) => {
                    Err("Missing or empty \"security_group_ids\" parameter.".to_owned())
                }
                (Err(error), _, _, _, _)
                | (_, Err(error), _, _, _)
                | (_, _, Err(error), _, _)
                | (_, _, _, Err(error), _)
                | (_, _, _, _, Err(error)) => Err(error),
            }
        }
        "azure_budget_create" => match (
            required("budget_name"),
            required("amount"),
            required("start_date"),
            required("end_date"),
        ) {
            (Ok(name), Ok(amount), Ok(start), Ok(end)) => {
                let time_grain = quote(optional_string_param(intent, "time_grain", "monthly"));
                let mut command = format!(
                    "az consumption budget create --budget-name {name} --amount {amount} --category cost --start-date {start} --end-date {end} --time-grain {time_grain}"
                );
                append_optional_argument(
                    intent,
                    "resource_group",
                    "--resource-group",
                    &quote,
                    &mut command,
                );
                Ok(command)
            }
            (Err(error), _, _, _)
            | (_, Err(error), _, _)
            | (_, _, Err(error), _)
            | (_, _, _, Err(error)) => Err(error),
        },
        "azure_vm_create" => match (
            required("vm_name"),
            required("resource_group"),
            required("image"),
            required("admin_username"),
        ) {
            (Ok(name), Ok(group), Ok(image), Ok(admin)) => {
                let mut command = format!(
                    "az vm create --name {name} --resource-group {group} --image {image} --admin-username {admin}"
                );
                append_optional_argument(intent, "location", "--location", &quote, &mut command);
                append_optional_argument(intent, "size", "--size", &quote, &mut command);
                if bool_param(intent, "generate_ssh_keys", false) {
                    command.push_str(" --generate-ssh-keys");
                }
                Ok(command)
            }
            (Err(error), _, _, _)
            | (_, Err(error), _, _)
            | (_, _, Err(error), _)
            | (_, _, _, Err(error)) => Err(error),
        },
        "gcp_identity_show" => Ok(
            "gcloud auth list --filter=status:ACTIVE --format='value(account)'; gcloud config get-value project"
                .to_owned(),
        ),
        "gcp_budget_create" => match (
            required("billing_account"),
            required("display_name"),
            string_param(intent, "amount"),
        ) {
            (Ok(account), Ok(name), Ok(amount)) => {
                let currency = optional_string_param(intent, "currency", "USD").to_uppercase();
                Ok(format!(
                    "gcloud billing budgets create --billing-account {account} --display-name {name} --budget-amount {}",
                    quote(&format!("{amount}{currency}"))
                ))
            }
            (Err(error), _, _) | (_, Err(error), _) | (_, _, Err(error)) => Err(error),
        },
        "gcp_compute_instance_list" => {
            let mut command = "gcloud compute instances list".to_owned();
            append_optional_argument(intent, "project", "--project", &quote, &mut command);
            append_optional_argument(intent, "zone", "--zones", &quote, &mut command);
            Ok(command)
        }
        "gcp_compute_instance_create" => match (
            required("instance_name"),
            required("zone"),
            required("machine_type"),
        ) {
            (Ok(name), Ok(zone), Ok(machine_type)) => {
                let mut command = format!(
                    "gcloud compute instances create {name} --zone {zone} --machine-type {machine_type}"
                );
                append_optional_argument(intent, "project", "--project", &quote, &mut command);
                append_optional_argument(
                    intent,
                    "image_family",
                    "--image-family",
                    &quote,
                    &mut command,
                );
                append_optional_argument(
                    intent,
                    "image_project",
                    "--image-project",
                    &quote,
                    &mut command,
                );
                Ok(command)
            }
            (Err(error), _, _) | (_, Err(error), _) | (_, _, Err(error)) => Err(error),
        },
        "ssh_connect" => match (
            string_param(intent, "host"),
            string_param(intent, "remote_command"),
        ) {
            (Ok(host), Ok(remote_command)) => {
                let user = intent.parameters.get("user").and_then(|value| value.as_str());
                let destination = quote(&match user {
                    Some(user) => format!("{user}@{host}"),
                    None => host.to_owned(),
                });
                let mut command = "ssh -o BatchMode=yes -o StrictHostKeyChecking=accept-new".to_owned();
                if let Some(identity) = intent.parameters.get("identity_file").and_then(|value| value.as_str()) {
                    command.push_str(&format!(" -i {}", quote(identity)));
                }
                if let Some(port) = intent.parameters.get("port").and_then(|value| value.as_i64()) {
                    command.push_str(&format!(" -p {port}"));
                }
                command.push_str(&format!(" {destination} {}", quote(remote_command)));
                Ok(command)
            }
            (Err(error), _) | (_, Err(error)) => Err(error),
        },
        _ => return None,
    };

    Some(command.map(|command| {
        if azure_login_required(&intent.action) {
            wrap_azure_prerequisite(&command, bash)
        } else {
            command
        }
    }))
}

fn render_azure_named_resource(
    intent: &CommandIntent,
    quote: &impl Fn(&str) -> String,
    prefix: &str,
) -> Result<String, String> {
    let name = string_param(intent, "vm_name").map(quote)?;
    let group = string_param(intent, "resource_group").map(quote)?;
    Ok(format!("{prefix} --name {name} --resource-group {group}"))
}

fn azure_login_required(action: &str) -> bool {
    matches!(
        action,
        "azure_resource_group_create"
            | "azure_storage_account_create"
            | "azure_storage_container_create"
            | "azure_vm_list"
            | "azure_vm_show"
            | "azure_nsg_list"
            | "azure_resource_list"
            | "azure_vm_start"
            | "azure_vm_stop"
            | "azure_vm_deallocate"
            | "azure_provider_show"
            | "azure_provider_register"
            | "azure_budget_create"
            | "azure_vm_create"
    )
}

fn append_optional_argument(
    intent: &CommandIntent,
    parameter: &str,
    flag: &str,
    quote: &impl Fn(&str) -> String,
    command: &mut String,
) {
    if let Some(value) = intent
        .parameters
        .get(parameter)
        .and_then(|value| value.as_str())
        .filter(|value| !value.trim().is_empty())
    {
        command.push_str(&format!(" {flag} {}", quote(value)));
    }
}

fn wrap_azure_prerequisite(command: &str, bash: bool) -> String {
    if bash {
        format!(
            "az account show --output none >/dev/null 2>&1 || {{ printf '%s\\n' 'Azure login required. Run: Sign in to Azure' >&2; exit 1; }}; {command}"
        )
    } else {
        format!(
            "az account show --output none; if ($LASTEXITCODE -ne 0) {{ Write-Error 'Azure login required. Run: Sign in to Azure'; exit $LASTEXITCODE }}; {command}"
        )
    }
}

fn validated_preview_count(intent: &CommandIntent) -> Result<i64, String> {
    let count = int_param(intent, "line_count")?;
    if !(1..=500).contains(&count) {
        return Err(format!(
            "Invalid preview size: {count}. Choose between 1 and 500 lines."
        ));
    }
    Ok(count)
}

fn bash_literal(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\"'\"'"))
}

fn string_param<'a>(intent: &'a CommandIntent, name: &str) -> Result<&'a str, String> {
    intent
        .parameters
        .get(name)
        .and_then(|value| value.as_str())
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| format!("Missing or empty \"{name}\" parameter."))
}

fn string_array_param<'a>(intent: &'a CommandIntent, name: &str) -> Result<Vec<&'a str>, String> {
    let Some(value) = intent.parameters.get(name) else {
        return Ok(Vec::new());
    };
    let values = value
        .as_array()
        .ok_or_else(|| format!("Invalid \"{name}\" parameter: expected a list of strings."))?;
    values
        .iter()
        .map(|value| {
            value
                .as_str()
                .ok_or_else(|| format!("Invalid \"{name}\" parameter: expected a list of strings."))
        })
        .collect()
}

fn render_multiple_creations(
    intent: &CommandIntent,
    bash: bool,
    files: bool,
) -> Result<String, String> {
    let paths = string_array_param(intent, "paths")?;
    if paths.is_empty() {
        return Err("Missing or empty \"paths\" parameter.".to_string());
    }

    if bash {
        let values = paths
            .iter()
            .map(|path| bash_literal(path))
            .collect::<Vec<_>>()
            .join(" ");
        let create = if files {
            "touch -- \"${paths[@]}\""
        } else {
            "mkdir -- \"${paths[@]}\""
        };
        let label = if files { "files" } else { "folders" };
        return Ok(format!(
            "paths=({values}); for path in \"${{paths[@]}}\"; do if [ -e \"$path\" ] || [ -L \"$path\" ]; then printf 'Path already exists: %s\\n' \"$path\" >&2; exit 1; fi; done; {create}; printf 'Created {label}: %s\\n' \"${{paths[*]}}\""
        ));
    }

    let values = paths
        .iter()
        .map(|path| format!("'{}'", escape_single_quoted(path)))
        .collect::<Vec<_>>()
        .join(", ");
    let item_type = if files { "File" } else { "Directory" };
    let label = if files { "files" } else { "folders" };
    Ok(format!(
        "$paths = @({values}); $existing = @($paths | Where-Object {{ Test-Path -LiteralPath $_ }}); if ($existing.Count -gt 0) {{ throw \"Path already exists: $($existing -join ', ')\" }}; foreach ($path in $paths) {{ New-Item -ItemType {item_type} -Path $path -ErrorAction Stop | Out-Null }}; Write-Output \"Created {label}: $($paths -join ', ')\""
    ))
}

fn int_param(intent: &CommandIntent, name: &str) -> Result<i64, String> {
    intent
        .parameters
        .get(name)
        .and_then(|value| value.as_i64())
        .ok_or_else(|| format!("Missing or invalid \"{name}\" parameter."))
}

fn render_file_edges(path: &str, count: i64) -> Result<String, String> {
    if !(1..=500).contains(&count) {
        return Err(format!(
            "Line preview size must be between 1 and 500, received {count}."
        ));
    }

    let path = escape_single_quoted(path);
    Ok(format!(
        "Write-Output '--- First {count} lines ---'; Get-Content -LiteralPath '{path}' -TotalCount {count}; Write-Output '--- Last {count} lines ---'; Get-Content -LiteralPath '{path}' -Tail {count}"
    ))
}

fn optional_string_param<'a>(intent: &'a CommandIntent, name: &str, default: &'a str) -> &'a str {
    intent
        .parameters
        .get(name)
        .and_then(|value| value.as_str())
        .filter(|value| !value.trim().is_empty())
        .unwrap_or(default)
}

fn bool_param(intent: &CommandIntent, name: &str, default: bool) -> bool {
    intent
        .parameters
        .get(name)
        .and_then(|value| value.as_bool())
        .unwrap_or(default)
}

fn render_change_directory(target: &str) -> String {
    if target.eq_ignore_ascii_case("home") {
        return "Set-Location -Path $HOME".to_owned();
    }
    format!("Set-Location -LiteralPath '{}'", escape_single_quoted(target))
}

fn render_inspect_path(path: &str) -> String {
    let path = escape_single_quoted(path);
    format!(
        "Get-Item -LiteralPath '{path}' -ErrorAction Stop | ForEach-Object {{ $kind = if ($_.PSIsContainer) {{ 'Folder' }} else {{ 'File' }}; Write-Output \"Target: $($_.FullName)\"; Write-Output \"Type: $kind\"; if ($_.PSIsContainer) {{ Write-Output 'Contents:'; Get-ChildItem -LiteralPath $_.FullName }} else {{ $_ | Format-List FullName,Length,LastWriteTime,Attributes }} }}"
    )
}

fn render_file_transfer(
    intent: &CommandIntent,
    bash: bool,
    move_source: bool,
) -> Result<String, String> {
    let source = string_param(intent, "source")?;
    let destination = string_param(intent, "destination")?;
    let operation = if move_source { "Moved" } else { "Copied" };

    if bash {
        let source = bash_literal(&windows_path_for_bash(source));
        let destination = bash_literal(destination);
        let command = if move_source { "mv" } else { "cp" };
        return Ok(format!(
            "source={source}; destination={destination}; if [ ! -f \"$source\" ]; then printf 'Source is not a file: %s\\n' \"$source\" >&2; exit 1; fi; if [ ! -d \"$destination\" ]; then printf 'Destination folder not found: %s\\n' \"$destination\" >&2; exit 1; fi; target=\"$destination/$(basename -- \"$source\")\"; if [ -e \"$target\" ] || [ -L \"$target\" ]; then printf 'Destination file already exists: %s\\n' \"$target\" >&2; exit 1; fi; {command} -- \"$source\" \"$target\"; printf '{operation} file to: %s\\n' \"$target\""
        ));
    }

    let source = escape_single_quoted(source);
    let destination = escape_single_quoted(destination);
    let command = if move_source { "Move-Item" } else { "Copy-Item" };
    Ok(format!(
        "$source = Get-Item -LiteralPath '{source}' -ErrorAction Stop; if ($source.PSIsContainer) {{ throw 'The selected source is a folder. Choose one file.' }}; $destination = Get-Item -LiteralPath '{destination}' -ErrorAction Stop; if (-not $destination.PSIsContainer) {{ throw 'The selected destination is not a folder.' }}; $target = Join-Path -Path $destination.FullName -ChildPath $source.Name; if (Test-Path -LiteralPath $target) {{ throw \"Destination file already exists: $target\" }}; {command} -LiteralPath $source.FullName -Destination $target -ErrorAction Stop; Write-Output \"{operation} file to: $target\""
    ))
}

fn render_file_backup(intent: &CommandIntent, bash: bool) -> Result<String, String> {
    let source = string_param(intent, "source")?;
    let backup = string_param(intent, "backup")?;

    if bash {
        let source = bash_literal(source);
        let backup = bash_literal(backup);
        return Ok(format!(
            "source={source}; backup={backup}; if [ ! -f \"$source\" ]; then printf 'Source is not a file: %s\\n' \"$source\" >&2; exit 1; fi; backup_parent=$(dirname -- \"$backup\"); if [ ! -d \"$backup_parent\" ]; then printf 'Backup folder not found: %s\\n' \"$backup_parent\" >&2; exit 1; fi; if [ -e \"$backup\" ] || [ -L \"$backup\" ]; then printf 'Backup file already exists: %s\\n' \"$backup\" >&2; exit 1; fi; cp -- \"$source\" \"$backup\"; printf 'Backed up file to: %s\\n' \"$backup\""
        ));
    }

    let source = escape_single_quoted(source);
    let backup = escape_single_quoted(backup);
    Ok(format!(
        "$source = Get-Item -LiteralPath '{source}' -ErrorAction Stop; if ($source.PSIsContainer) {{ throw 'The selected source is a folder. Choose one file.' }}; $backupPath = [System.IO.Path]::GetFullPath((Join-Path -Path (Get-Location).Path -ChildPath '{backup}')); $backupParent = Split-Path -Parent $backupPath; if (-not (Test-Path -LiteralPath $backupParent -PathType Container)) {{ throw \"Backup folder not found: $backupParent\" }}; if (Test-Path -LiteralPath $backupPath) {{ throw \"Backup file already exists: $backupPath\" }}; Copy-Item -LiteralPath $source.FullName -Destination $backupPath -ErrorAction Stop; Write-Output \"Backed up file to: $backupPath\""
    ))
}

fn render_file_replace(intent: &CommandIntent, bash: bool) -> Result<String, String> {
    let target = string_param(intent, "target")?;
    let source = string_param(intent, "source")?;

    if bash {
        let target = bash_literal(target);
        let source = bash_literal(&windows_path_for_bash(source));
        return Ok(format!(
            "target={target}; source={source}; if [ ! -f \"$target\" ]; then printf 'Target is not a file: %s\\n' \"$target\" >&2; exit 1; fi; if [ ! -f \"$source\" ]; then printf 'Replacement source is not a file: %s\\n' \"$source\" >&2; exit 1; fi; if [ \"$(realpath -- \"$target\")\" = \"$(realpath -- \"$source\")\" ]; then printf 'Replacement source and target are the same file.\\n' >&2; exit 1; fi; cp --force -- \"$source\" \"$target\"; printf 'Replaced file: %s\\n' \"$target\""
        ));
    }

    let target = escape_single_quoted(target);
    let source = escape_single_quoted(source);
    Ok(format!(
        "$target = Get-Item -LiteralPath '{target}' -ErrorAction Stop; if ($target.PSIsContainer) {{ throw 'The selected target is a folder. Choose one file.' }}; $source = Get-Item -LiteralPath '{source}' -ErrorAction Stop; if ($source.PSIsContainer) {{ throw 'The replacement source is a folder. Choose one file.' }}; if ($target.FullName -eq $source.FullName) {{ throw 'Replacement source and target are the same file.' }}; Copy-Item -LiteralPath $source.FullName -Destination $target.FullName -Force -ErrorAction Stop; Write-Output \"Replaced file: $($target.FullName)\""
    ))
}

fn render_backup_and_replace(intent: &CommandIntent, bash: bool) -> Result<String, String> {
    let target = string_param(intent, "target")?;
    let backup = string_param(intent, "backup")?;
    let source = string_param(intent, "source")?;

    if bash {
        let target = bash_literal(target);
        let backup = bash_literal(backup);
        let source = bash_literal(&windows_path_for_bash(source));
        return Ok(format!(
            "target={target}; backup={backup}; source={source}; if [ ! -f \"$target\" ]; then printf 'Target is not a file: %s\\n' \"$target\" >&2; exit 1; fi; if [ ! -f \"$source\" ]; then printf 'Replacement source is not a file: %s\\n' \"$source\" >&2; exit 1; fi; backup_parent=$(dirname -- \"$backup\"); if [ ! -d \"$backup_parent\" ]; then printf 'Backup folder not found: %s\\n' \"$backup_parent\" >&2; exit 1; fi; if [ -e \"$backup\" ] || [ -L \"$backup\" ]; then printf 'Backup file already exists: %s\\n' \"$backup\" >&2; exit 1; fi; if [ \"$(realpath -- \"$target\")\" = \"$(realpath -- \"$source\")\" ]; then printf 'Replacement source and target are the same file.\\n' >&2; exit 1; fi; cp -- \"$target\" \"$backup\" && cp --force -- \"$source\" \"$target\"; printf 'Backed up file to: %s\\nReplaced file: %s\\n' \"$backup\" \"$target\""
        ));
    }

    let target = escape_single_quoted(target);
    let backup = escape_single_quoted(backup);
    let source = escape_single_quoted(source);
    Ok(format!(
        "$target = Get-Item -LiteralPath '{target}' -ErrorAction Stop; if ($target.PSIsContainer) {{ throw 'The selected target is a folder. Choose one file.' }}; $source = Get-Item -LiteralPath '{source}' -ErrorAction Stop; if ($source.PSIsContainer) {{ throw 'The replacement source is a folder. Choose one file.' }}; if ($target.FullName -eq $source.FullName) {{ throw 'Replacement source and target are the same file.' }}; $backupPath = [System.IO.Path]::GetFullPath((Join-Path -Path (Get-Location).Path -ChildPath '{backup}')); $backupParent = Split-Path -Parent $backupPath; if (-not (Test-Path -LiteralPath $backupParent -PathType Container)) {{ throw \"Backup folder not found: $backupParent\" }}; if (Test-Path -LiteralPath $backupPath) {{ throw \"Backup file already exists: $backupPath\" }}; Copy-Item -LiteralPath $target.FullName -Destination $backupPath -ErrorAction Stop; Copy-Item -LiteralPath $source.FullName -Destination $target.FullName -Force -ErrorAction Stop; Write-Output \"Backed up file to: $backupPath\"; Write-Output \"Replaced file: $($target.FullName)\""
    ))
}

fn windows_path_for_bash(path: &str) -> String {
    let raw = path.strip_prefix(r"\\?\").unwrap_or(path);
    let bytes = raw.as_bytes();
    if bytes.len() < 3
        || bytes[1] != b':'
        || (bytes[2] != b'\\' && bytes[2] != b'/')
        || !(bytes[0] as char).is_ascii_alphabetic()
    {
        return path.to_owned();
    }

    let drive = (bytes[0] as char).to_ascii_lowercase();
    let remainder = raw[3..].replace('\\', "/");
    if remainder.is_empty() {
        format!("/mnt/{drive}")
    } else {
        format!("/mnt/{drive}/{remainder}")
    }
}

fn render_find_files(root: &str, pattern: &str, recursive: bool) -> String {
    let mut command = format!(
        "Get-ChildItem -LiteralPath '{}' -Filter '{}' -File",
        escape_single_quoted(root),
        escape_single_quoted(pattern)
    );
    if recursive {
        command.push_str(" -Recurse -ErrorAction SilentlyContinue");
    }
    command
}

fn render_view_file_lines(path: &str, start: i64, end: i64) -> Result<String, String> {
    if start < 1 || end < start {
        return Err(format!(
            "Invalid line range: start {start} must be at least 1 and no greater than end {end}."
        ));
    }
    let count = end - start + 1;
    Ok(format!(
        "Get-Content -LiteralPath '{}' | Select-Object -Skip {} -First {}",
        escape_single_quoted(path),
        start - 1,
        count
    ))
}

fn render_file_preview(path: &str, count: i64, from_end: bool) -> Result<String, String> {
    if !(1..=500).contains(&count) {
        return Err(format!(
            "Invalid preview size: {count}. Choose between 1 and 500 lines."
        ));
    }
    if from_end {
        Ok(format!(
            "Get-Content -LiteralPath '{}' -Tail {}",
            escape_single_quoted(path),
            count
        ))
    } else {
        Ok(format!(
            "Get-Content -LiteralPath '{}' | Select-Object -First {}",
            escape_single_quoted(path),
            count
        ))
    }
}

fn escape_single_quoted(value: &str) -> String {
    value.replace('\'', "''")
}

#[cfg(test)]
mod tests {
    use super::{render_intent, render_intent_for_shell};
    use crate::models::intent::CommandIntent;
    use serde_json::json;

    fn intent(action: &str, parameters: serde_json::Value) -> CommandIntent {
        CommandIntent {
            schema_version: 1,
            action: action.to_owned(),
            parameters,
        }
    }

    #[test]
    fn renders_show_current_directory() {
        let rendered = render_intent(&intent("show_current_directory", json!({}))).unwrap();
        assert_eq!(rendered, "Get-Location");
    }

    #[test]
    fn renders_change_directory_to_home() {
        let rendered =
            render_intent(&intent("change_directory", json!({"target": "home"}))).unwrap();
        assert_eq!(rendered, "Set-Location -Path $HOME");
    }

    #[test]
    fn renders_change_directory_to_named_target() {
        let rendered =
            render_intent(&intent("change_directory", json!({"target": "downloads"}))).unwrap();
        assert_eq!(rendered, "Set-Location -LiteralPath 'downloads'");
    }

    #[test]
    fn escapes_single_quotes_in_change_directory_target() {
        let rendered = render_intent(&intent(
            "change_directory",
            json!({"target": "o'brien's folder"}),
        ))
        .unwrap();
        assert_eq!(
            rendered,
            "Set-Location -LiteralPath 'o''brien''s folder'"
        );
    }

    #[test]
    fn renders_find_files_non_recursive() {
        let rendered = render_intent(&intent(
            "find_files",
            json!({"root": ".", "pattern": "*.txt", "recursive": false}),
        ))
        .unwrap();
        assert_eq!(rendered, "Get-ChildItem -LiteralPath '.' -Filter '*.txt' -File");
    }

    #[test]
    fn renders_find_files_recursive() {
        let rendered = render_intent(&intent(
            "find_files",
            json!({"root": ".", "pattern": "*.rs", "recursive": true}),
        ))
        .unwrap();
        assert_eq!(
            rendered,
            "Get-ChildItem -LiteralPath '.' -Filter '*.rs' -File -Recurse -ErrorAction SilentlyContinue"
        );
    }

    #[test]
    fn renders_view_file_lines() {
        let rendered = render_intent(&intent(
            "view_file_lines",
            json!({"path": "sample.txt", "start_line": 20, "end_line": 100}),
        ))
        .unwrap();
        assert_eq!(
            rendered,
            "Get-Content -LiteralPath 'sample.txt' | Select-Object -Skip 19 -First 81"
        );
    }

    #[test]
    fn rejects_view_file_lines_with_inverted_range() {
        let error = render_intent(&intent(
            "view_file_lines",
            json!({"path": "sample.txt", "start_line": 100, "end_line": 20}),
        ))
        .unwrap_err();
        assert!(error.contains("Invalid line range"));
    }

    #[test]
    fn rejects_view_file_lines_with_zero_start() {
        let error = render_intent(&intent(
            "view_file_lines",
            json!({"path": "sample.txt", "start_line": 0, "end_line": 5}),
        ))
        .unwrap_err();
        assert!(error.contains("Invalid line range"));
    }

    #[test]
    fn renders_change_directory_up_one_level() {
        let rendered =
            render_intent(&intent("change_directory", json!({"target": ".."}))).unwrap();
        assert_eq!(rendered, "Set-Location -LiteralPath '..'");
    }

    #[test]
    fn renders_list_directory() {
        let rendered = render_intent(&intent("list_directory", json!({"root": "."}))).unwrap();
        assert_eq!(rendered, "Get-ChildItem -LiteralPath '.'");
    }

    #[test]
    fn renders_hidden_items_with_name_and_mode_for_each_shell() {
        let command = intent("list_hidden_items_with_details", json!({"root": "."}));

        assert_eq!(
            render_intent_for_shell(&command, "powershell").unwrap(),
            "Get-ChildItem -LiteralPath '.' -Force | Select-Object Name, Mode"
        );
        assert_eq!(
            render_intent_for_shell(&command, "bash").unwrap(),
            "ls -la -- '.'"
        );
    }

    #[test]
    fn renders_inspect_named_path() {
        let rendered =
            render_intent(&intent("inspect_path", json!({"path": ".claude"}))).unwrap();
        assert_eq!(
            rendered,
            "Get-Item -LiteralPath '.claude' -ErrorAction Stop | ForEach-Object { $kind = if ($_.PSIsContainer) { 'Folder' } else { 'File' }; Write-Output \"Target: $($_.FullName)\"; Write-Output \"Type: $kind\"; if ($_.PSIsContainer) { Write-Output 'Contents:'; Get-ChildItem -LiteralPath $_.FullName } else { $_ | Format-List FullName,Length,LastWriteTime,Attributes } }"
        );
    }

    #[test]
    fn renders_make_directory() {
        let rendered =
            render_intent(&intent("make_directory", json!({"path": "drafts"}))).unwrap();
        assert_eq!(rendered, "New-Item -ItemType Directory -Path 'drafts'");
    }

    #[test]
    fn renders_multiple_folder_creation_for_both_shells() {
        let create = intent(
            "make_directories",
            json!({"paths": ["components", "api"]}),
        );

        let powershell = render_intent(&create).unwrap();
        assert!(powershell.contains("$paths = @('components', 'api')"));
        assert!(powershell.contains("New-Item -ItemType Directory"));
        assert!(powershell.contains("Path already exists"));

        let bash = render_intent_for_shell(&create, "bash").unwrap();
        assert!(bash.contains("paths=('components' 'api')"));
        assert!(bash.contains("mkdir -- \"${paths[@]}\""));
        assert!(bash.contains("Path already exists"));
    }

    #[test]
    fn renders_multiple_file_creation_for_both_shells() {
        let create = intent(
            "create_files",
            json!({"paths": ["src/index.ts", "src/app.ts"]}),
        );

        let powershell = render_intent(&create).unwrap();
        assert!(powershell.contains("$paths = @('src/index.ts', 'src/app.ts')"));
        assert!(powershell.contains("New-Item -ItemType File"));

        let bash = render_intent_for_shell(&create, "bash").unwrap();
        assert!(bash.contains("paths=('src/index.ts' 'src/app.ts')"));
        assert!(bash.contains("touch -- \"${paths[@]}\""));
    }

    #[test]
    fn escapes_single_quotes_in_make_directory_path() {
        let rendered =
            render_intent(&intent("make_directory", json!({"path": "o'brien"}))).unwrap();
        assert_eq!(rendered, "New-Item -ItemType Directory -Path 'o''brien'");
    }

    #[test]
    fn renders_workspace_relative_file_creation_for_both_shells() {
        let create = intent(
            "create_file",
            json!({"path": "scripts/freeze_holdout_source_sample.py"}),
        );

        assert_eq!(
            render_intent(&create).unwrap(),
            "New-Item -ItemType File -Path 'scripts/freeze_holdout_source_sample.py' -ErrorAction Stop"
        );
        assert_eq!(
            render_intent_for_shell(&create, "bash").unwrap(),
            "if [ -e 'scripts/freeze_holdout_source_sample.py' ] || [ -L 'scripts/freeze_holdout_source_sample.py' ]; then printf 'Path already exists: %s\\n' 'scripts/freeze_holdout_source_sample.py' >&2; exit 1; fi; touch -- 'scripts/freeze_holdout_source_sample.py'"
        );
    }

    #[test]
    fn renders_file_only_deletion_for_both_shells() {
        let delete = intent(
            "delete_file",
            json!({"path": "scripts/freeze_holdout_source_sample.py"}),
        );

        assert_eq!(
            render_intent(&delete).unwrap(),
            "$target = Get-Item -LiteralPath 'scripts/freeze_holdout_source_sample.py' -ErrorAction Stop; if ($target.PSIsContainer) { throw 'The selected path is a folder. Use an explicit folder-deletion workflow instead.' }; Remove-Item -LiteralPath $target.FullName -ErrorAction Stop"
        );
        assert_eq!(
            render_intent_for_shell(&delete, "bash").unwrap(),
            "if [ -d 'scripts/freeze_holdout_source_sample.py' ]; then printf 'The selected path is a folder. Use an explicit folder-deletion workflow instead.\\n' >&2; exit 1; fi; rm -- 'scripts/freeze_holdout_source_sample.py'"
        );
    }

    #[test]
    fn renders_guarded_file_copy_for_powershell_and_bash() {
        let copy = intent(
            "copy_file",
            json!({
                "source": r"D:\Data\Downloads\freeze_external_adjudication.py",
                "destination": "scripts"
            }),
        );

        let powershell = render_intent(&copy).unwrap();
        assert!(powershell.contains(
            "Get-Item -LiteralPath 'D:\\Data\\Downloads\\freeze_external_adjudication.py'"
        ));
        assert!(powershell.contains("Get-Item -LiteralPath 'scripts'"));
        assert!(powershell.contains("Destination file already exists"));
        assert!(powershell.contains("Copy-Item -LiteralPath $source.FullName"));

        let bash = render_intent_for_shell(&copy, "bash").unwrap();
        assert!(bash.contains(
            "source='/mnt/d/Data/Downloads/freeze_external_adjudication.py'"
        ));
        assert!(bash.contains("destination='scripts'"));
        assert!(bash.contains("cp -- \"$source\" \"$target\""));
    }

    #[test]
    fn renders_guarded_file_move_without_silent_overwrite() {
        let move_file = intent(
            "move_file",
            json!({
                "source": r"D:\Data\Downloads\freeze_external_adjudication.py",
                "destination": "scripts"
            }),
        );

        let powershell = render_intent(&move_file).unwrap();
        assert!(powershell.contains("Destination file already exists"));
        assert!(powershell.contains("Move-Item -LiteralPath $source.FullName"));

        let bash = render_intent_for_shell(&move_file, "bash").unwrap();
        assert!(bash.contains("mv -- \"$source\" \"$target\""));
    }

    #[test]
    fn renders_guarded_file_backup_for_both_shells() {
        let backup = intent(
            "backup_file",
            json!({
                "source": "app/api/routes/ask.py",
                "backup": "app/api/routes/ask_before_evidence_adequacy.py"
            }),
        );

        let powershell = render_intent(&backup).unwrap();
        assert!(powershell.contains("Get-Item -LiteralPath 'app/api/routes/ask.py'"));
        assert!(powershell.contains("Backup file already exists"));
        assert!(powershell.contains(
            "Copy-Item -LiteralPath $source.FullName -Destination $backupPath"
        ));
        assert!(!powershell.contains("-Force"));

        let bash = render_intent_for_shell(&backup, "bash").unwrap();
        assert!(bash.contains("source='app/api/routes/ask.py'"));
        assert!(bash.contains("backup='app/api/routes/ask_before_evidence_adequacy.py'"));
        assert!(bash.contains("Backup file already exists"));
        assert!(bash.contains("cp -- \"$source\" \"$backup\""));
    }

    #[test]
    fn renders_explicit_file_replacement_as_forceful_and_guarded() {
        let replace = intent(
            "replace_file",
            json!({
                "target": "app/api/routes/ask.py",
                "source": r"D:\Data\Downloads\ask_evidence_adequacy_candidate.py"
            }),
        );

        let powershell = render_intent(&replace).unwrap();
        assert!(powershell.contains("Get-Item -LiteralPath 'app/api/routes/ask.py'"));
        assert!(powershell.contains(
            "Get-Item -LiteralPath 'D:\\Data\\Downloads\\ask_evidence_adequacy_candidate.py'"
        ));
        assert!(powershell.contains("Replacement source and target are the same file"));
        assert!(powershell.contains(
            "Copy-Item -LiteralPath $source.FullName -Destination $target.FullName -Force"
        ));

        let bash = render_intent_for_shell(&replace, "bash").unwrap();
        assert!(bash.contains(
            "source='/mnt/d/Data/Downloads/ask_evidence_adequacy_candidate.py'"
        ));
        assert!(bash.contains("cp --force -- \"$source\" \"$target\""));
    }

    #[test]
    fn renders_combined_backup_before_replacement() {
        let workflow = intent(
            "backup_and_replace_file",
            json!({
                "target": "app/api/routes/ask.py",
                "backup": "app/api/routes/ask_before_evidence_adequacy.py",
                "source": r"D:\Data\Downloads\ask_evidence_adequacy_candidate.py"
            }),
        );

        let powershell = render_intent(&workflow).unwrap();
        let backup_index = powershell
            .find("Copy-Item -LiteralPath $target.FullName -Destination $backupPath")
            .unwrap();
        let replace_index = powershell
            .find("Copy-Item -LiteralPath $source.FullName -Destination $target.FullName -Force")
            .unwrap();
        assert!(backup_index < replace_index);
        assert!(powershell.contains("Backup file already exists"));

        let bash = render_intent_for_shell(&workflow, "bash").unwrap();
        assert!(bash.contains(
            "cp -- \"$target\" \"$backup\" && cp --force -- \"$source\" \"$target\""
        ));
    }

    #[test]
    fn renders_search_file_contents() {
        let rendered = render_intent(&intent(
            "search_file_contents",
            json!({"term": "TODO", "root": "."}),
        ))
        .unwrap();
        assert_eq!(
            rendered,
            "Get-ChildItem -LiteralPath '.' -Recurse -File -ErrorAction SilentlyContinue | Select-String -Pattern 'TODO'"
        );
    }

    #[test]
    fn renders_read_file() {
        let rendered =
            render_intent(&intent("read_file", json!({"path": "package.json"}))).unwrap();
        assert_eq!(rendered, "Get-Content -LiteralPath 'package.json'");
    }

    #[test]
    fn renders_file_start_preview() {
        let rendered = render_intent(&intent(
            "view_file_start",
            json!({"path": "README.md", "line_count": 30}),
        ))
        .unwrap();
        assert_eq!(
            rendered,
            "Get-Content -LiteralPath 'README.md' | Select-Object -First 30"
        );
    }

    #[test]
    fn renders_file_end_preview() {
        let rendered = render_intent(&intent(
            "view_file_end",
            json!({"path": "app.log", "line_count": 20}),
        ))
        .unwrap();
        assert_eq!(rendered, "Get-Content -LiteralPath 'app.log' -Tail 20");
    }

    #[test]
    fn renders_first_and_last_file_preview() {
        let rendered = render_intent(&intent(
            "view_file_edges",
            json!({"path": ".env", "line_count": 20}),
        ))
        .unwrap();
        assert_eq!(
            rendered,
            "Write-Output '--- First 20 lines ---'; Get-Content -LiteralPath '.env' -TotalCount 20; Write-Output '--- Last 20 lines ---'; Get-Content -LiteralPath '.env' -Tail 20"
        );
    }

    #[test]
    fn rejects_an_excessive_file_preview() {
        let error = render_intent(&intent(
            "view_file_start",
            json!({"path": "large.log", "line_count": 501}),
        ))
        .unwrap_err();
        assert!(error.contains("between 1 and 500"));
    }

    #[test]
    fn renders_list_directories() {
        let rendered =
            render_intent(&intent("list_directories", json!({"root": "."}))).unwrap();
        assert_eq!(rendered, "Get-ChildItem -LiteralPath '.' -Directory");
    }

    #[test]
    fn renders_file_metadata() {
        let rendered =
            render_intent(&intent("inspect_file", json!({"path": "package.json"}))).unwrap();
        assert_eq!(
            rendered,
            "Get-Item -LiteralPath 'package.json' | Format-List FullName,Length,LastWriteTime,Attributes"
        );
    }

    #[test]
    fn renders_recursive_file_count() {
        let rendered = render_intent(&intent(
            "count_files",
            json!({"root": ".", "recursive": true}),
        ))
        .unwrap();
        assert_eq!(
            rendered,
            "Get-ChildItem -LiteralPath '.' -File -Recurse | Measure-Object | Select-Object -ExpandProperty Count"
        );
    }

    #[test]
    fn renders_process_list() {
        let rendered = render_intent(&intent("list_processes", json!({}))).unwrap();
        assert!(rendered.starts_with("Get-Process"));
        assert!(rendered.contains("Select-Object -First 25"));
    }

    #[test]
    fn renders_single_line_deterministic_catalog_command() {
        let rendered = render_intent(&intent(
            "catalog_command",
            json!({
                "catalog_id": "docker_list_docker_images",
                "command": "docker images",
                "risk": "DockerSafe"
            }),
        ))
        .unwrap();
        assert_eq!(rendered, "docker images");

        let multiline = render_intent(&intent(
            "catalog_command",
            json!({
                "catalog_id": "invalid_multiline",
                "command": "docker images\ndocker ps",
                "risk": "DockerSafe"
            }),
        ));
        assert!(multiline.is_err());
    }

    #[test]
    fn renders_listening_ports() {
        let rendered = render_intent(&intent("list_listening_ports", json!({}))).unwrap();
        assert!(rendered.starts_with("Get-NetTCPConnection -State Listen"));
    }

    #[test]
    fn renders_specific_port_inspection() {
        let rendered =
            render_intent(&intent("inspect_port", json!({"port": 3000}))).unwrap();
        assert!(rendered.starts_with("Get-NetTCPConnection -LocalPort 3000"));
    }

    #[test]
    fn rejects_invalid_port() {
        let error = render_intent(&intent("inspect_port", json!({"port": 70000}))).unwrap_err();
        assert!(error.contains("Invalid TCP port"));
    }

    #[test]
    fn rejects_unsupported_actions() {
        let error = render_intent(&intent("delete_everything", json!({}))).unwrap_err();
        assert!(error.contains("Unsupported intent action"));
    }

    #[test]
    fn rejects_missing_required_parameters() {
        let error = render_intent(&intent("change_directory", json!({}))).unwrap_err();
        assert!(error.contains("Missing or empty"));
    }

    #[test]
    fn renders_navigation_for_bash() {
        let rendered = render_intent_for_shell(
            &intent("change_directory", json!({"target": "terminal-mate"})),
            "bash",
        )
        .unwrap();
        assert_eq!(rendered, "cd -- 'terminal-mate'");
    }

    #[test]
    fn renders_recursive_file_search_for_bash() {
        let rendered = render_intent_for_shell(
            &intent(
                "find_files",
                json!({"root": ".", "pattern": "*.sh", "recursive": true}),
            ),
            "bash",
        )
        .unwrap();
        assert_eq!(rendered, "find '.' -type f -name '*.sh' -print");
    }

    #[test]
    fn renders_folder_or_file_inspection_for_bash() {
        let rendered = render_intent_for_shell(
            &intent("inspect_path", json!({"path": ".claude"})),
            "bash",
        )
        .unwrap();
        assert!(rendered.contains("Type: Folder"));
        assert!(rendered.contains("Type: File"));
        assert!(rendered.contains("ls -la -- '.claude'"));
    }

    #[test]
    fn renders_azure_resource_group_creation_with_login_prerequisite() {
        let rendered = render_intent_for_shell(
            &intent(
                "azure_resource_group_create",
                json!({"resource_group": "demo-rg", "location": "australiaeast"}),
            ),
            "powershell",
        )
        .unwrap();
        assert!(rendered.starts_with("az account show --output none"));
        assert!(rendered.contains(
            "az group create --name 'demo-rg' --location 'australiaeast'"
        ));
    }

    #[test]
    fn renders_aws_budget_json_without_exposing_shell_construction_to_ai() {
        let budget = intent(
            "aws_budget_create",
            json!({
                "account_id": "123456789012",
                "budget_name": "Waypoint-Monthly-Budget",
                "amount": "5"
            }),
        );

        let powershell = render_intent_for_shell(&budget, "powershell").unwrap();
        let bash = render_intent_for_shell(&budget, "bash").unwrap();

        for rendered in [powershell, bash] {
            assert!(rendered.starts_with("aws budgets create-budget"));
            assert!(rendered.contains("--account-id '123456789012'"));
            assert!(rendered.contains("\"BudgetName\":\"Waypoint-Monthly-Budget\""));
            assert!(rendered.contains("\"Amount\":\"5\""));
            assert!(rendered.contains("\"TimeUnit\":\"MONTHLY\""));
        }
    }

    #[test]
    fn renders_aws_budget_alert_with_notification_subscriber() {
        let alert = intent(
            "aws_budget_alert_create",
            json!({
                "account_id": "123456789012",
                "budget_name": "Waypoint-Monthly-Budget",
                "amount": "5",
                "threshold": "80",
                "notification_email": "owner@example.com"
            }),
        );

        let rendered = render_intent_for_shell(&alert, "powershell").unwrap();
        assert!(rendered.starts_with("aws budgets create-budget"));
        assert!(rendered.contains("--notifications-with-subscribers"));
        assert!(rendered.contains("\"NotificationType\":\"ACTUAL\""));
        assert!(rendered.contains("\"Threshold\":80.0"));
        assert!(rendered.contains("\"Address\":\"owner@example.com\""));
    }

    #[test]
    fn renders_provider_instance_creation_from_validated_parameters() {
        let aws = render_intent_for_shell(
            &intent(
                "aws_ec2_instance_create",
                json!({
                    "image_id": "ami-1234567890abcdef0",
                    "instance_type": "t3.micro",
                    "key_name": "deploy-key",
                    "security_group_ids": ["sg-12345678"],
                    "subnet_id": "subnet-12345678",
                    "region": "ap-southeast-2"
                }),
            ),
            "powershell",
        )
        .unwrap();
        assert!(aws.contains("aws ec2 run-instances"));
        assert!(aws.contains("--region 'ap-southeast-2'"));

        let gcp = render_intent_for_shell(
            &intent(
                "gcp_compute_instance_create",
                json!({
                    "instance_name": "waypoint-api",
                    "zone": "australia-southeast1-a",
                    "machine_type": "e2-micro"
                }),
            ),
            "bash",
        )
        .unwrap();
        assert_eq!(
            gcp,
            "gcloud compute instances create 'waypoint-api' --zone 'australia-southeast1-a' --machine-type 'e2-micro'"
        );
    }

    #[test]
    fn renders_non_interactive_ssh_with_optional_connection_parameters() {
        let rendered = render_intent_for_shell(
            &intent(
                "ssh_connect",
                json!({
                    "host": "example.com",
                    "user": "deploy",
                    "identity_file": "~/.ssh/deploy-key",
                    "port": 2222,
                    "remote_command": "uptime"
                }),
            ),
            "bash",
        )
        .unwrap();
        assert_eq!(
            rendered,
            "ssh -o BatchMode=yes -o StrictHostKeyChecking=accept-new -i '~/.ssh/deploy-key' -p 2222 'deploy@example.com' 'uptime'"
        );
    }

    #[test]
    fn renders_azure_vm_read_for_bash_with_login_prerequisite() {
        let rendered = render_intent_for_shell(
            &intent(
                "azure_vm_show",
                json!({"vm_name": "app-vm", "resource_group": "demo-rg"}),
            ),
            "bash",
        )
        .unwrap();
        assert!(rendered.contains("Azure login required"));
        assert!(rendered.contains(
            "az vm show --name 'app-vm' --resource-group 'demo-rg'"
        ));
    }

    #[test]
    fn renders_azure_vm_list_with_show_details_and_a_login_prerequisite() {
        // --show-details is deliberate: without it, "list azure vms" would
        // omit power state, which is exactly the detail that made this
        // worth asking for in the first place (matches the same flag now
        // used for the auto status check after start/deallocate).
        let rendered = render_intent_for_shell(
            &intent("azure_vm_list", json!({})),
            "powershell",
        )
        .unwrap();
        assert!(rendered.starts_with("az account show --output none"));
        assert!(rendered.contains("az vm list --show-details --output table"));
    }

    #[test]
    fn renders_azure_account_clear_without_a_login_prerequisite() {
        // Deliberately has no login-preflight wrapper: this command exists to
        // fix a broken/stale login, so requiring a working login first would
        // be self-defeating.
        let rendered =
            render_intent_for_shell(&intent("azure_account_clear", json!({})), "powershell")
                .unwrap();
        assert_eq!(rendered, "az account clear");
    }

    #[test]
    fn renders_azure_provider_show_with_login_prerequisite() {
        let rendered = render_intent_for_shell(
            &intent("azure_provider_show", json!({"namespace": "Microsoft.Storage"})),
            "powershell",
        )
        .unwrap();
        assert!(rendered.starts_with("az account show --output none"));
        assert!(rendered.contains(
            "az provider show --namespace 'Microsoft.Storage' --query registrationState"
        ));
    }

    #[test]
    fn renders_azure_provider_register_with_login_prerequisite_for_bash() {
        let rendered = render_intent_for_shell(
            &intent("azure_provider_register", json!({"namespace": "Microsoft.Storage"})),
            "bash",
        )
        .unwrap();
        assert!(rendered.contains("Azure login required"));
        assert!(rendered.contains("az provider register --namespace 'Microsoft.Storage'"));
    }

    #[test]
    fn renders_azure_and_terraform_golden_matrix_for_each_shell() {
        struct Case {
            action: &'static str,
            parameters: serde_json::Value,
            powershell: &'static str,
            bash: &'static str,
        }

        let powershell_azure_preflight = "az account show --output none; if ($LASTEXITCODE -ne 0) { Write-Error 'Azure login required. Run: Sign in to Azure'; exit $LASTEXITCODE }; ";
        let bash_azure_preflight = "az account show --output none >/dev/null 2>&1 || { printf '%s\\n' 'Azure login required. Run: Sign in to Azure' >&2; exit 1; }; ";
        let cases = [
            Case {
                action: "azure_cli_version",
                parameters: json!({}),
                powershell: "az --version",
                bash: "az --version",
            },
            Case {
                action: "azure_login_device_code",
                parameters: json!({}),
                powershell: "az login --use-device-code",
                bash: "az login --use-device-code",
            },
            Case {
                action: "azure_subscription_set",
                parameters: json!({"subscription": "Owner's Subscription"}),
                powershell: "az account set --subscription 'Owner''s Subscription'",
                bash: "az account set --subscription 'Owner'\"'\"'s Subscription'",
            },
            Case {
                action: "azure_resource_group_create",
                parameters: json!({"resource_group": "demo rg", "location": "australiaeast"}),
                powershell: "az group create --name 'demo rg' --location 'australiaeast'",
                bash: "az group create --name 'demo rg' --location 'australiaeast'",
            },
            Case {
                action: "azure_storage_account_create",
                parameters: json!({"account_name": "demostore", "resource_group": "demo rg"}),
                powershell: "az storage account create --name 'demostore' --resource-group 'demo rg' --sku Standard_LRS --encryption-services blob",
                bash: "az storage account create --name 'demostore' --resource-group 'demo rg' --sku Standard_LRS --encryption-services blob",
            },
            Case {
                action: "azure_vm_show",
                parameters: json!({"vm_name": "app vm", "resource_group": "demo rg"}),
                powershell: "az vm show --name 'app vm' --resource-group 'demo rg'",
                bash: "az vm show --name 'app vm' --resource-group 'demo rg'",
            },
            Case {
                action: "azure_vm_start",
                parameters: json!({"vm_name": "app vm", "resource_group": "demo rg"}),
                powershell: "az vm start --name 'app vm' --resource-group 'demo rg'",
                bash: "az vm start --name 'app vm' --resource-group 'demo rg'",
            },
            Case {
                action: "azure_vm_deallocate",
                parameters: json!({"vm_name": "app vm", "resource_group": "demo rg"}),
                powershell: "az vm deallocate --name 'app vm' --resource-group 'demo rg'",
                bash: "az vm deallocate --name 'app vm' --resource-group 'demo rg'",
            },
            Case {
                action: "terraform_version",
                parameters: json!({}),
                powershell: "terraform --version",
                bash: "terraform --version",
            },
            Case {
                action: "terraform_init",
                parameters: json!({}),
                powershell: "terraform init",
                bash: "terraform init",
            },
            Case {
                action: "terraform_validate",
                parameters: json!({}),
                powershell: "terraform validate",
                bash: "terraform validate",
            },
            Case {
                action: "terraform_plan",
                parameters: json!({}),
                powershell: "terraform plan",
                bash: "terraform plan",
            },
            Case {
                action: "terraform_state_show",
                parameters: json!({"address": "azurerm_resource_group.demo"}),
                powershell: "terraform state show 'azurerm_resource_group.demo'",
                bash: "terraform state show 'azurerm_resource_group.demo'",
            },
            Case {
                action: "terraform_import",
                parameters: json!({
                    "address": "azurerm_resource_group.demo",
                    "resource_id": "/subscriptions/example/resourceGroups/demo rg"
                }),
                powershell: "terraform import 'azurerm_resource_group.demo' '/subscriptions/example/resourceGroups/demo rg'",
                bash: "terraform import 'azurerm_resource_group.demo' '/subscriptions/example/resourceGroups/demo rg'",
            },
            Case {
                action: "terraform_apply",
                parameters: json!({}),
                powershell: "terraform apply -auto-approve",
                bash: "terraform apply -auto-approve",
            },
            Case {
                action: "terraform_destroy",
                parameters: json!({}),
                powershell: "terraform destroy -auto-approve",
                bash: "terraform destroy -auto-approve",
            },
        ];

        for case in cases {
            let command_intent = intent(case.action, case.parameters);
            let powershell = render_intent_for_shell(&command_intent, "powershell").unwrap();
            let bash = render_intent_for_shell(&command_intent, "bash").unwrap();

            let expected_powershell = if matches!(
                case.action,
                "azure_resource_group_create"
                    | "azure_storage_account_create"
                    | "azure_vm_show"
                    | "azure_vm_start"
                    | "azure_vm_deallocate"
            ) {
                format!("{powershell_azure_preflight}{}", case.powershell)
            } else {
                case.powershell.to_owned()
            };
            let expected_bash = if matches!(
                case.action,
                "azure_resource_group_create"
                    | "azure_storage_account_create"
                    | "azure_vm_show"
                    | "azure_vm_start"
                    | "azure_vm_deallocate"
            ) {
                format!("{bash_azure_preflight}{}", case.bash)
            } else {
                case.bash.to_owned()
            };

            assert_eq!(powershell, expected_powershell, "PowerShell: {}", case.action);
            assert_eq!(bash, expected_bash, "Bash: {}", case.action);
        }
    }

    #[test]
    fn renders_terraform_and_explain_only_commands_exactly() {
        assert_eq!(
            render_intent(&intent("terraform_plan", json!({}))).unwrap(),
            "terraform plan"
        );
        assert_eq!(
            render_intent(&intent("terraform_apply", json!({}))).unwrap(),
            "terraform apply -auto-approve"
        );
        assert_eq!(
            render_intent(&intent("terraform_destroy", json!({}))).unwrap(),
            "terraform destroy -auto-approve"
        );
        assert_eq!(
            render_intent(&intent(
                "azure_resource_group_delete",
                json!({"resource_group": "demo-rg"}),
            ))
            .unwrap(),
            "az group delete --name 'demo-rg' --yes --no-wait"
        );
    }

    #[test]
    fn renders_ssh_key_paths_for_each_shell() {
        let ssh_intent = intent(
            "generate_ssh_key",
            json!({"key_name": "deploy-key", "comment": "TerminalMate"}),
        );
        assert!(render_intent_for_shell(&ssh_intent, "powershell")
            .unwrap()
            .contains("Join-Path $HOME 'deploy-key'"));
        assert!(render_intent_for_shell(&ssh_intent, "bash")
            .unwrap()
            .contains("\"$HOME/deploy-key\""));
    }

    #[test]
    fn renders_show_ssh_public_key_for_each_shell() {
        let show_key_intent = intent("show_ssh_public_key", json!({"key_name": "deploy-key"}));

        assert_eq!(
            render_intent_for_shell(&show_key_intent, "bash").unwrap(),
            "cat -- \"$HOME/deploy-key.pub\""
        );
        assert_eq!(
            render_intent_for_shell(&show_key_intent, "powershell").unwrap(),
            "Get-Content -LiteralPath (Join-Path $HOME 'deploy-key.pub')"
        );

        // Path separators in the key name are sanitized the same way
        // generate_ssh_key already does, so the two always stay in sync.
        let nested_key_intent =
            intent("show_ssh_public_key", json!({"key_name": "keys/deploy-key"}));
        assert_eq!(
            render_intent_for_shell(&nested_key_intent, "bash").unwrap(),
            "cat -- \"$HOME/keys_deploy-key.pub\""
        );
    }

    #[test]
    fn renders_ssh_vm_with_flags_that_avoid_the_host_key_and_password_prompts() {
        let ssh_vm_intent = intent(
            "ssh_vm",
            json!({"admin": "azureuser", "ip": "4.196.163.10", "key_name": "glaucoma-ai-azure-key"}),
        );

        let bash_rendered = render_intent_for_shell(&ssh_vm_intent, "bash").unwrap();
        assert!(bash_rendered.contains("-o BatchMode=yes"));
        assert!(bash_rendered.contains("-o StrictHostKeyChecking=accept-new"));
        assert!(bash_rendered.contains("\"$HOME/glaucoma-ai-azure-key\""));
        assert!(bash_rendered.contains("'azureuser@4.196.163.10'"));

        let powershell_rendered = render_intent_for_shell(&ssh_vm_intent, "powershell").unwrap();
        assert!(powershell_rendered.contains("-o BatchMode=yes"));
        assert!(powershell_rendered.contains("-o StrictHostKeyChecking=accept-new"));
        assert!(powershell_rendered.contains("Join-Path $HOME 'glaucoma-ai-azure-key'"));
    }

    #[test]
    fn renders_single_python_script_for_each_runtime() {
        let run = intent("run_script", json!({"path": "scripts/seed data.py"}));

        let powershell = render_intent_for_shell(&run, "powershell").unwrap();
        assert!(powershell.contains("Get-Item -LiteralPath 'scripts/seed data.py'"));
        assert!(powershell.contains("& python -u $script.FullName"));
        assert!(powershell.contains("Script failed"));

        let bash = render_intent_for_shell(&run, "bash").unwrap();
        assert!(bash.contains("realpath -- 'scripts/seed data.py'"));
        assert!(bash.contains("*.py) python3 -u"));
        assert!(bash.contains("tm_run_script \"$script\""));
    }

    #[test]
    fn treats_a_single_leading_separator_as_workspace_relative_in_powershell() {
        let backslash = intent(
            "run_script",
            json!({"path": "\\scripts\\evaluate_answers.py"}),
        );
        let slash = intent(
            "run_script",
            json!({"path": "/scripts/evaluate_answers.py"}),
        );

        let backslash_rendered = render_intent_for_shell(&backslash, "powershell").unwrap();
        let slash_rendered = render_intent_for_shell(&slash, "powershell").unwrap();

        assert!(backslash_rendered
            .contains("Get-Item -LiteralPath 'scripts\\evaluate_answers.py'"));
        assert!(slash_rendered.contains("Get-Item -LiteralPath 'scripts/evaluate_answers.py'"));
    }

    #[test]
    fn preserves_absolute_script_paths_for_their_runtime() {
        let drive_path = intent(
            "run_script",
            json!({"path": "D:\\tools\\evaluate_answers.py"}),
        );
        let unc_path = intent(
            "run_script",
            json!({"path": "\\\\server\\share\\evaluate_answers.py"}),
        );
        let linux_path = intent(
            "run_script",
            json!({"path": "/opt/tools/evaluate_answers.py"}),
        );

        let drive_rendered = render_intent_for_shell(&drive_path, "powershell").unwrap();
        let unc_rendered = render_intent_for_shell(&unc_path, "powershell").unwrap();
        let linux_rendered = render_intent_for_shell(&linux_path, "bash").unwrap();

        assert!(drive_rendered
            .contains("Get-Item -LiteralPath 'D:\\tools\\evaluate_answers.py'"));
        assert!(unc_rendered
            .contains("Get-Item -LiteralPath '\\\\server\\share\\evaluate_answers.py'"));
        assert!(linux_rendered.contains("realpath -- '/opt/tools/evaluate_answers.py'"));
    }

    #[test]
    fn renders_single_script_arguments_as_shell_literals() {
        let run = intent(
            "run_script",
            json!({
                "path": "collect_manual.py",
                "interpreter": "python",
                "arguments": ["--prefixes", "R", "--out", "../generic-residence", "O'Brien"]
            }),
        );

        let powershell = render_intent_for_shell(&run, "powershell").unwrap();
        assert!(powershell.contains(
            "& python -u $script.FullName '--prefixes' 'R' '--out' '../generic-residence' 'O''Brien'"
        ));

        let bash = render_intent_for_shell(&run, "bash").unwrap();
        assert!(bash.contains(
            "tm_run_script \"$script\" '--prefixes' 'R' '--out' '../generic-residence' 'O'\"'\"'Brien'"
        ));
        assert!(bash.contains("python3 -u \"$script\" \"$@\""));
    }

    #[test]
    fn rejects_non_string_script_arguments() {
        let run = intent(
            "run_script",
            json!({"path": "collect_manual.py", "arguments": ["--limit", 5]}),
        );
        assert!(render_intent_for_shell(&run, "powershell")
            .unwrap_err()
            .contains("expected a list of strings"));
    }

    #[test]
    fn renders_script_batches_in_requested_order() {
        let newest = intent(
            "run_scripts",
            json!({"root": ".", "order": "modified_desc", "recursive": false}),
        );
        let powershell = render_intent_for_shell(&newest, "powershell").unwrap();
        assert!(powershell.contains("Descending=$true"));
        assert!(powershell.contains("foreach ($script in $scripts)"));
        assert!(!powershell.contains(" -Recurse"));

        let alphabetical = intent(
            "run_scripts",
            json!({"root": ".", "order": "name", "recursive": true, "extension": ".py"}),
        );
        let bash = render_intent_for_shell(&alphabetical, "bash").unwrap();
        assert!(bash.contains("-iname '*.py'"));
        assert!(bash.contains("-print0 | sort -z"));
        assert!(!bash.contains("-maxdepth 1"));
        assert!(bash.contains("exit $?"));
    }

    #[test]
    fn renders_script_order_preview_without_execution() {
        let preview = intent(
            "list_scripts",
            json!({"root": ".", "order": "modified_asc", "recursive": false}),
        );

        let powershell = render_intent_for_shell(&preview, "powershell").unwrap();
        assert!(powershell.contains("Descending=$false"));
        assert!(powershell.contains("Select-Object FullName,Extension,LastWriteTime"));
        assert!(!powershell.contains(">>> Running"));
    }

    #[test]
    fn rejects_runtime_incompatible_script_filter() {
        let shell_scripts = intent(
            "run_scripts",
            json!({"root": ".", "order": "name", "extension": ".sh"}),
        );
        let error = render_intent_for_shell(&shell_scripts, "powershell").unwrap_err();
        assert!(error.contains("not supported in PowerShell"));
    }

    #[test]
    fn rejects_a_single_script_that_is_incompatible_with_the_runtime() {
        let shell_script = intent("run_script", json!({"path": "scripts/deploy.sh"}));
        assert!(render_intent_for_shell(&shell_script, "powershell")
            .unwrap_err()
            .contains("not supported in PowerShell"));

        let powershell_script = intent("run_script", json!({"path": "scripts/deploy.ps1"}));
        assert!(render_intent_for_shell(&powershell_script, "bash")
            .unwrap_err()
            .contains("not supported in WSL Bash"));
    }
}
