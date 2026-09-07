use regex::Regex;
use std::path::{Path, PathBuf};
use tauri::State;

use crate::{
    app_state::AppState,
    commands::intent_adapter::render_intent_for_shell,
    models::{
        intent::{
            IntentRequest, IntentResponse, ResolvedCommand, ResolvedCommandExecutionMode,
            ResolvedCommandSource, SidecarAiPlannerConfig, SidecarExecutionProfile,
        },
        workspace::ExecutionProfile,
    },
    sidecar_proxy::SidecarProxy,
};

#[tauri::command]
pub async fn resolve_command(
    message: String,
    profile: ExecutionProfile,
    state: State<'_, AppState>,
    proxy: State<'_, SidecarProxy>,
) -> Result<ResolvedCommand, String> {
    let trimmed = message.trim();
    if trimmed.is_empty() {
        return Err("Enter a command.".to_owned());
    }

    if let Some((command, action, response_source)) =
        try_interpret_with_action(trimmed, &profile, &state, &proxy).await?
    {
        let (execution_mode, explanation) = execution_guidance(&action);
        let (source, label) = if response_source == "ai" {
            (ResolvedCommandSource::AiIntent, "AI PLAN")
        } else {
            (ResolvedCommandSource::Intent, "LOCAL MATCH")
        };
        return Ok(ResolvedCommand {
            command,
            source,
            note: Some(format!("{label}: interpreted \"{trimmed}\" as {action}")),
            execution_mode,
            explanation,
        });
    }

    let (prepared, adjusted) = prepare_direct_command(trimmed, &profile);

    Ok(ResolvedCommand {
        command: normalize_direct_command(&prepared, &profile.shell)?,
        source: ResolvedCommandSource::Direct,
        note: adjusted.then(|| {
            "Adjusted workspace-relative paths and PowerShell pipeline syntax before review."
                .to_owned()
        }),
        execution_mode: ResolvedCommandExecutionMode::Execute,
        explanation: None,
    })
}

fn prepare_direct_command(command: &str, profile: &ExecutionProfile) -> (String, bool) {
    if !is_powershell(&profile.shell) {
        return (command.to_owned(), false);
    }

    let repaired_pipeline = repair_powershell_pipeline_variable(command);
    let repaired_paths = repair_duplicated_workspace_prefix(
        &repaired_pipeline,
        Path::new(&profile.working_directory),
    );
    let adjusted = repaired_paths != command;

    (repaired_paths, adjusted)
}

fn is_powershell(shell: &str) -> bool {
    shell.eq_ignore_ascii_case("powershell") || shell.eq_ignore_ascii_case("pwsh")
}

fn repair_powershell_pipeline_variable(command: &str) -> String {
    let where_object_block = Regex::new(r"(?is)(where-object\s*\{[^{}]*)\$\*\.")
        .expect("PowerShell pipeline-variable regex must compile");
    let mut repaired = command.to_owned();

    loop {
        let next = where_object_block
            .replace_all(&repaired, |captures: &regex::Captures<'_>| {
                format!("{}$_.", &captures[1])
            })
            .into_owned();
        if next == repaired {
            return repaired;
        }
        repaired = next;
    }
}

fn repair_duplicated_workspace_prefix(command: &str, working_directory: &Path) -> String {
    let Some(workspace_name) = working_directory
        .file_name()
        .and_then(|name| name.to_str())
    else {
        return command.to_owned();
    };

    let Some(command_start) = find_case_insensitive(command, "get-childitem") else {
        return command.to_owned();
    };
    let arguments_start = command_start + "get-childitem".len();
    let rest = &command[arguments_start..];
    let trimmed_rest = rest.trim_start();
    let leading_whitespace = rest.len() - trimmed_rest.len();
    let paths_start = arguments_start + leading_whitespace;
    let paths_end = trimmed_rest
        .find(|character: char| character.is_whitespace())
        .unwrap_or(trimmed_rest.len());
    let path_list = &trimmed_rest[..paths_end];

    if path_list.is_empty() || path_list.starts_with('-') {
        return command.to_owned();
    }

    let candidates = path_list.split(',').map(str::trim).collect::<Vec<_>>();
    let corrected = candidates
        .iter()
        .map(|candidate| strip_workspace_prefix(candidate, workspace_name))
        .collect::<Option<Vec<_>>>();
    let Some(corrected) = corrected else {
        return command.to_owned();
    };

    let originals_exist = candidates
        .iter()
        .any(|candidate| workspace_child(working_directory, candidate).exists());
    let corrected_exist = corrected
        .iter()
        .all(|candidate| workspace_child(working_directory, candidate).exists());
    if originals_exist || !corrected_exist {
        return command.to_owned();
    }

    let mut repaired = String::with_capacity(command.len());
    repaired.push_str(&command[..paths_start]);
    repaired.push_str(&corrected.join(","));
    repaired.push_str(&trimmed_rest[paths_end..]);
    repaired
}

fn find_case_insensitive(value: &str, needle: &str) -> Option<usize> {
    value.to_ascii_lowercase().find(needle)
}

fn strip_workspace_prefix(candidate: &str, workspace_name: &str) -> Option<String> {
    let (quote, value) = if candidate.starts_with('\'') && candidate.ends_with('\'') {
        (Some('\''), &candidate[1..candidate.len() - 1])
    } else if candidate.starts_with('"') && candidate.ends_with('"') {
        (Some('"'), &candidate[1..candidate.len() - 1])
    } else {
        (None, candidate)
    };
    let value = value.strip_prefix(".\\").unwrap_or(value);
    let prefix_length = workspace_name.len();
    if value.len() <= prefix_length
        || !value[..prefix_length].eq_ignore_ascii_case(workspace_name)
        || !matches!(value.as_bytes()[prefix_length], b'\\' | b'/')
    {
        return None;
    }

    let stripped = &value[prefix_length + 1..];
    if stripped.is_empty() {
        return None;
    }

    Some(match quote {
        Some(character) => format!("{character}{stripped}{character}"),
        None => stripped.to_owned(),
    })
}

fn workspace_child(working_directory: &Path, candidate: &str) -> PathBuf {
    let value = candidate.trim_matches(['\'', '"']);
    let native = value.replace(['\\', '/'], std::path::MAIN_SEPARATOR_STR);
    working_directory.join(native)
}

fn update_powershell_group_stack(line: &str, stack: &mut Vec<char>) -> Result<(), String> {
    let mut chars = line.chars().peekable();
    let mut quote = None;

    while let Some(character) = chars.next() {
        if let Some(active_quote) = quote {
            if character == '`' && active_quote == '"' {
                let _ = chars.next();
            } else if character == active_quote {
                if active_quote == '\'' && chars.peek() == Some(&'\'') {
                    let _ = chars.next();
                } else {
                    quote = None;
                }
            }
            continue;
        }

        match character {
            '\'' | '"' => quote = Some(character),
            '(' | '[' | '{' => stack.push(character),
            ')' | ']' | '}' => {
                let expected = match character {
                    ')' => '(',
                    ']' => '[',
                    '}' => '{',
                    _ => unreachable!(),
                };
                if stack.pop() != Some(expected) {
                    return Err(
                        "The pasted PowerShell command has mismatched parentheses, brackets, or braces."
                            .to_owned(),
                    );
                }
            }
            _ => {}
        }
    }

    if quote.is_some() {
        return Err("Complete the quoted PowerShell value before running the command.".to_owned());
    }
    Ok(())
}

fn is_powershell_grouped_iteration_script(command: &str) -> bool {
    let mut group_stack = Vec::new();
    let mut found_iteration = false;
    let mut opened_iteration_group = false;
    let mut completed_iteration = false;
    let mut awaiting_pipeline_command = false;

    for line in command.lines().map(str::trim).filter(|line| !line.is_empty()) {
        let depth_before = group_stack.len();
        if completed_iteration {
            return false;
        }

        if !found_iteration && depth_before == 0 {
            let lowercase = line.to_ascii_lowercase();
            let is_foreach = lowercase
                .strip_prefix("foreach")
                .is_some_and(|rest| rest.trim_start().starts_with('('));
            let is_foreach_object = lowercase
                .split('|')
                .any(|segment| {
                    segment
                        .trim_start()
                        .strip_prefix("foreach-object")
                        .is_some_and(|rest| rest.contains('{'))
                });
            let is_variable_assignment = line.starts_with('$') && line.contains('=');
            if is_foreach || is_foreach_object {
                found_iteration = true;
                awaiting_pipeline_command = false;
            } else if !is_variable_assignment {
                if ends_with_powershell_pipeline(line) {
                    awaiting_pipeline_command = true;
                } else if awaiting_pipeline_command
                    && lowercase
                        .strip_prefix("foreach-object")
                        .is_some_and(|rest| rest.contains('{'))
                {
                    found_iteration = true;
                    awaiting_pipeline_command = false;
                } else {
                    return false;
                }
            }
        }

        if update_powershell_group_stack(line, &mut group_stack).is_err() {
            return false;
        }
        if found_iteration && !group_stack.is_empty() {
            opened_iteration_group = true;
        }
        if found_iteration && opened_iteration_group && group_stack.is_empty() {
            completed_iteration = true;
        }
    }

    found_iteration && completed_iteration
}

fn is_powershell_scoped_environment_script(command: &str) -> bool {
    let lines = command
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>();

    if lines.len() < 2 || !is_powershell_environment_assignment(lines[0]) {
        return false;
    }

    lines[1..]
        .iter()
        .any(|line| !is_powershell_environment_assignment(line))
}

fn is_powershell_variable_backed_script(command: &str) -> bool {
    let lines = command
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>();

    // Loop scripts use the stricter block-aware parser, which rejects any
    // unrelated command appended after the completed loop.
    if lines.iter().any(|line| {
        let lowercase = line.to_ascii_lowercase();
        lowercase.starts_with("foreach") || lowercase.contains("| foreach-object {")
    }) {
        return false;
    }

    let mut assignments = Vec::new();

    for (index, line) in lines.iter().enumerate() {
        let Some(rest) = line.strip_prefix('$') else {
            continue;
        };
        let Some((name, _value)) = rest.split_once('=') else {
            continue;
        };
        let name = name.trim();
        if !name.is_empty()
            && name
                .chars()
                .all(|character| character == '_' || character.is_ascii_alphanumeric())
        {
            assignments.push((name.to_ascii_lowercase(), index));
        }
    }

    assignments.into_iter().any(|(name, assignment_index)| {
        let reference = format!("${name}");
        lines[assignment_index + 1..].iter().any(|line| {
            let lowercase = line.to_ascii_lowercase();
            lowercase.match_indices(&reference).any(|(index, _)| {
                lowercase[index + reference.len()..]
                    .chars()
                    .next()
                    .is_none_or(|character| {
                        character != '_' && !character.is_ascii_alphanumeric()
                    })
            })
        })
    })
}

fn is_powershell_environment_assignment(line: &str) -> bool {
    let lowercase = line.to_ascii_lowercase();
    let Some(rest) = lowercase.strip_prefix("$env:") else {
        return false;
    };
    let Some((name, _value)) = rest.split_once('=') else {
        return false;
    };

    let name = name.trim();
    !name.is_empty()
        && name
            .chars()
            .all(|character| character == '_' || character.is_ascii_alphanumeric())
}

fn normalize_direct_command(command: &str, shell: &str) -> Result<String, String> {
    let powershell = is_powershell(shell);
    let grouped_command;
    let command = if powershell
        && (is_powershell_grouped_iteration_script(command)
            || is_powershell_scoped_environment_script(command)
            || is_powershell_variable_backed_script(command))
    {
        grouped_command = format!("& {{\n{}\n}}", command.trim());
        grouped_command.as_str()
    } else {
        command.trim()
    };
    let mut lines = command.lines().peekable();
    let mut normalized = String::new();
    let mut group_stack = Vec::new();

    while let Some(line) = lines.next() {
        let line = line.trim_end_matches('\r').trim();
        let marker_continued = line.ends_with('\\') || line.ends_with('`');
        let pipeline_continued = powershell && ends_with_powershell_pipeline(line);
        let continued = marker_continued || pipeline_continued;
        let content = if marker_continued {
            line[..line.len() - 1].trim_end()
        } else {
            line
        };

        if powershell {
            update_powershell_group_stack(content, &mut group_stack)?;
        }

        normalized.push_str(content);
        if lines.peek().is_some() {
            if !continued && (!powershell || group_stack.is_empty()) {
                return Err(
                    "Enter one command at a time. Bash \\, PowerShell `, and grouped PowerShell multiline expressions are supported."
                        .to_owned(),
                );
            }
            normalized.push(if continued || !powershell { ' ' } else { '\n' });
        }
    }

    if powershell && !group_stack.is_empty() {
        return Err(
            "Complete the PowerShell parentheses, brackets, or braces before running the command."
                .to_owned(),
        );
    }

    Ok(normalized)
}

fn ends_with_powershell_pipeline(line: &str) -> bool {
    let trimmed = line.trim_end();
    if !trimmed.ends_with('|') || trimmed.ends_with("||") {
        return false;
    }

    trimmed
        .strip_suffix('|')
        .expect("pipeline suffix was checked")
        .chars()
        .rev()
        .take_while(|character| *character == '`')
        .count()
        % 2
        == 0
}

// terraform_apply/terraform_destroy deliberately are not in this list.
// Terraform's interactive confirmation prompt (the reason they were Explain
// Only in the first place) is now avoided by rendering with -auto-approve
// (see intent_adapter.rs), so they execute through the normal High Risk /
// typed-RUN path like any other destructive command, instead of requiring a
// separate manual run outside TerminalMate.
fn execution_guidance(
    action: &str,
) -> (ResolvedCommandExecutionMode, Option<String>) {
    let explanation = match action {
        "azure_storage_keys_list" => Some(
            "This command can reveal storage-account credentials. TerminalMate shows it for review but will not retrieve or display secrets. Copy it only when you understand where its output will be stored."
                .to_owned(),
        ),
        "azure_resource_group_delete" => Some(
            "Deleting an Azure resource group removes every resource inside it. TerminalMate will not execute this command. Verify the subscription and resource group before running it manually."
                .to_owned(),
        ),
        _ => None,
    };

    if explanation.is_some() {
        (ResolvedCommandExecutionMode::ExplainOnly, explanation)
    } else {
        (ResolvedCommandExecutionMode::Execute, None)
    }
}

/// `None` covers every reason to fall back to treating `message` as a
/// literal command: the sidecar isn't ready, it couldn't be reached, it
/// didn't recognize the request, or it matched an action this build's
/// adapter doesn't render. Plain-English matching is an enhancement layered
/// on top of the always-available direct command path, never a requirement
/// for it.
#[cfg(test)]
async fn try_interpret(
    message: &str,
    profile: &ExecutionProfile,
    state: &AppState,
    proxy: &SidecarProxy,
) -> Result<Option<String>, String> {
    try_interpret_with_action(message, profile, state, proxy)
        .await
        .map(|resolved| resolved.map(|(command, _, _)| command))
}

async fn try_interpret_with_action(
    message: &str,
    profile: &ExecutionProfile,
    state: &AppState,
    proxy: &SidecarProxy,
) -> Result<Option<(String, String, String)>, String> {
    let ai_settings = state.ai_planner.lock().await.clone();
    let request = IntentRequest {
        message: message.to_owned(),
        execution_profile: SidecarExecutionProfile::from(profile),
        ai_config: Some(SidecarAiPlannerConfig {
            enabled: ai_settings.enabled,
            endpoint: ai_settings.endpoint,
            model: ai_settings.model,
            api_key: ai_settings.api_key,
        }),
    };

    let response: Option<IntentResponse> = proxy
        .post_if_ready(state, "/v1/intents/interpret", &request)
        .await;
    let Some(response) = response else {
        return Ok(None);
    };

    if response.requires_clarification {
        return Err(response
            .message
            .unwrap_or_else(|| "More information is required for this request.".to_owned()));
    }

    if !response.matched {
        return Ok(None);
    }

    let intent = response
        .intent
        .ok_or_else(|| "The matched request did not include a command intent.".to_owned())?;
    let action = intent.action.clone();
    let source = response.source;
    render_intent_for_shell(&intent, &profile.shell)
        .map(|command| Some((command, action, source)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::BufRead;
    use std::path::PathBuf;
    use std::process::{Child, Command, Stdio};

    fn windows_test_profile() -> ExecutionProfile {
        ExecutionProfile {
            host_os: "windows".to_owned(),
            runtime: "local".to_owned(),
            runtime_name: "Windows".to_owned(),
            target_os: "windows".to_owned(),
            shell: "powershell".to_owned(),
            working_directory: r"C:\Windows".to_owned(),
            architecture: "x86_64".to_owned(),
            privilege: "standard-user".to_owned(),
        }
    }

    #[test]
    fn repairs_duplicated_workspace_paths_and_where_object_pipeline_variables() {
        let root = std::env::temp_dir().join(format!(
            "terminal-mate-workspace-path-repair-{}",
            uuid::Uuid::new_v4()
        ));
        let backend = root.join("backend");
        for child in ["scripts", "tests", "_experiments"] {
            fs::create_dir_all(backend.join(child)).unwrap();
        }
        let mut profile = windows_test_profile();
        profile.working_directory = backend.to_string_lossy().into_owned();
        let command = r#"Get-ChildItem backend\scripts,backend\tests,backend\_experiments -File | Where-Object {
$*.Name -like "*source_boundary_classifier*" -or
$*.Name -like "*source_boundary_contract*"
} | Select-Object FullName | Sort-Object FullName"#;

        let (prepared, adjusted) = prepare_direct_command(command, &profile);

        assert!(adjusted);
        assert!(prepared.starts_with(
            "Get-ChildItem scripts,tests,_experiments -File | Where-Object {"
        ));
        assert!(prepared.contains("$_.Name -like \"*source_boundary_classifier*\""));
        assert!(!prepared.contains("backend\\backend"));
        assert!(!prepared.contains("$*.Name"));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn preserves_an_existing_nested_workspace_path() {
        let root = std::env::temp_dir().join(format!(
            "terminal-mate-workspace-path-preserve-{}",
            uuid::Uuid::new_v4()
        ));
        let backend = root.join("backend");
        fs::create_dir_all(backend.join("scripts")).unwrap();
        fs::create_dir_all(backend.join("backend").join("scripts")).unwrap();
        let mut profile = windows_test_profile();
        profile.working_directory = backend.to_string_lossy().into_owned();
        let command = "Get-ChildItem backend\\scripts -File";

        let (prepared, adjusted) = prepare_direct_command(command, &profile);

        assert!(!adjusted);
        assert_eq!(prepared, command);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn terraform_apply_and_destroy_execute_normally_instead_of_explain_only() {
        assert_eq!(
            execution_guidance("terraform_apply"),
            (ResolvedCommandExecutionMode::Execute, None)
        );
        assert_eq!(
            execution_guidance("terraform_destroy"),
            (ResolvedCommandExecutionMode::Execute, None)
        );
    }

    #[test]
    fn credential_and_infrastructure_deletion_intents_remain_explain_only() {
        let (mode, explanation) = execution_guidance("azure_storage_keys_list");
        assert_eq!(mode, ResolvedCommandExecutionMode::ExplainOnly);
        assert!(explanation.is_some());

        let (mode, explanation) = execution_guidance("azure_resource_group_delete");
        assert_eq!(mode, ResolvedCommandExecutionMode::ExplainOnly);
        assert!(explanation.is_some());
    }

    #[test]
    fn normalizes_bash_and_powershell_continued_direct_commands() {
        assert_eq!(
            normalize_direct_command(
                "az group create \\\n  --name glaucoma-ai-tfstate-rg \\\n  --location australiaeast",
                "bash",
            )
            .unwrap(),
            "az group create --name glaucoma-ai-tfstate-rg --location australiaeast"
        );
        assert_eq!(
            normalize_direct_command(
                "az group create `\r\n  --name glaucoma-ai-tfstate-rg `\r\n  --location australiaeast",
                "powershell",
            )
            .unwrap(),
            "az group create --name glaucoma-ai-tfstate-rg --location australiaeast"
        );
        assert_eq!(
            normalize_direct_command(
                "Get-ChildItem backend\\app -Recurse -Filter *.py |\n  Select-String -Pattern \"OpenAI|AsyncOpenAI\" |\n  Select-Object Path,LineNumber,Line",
                "powershell",
            )
            .unwrap(),
            "Get-ChildItem backend\\app -Recurse -Filter *.py | Select-String -Pattern \"OpenAI|AsyncOpenAI\" | Select-Object Path,LineNumber,Line"
        );
    }

    #[test]
    fn normalizes_grouped_powershell_multiline_expressions() {
        let command = r#"[Environment]::SetEnvironmentVariable(
  "Path",
  "C:\Users\Allan\.local\bin;" + [Environment]::GetEnvironmentVariable("Path", "User"),
  "User"
)"#;

        assert_eq!(
            normalize_direct_command(command, "powershell").unwrap(),
            r#"[Environment]::SetEnvironmentVariable(
"Path",
"C:\Users\Allan\.local\bin;" + [Environment]::GetEnvironmentVariable("Path", "User"),
"User"
)"#
        );
    }

    #[test]
    fn normalizes_a_variable_prelude_and_foreach_as_one_powershell_script() {
        let command = r#"$temp = (Get-Content .tmp\collector-validation-manifest.json -Raw | ConvertFrom-Json).pages
$prod = (Get-Content data\manifest.json -Raw | ConvertFrom-Json).pages

foreach ($code in @("R2.40", "U8.25")) {
    $t = $temp | Where-Object section_code -eq $code
    $p = $prod | Where-Object section_code -eq $code

    [PSCustomObject]@{
        Section       = $code
        TempHash      = $t.content_hash
        CanonicalHash = $p.content_hash
        Match         = $t.content_hash -eq $p.content_hash
    }
}"#;

        let normalized = normalize_direct_command(command, "powershell").unwrap();

        assert!(normalized.starts_with("& {\n$temp ="));
        assert!(normalized.contains("foreach ($code in @(\"R2.40\", \"U8.25\")) {"));
        assert!(normalized.contains("[PSCustomObject]@{\n"));
        assert!(normalized.ends_with("\n}\n}"));
    }

    #[test]
    fn normalizes_a_variable_prelude_and_foreach_object_range_as_one_powershell_script() {
        let command = r#"$lines = Get-Content src\index.css

80..180 | ForEach-Object {
    "{0,4}: {1}" -f ($_ + 1), $lines[$_]
}"#;

        let normalized = normalize_direct_command(command, "powershell").unwrap();

        assert!(normalized.starts_with("& {\n$lines = Get-Content src\\index.css"));
        assert!(normalized.contains("80..180 | ForEach-Object {"));
        assert!(normalized.contains("$lines[$_]"));
        assert!(normalized.ends_with("\n}\n}"));
    }

    #[test]
    fn normalizes_a_scoped_environment_workflow_as_one_powershell_script() {
        let command = r#"$env:CORS_ORIGIN="https://waypoint.example.com"
Set-Location .\backend
uv run python -c "from app.config import get_settings; print(get_settings().cors_origin)"
Set-Location ..
Remove-Item Env:CORS_ORIGIN"#;

        let normalized = normalize_direct_command(command, "powershell").unwrap();

        assert!(normalized.starts_with("& {\n$env:CORS_ORIGIN="));
        assert!(normalized.contains("Set-Location .\\backend\n"));
        assert!(normalized.contains("Remove-Item Env:CORS_ORIGIN"));
        assert!(normalized.ends_with("\n}"));
    }

    #[test]
    fn normalizes_dependent_powershell_variables_as_one_script() {
        let docker_command = r#"$container = (docker inspect waypoint-db | ConvertFrom-Json)[0]
$container.Mounts | Where-Object { $_.Destination -eq "/var/lib/postgresql/data" } | Select-Object Type, Name, Source, Destination"#;
        let rest_command = r#"$body = @{
    question = "Can I work during university holidays?"
} | ConvertTo-Json -Compress
$response = Invoke-RestMethod -Uri "http://localhost:8100/ask" -Method Post -ContentType "application/json" -Body $body
$response | ConvertTo-Json -Depth 10"#;

        let docker_normalized = normalize_direct_command(docker_command, "powershell").unwrap();
        assert!(docker_normalized.starts_with("& {\n$container ="));
        assert!(docker_normalized.contains("$container.Mounts"));
        assert!(docker_normalized.ends_with("\n}"));

        let rest_normalized = normalize_direct_command(rest_command, "powershell").unwrap();
        assert!(rest_normalized.starts_with("& {\n$body = @{"));
        assert!(rest_normalized.contains("-Body $body"));
        assert!(rest_normalized.contains("$response | ConvertTo-Json"));
        assert!(rest_normalized.ends_with("\n}"));
    }

    #[test]
    fn does_not_group_independent_powershell_commands_after_an_assignment() {
        let error = normalize_direct_command("$value = 1\nnpm run build", "powershell")
            .unwrap_err();
        assert!(error.contains("one command at a time"));
    }

    #[test]
    fn foreach_script_support_does_not_allow_commands_after_the_loop() {
        let command = "$items = @(1, 2)\nforeach ($item in $items) {\n  $item\n}\nRemove-Item .";
        let error = normalize_direct_command(command, "powershell").unwrap_err();
        assert!(error.contains("one command at a time"));
    }

    #[test]
    fn grouped_multiline_support_does_not_allow_a_second_command() {
        let command = "Write-Output (\n  'first'\n)\nWrite-Output 'second'";
        let error = normalize_direct_command(command, "powershell").unwrap_err();
        assert!(error.contains("one command at a time"));

        let bash_error = normalize_direct_command("printf (\n  test\n)", "bash").unwrap_err();
        assert!(bash_error.contains("one command at a time"));
    }

    #[test]
    fn rejects_incomplete_or_mismatched_powershell_groups() {
        let incomplete = normalize_direct_command("Write-Output (\n  'value'", "powershell")
            .unwrap_err();
        assert!(incomplete.contains("Complete the PowerShell"));

        let mismatched = normalize_direct_command("Write-Output (\n  'value'\n]", "powershell")
            .unwrap_err();
        assert!(mismatched.contains("mismatched"));
    }

    #[test]
    fn rejects_multiple_direct_commands_in_one_submission() {
        let error = normalize_direct_command("az account show\naz group list", "powershell")
            .unwrap_err();
        assert!(error.contains("one command at a time"));
    }

    /// Spawns the actual PyInstaller-built sidecar binary (the same one Tauri's
    /// shell plugin launches in production) directly via `std::process::Command`,
    /// since unit tests have no `AppHandle` to go through `tauri_plugin_shell`
    /// with. Everything downstream of the port/token it hands back exercises
    /// the real process, the real HTTP+auth wire protocol, and the real local
    /// intent matcher — only the launch mechanism differs from production.
    struct RunningSidecar {
        child: Child,
        endpoint: String,
        token: String,
    }

    impl Drop for RunningSidecar {
        fn drop(&mut self) {
            let _ = Command::new("taskkill")
                .args(["/T", "/F", "/PID", &self.child.id().to_string()])
                .output();
        }
    }

    fn spawn_real_sidecar() -> RunningSidecar {
        let binary = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("binaries")
            .join("terminal-mate-sidecar-x86_64-pc-windows-msvc.exe");
        assert!(
            binary.exists(),
            "sidecar binary not found at {binary:?} — run `npm run sidecar:build` first"
        );

        let token = "test-token-0123456789abcdef0123456789abcdef".to_owned();
        let mut child = Command::new(&binary)
            .env("TERMINAL_MATE_SESSION_TOKEN", &token)
            .env("TERMINAL_MATE_ENV", "test")
            .env("TERMINAL_MATE_PARENT_PID", std::process::id().to_string())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .expect("the real sidecar binary should spawn");

        let stdout = child.stdout.take().expect("sidecar stdout should be piped");
        let mut reader = std::io::BufReader::new(stdout);
        let mut line = String::new();
        reader
            .read_line(&mut line)
            .expect("the sidecar should print a readiness line");
        let payload = line
            .trim()
            .strip_prefix("TERMINAL_MATE_READY:")
            .expect("the sidecar's first stdout line should be the readiness message");
        let parsed: serde_json::Value =
            serde_json::from_str(payload).expect("the readiness payload should be valid JSON");
        let port = parsed["port"]
            .as_u64()
            .expect("the readiness payload should include a port");

        RunningSidecar {
            child,
            endpoint: format!("http://127.0.0.1:{port}"),
            token,
        }
    }

    async fn configured_ready_state(sidecar: &RunningSidecar) -> AppState {
        let state = AppState::default();
        state
            .configure_sidecar(sidecar.endpoint.clone(), sidecar.token.clone(), "1".to_owned())
            .await;
        state.ready().await;
        state
    }

    #[cfg(target_os = "windows")]
    #[tokio::test]
    async fn resolves_a_navigation_request_through_the_real_sidecar() {
        let sidecar = spawn_real_sidecar();
        let state = configured_ready_state(&sidecar).await;
        let proxy = SidecarProxy::new().expect("the proxy should build");

        let rendered = try_interpret("where am i", &windows_test_profile(), &state, &proxy)
            .await
            .expect("the sidecar request should complete")
            .expect("the real sidecar should match this request and it should render");

        assert_eq!(rendered, "Get-Location");
    }

    #[cfg(target_os = "windows")]
    #[tokio::test]
    async fn resolves_a_script_request_through_the_real_sidecar() {
        let sidecar = spawn_real_sidecar();
        let state = configured_ready_state(&sidecar).await;
        let proxy = SidecarProxy::new().expect("the proxy should build");

        let rendered = try_interpret(
            "run trim_generic_residence.py",
            &windows_test_profile(),
            &state,
            &proxy,
        )
        .await
        .expect("the sidecar request should complete")
        .expect("the real sidecar should match this request and it should render");

        assert!(rendered.contains("Get-Item -LiteralPath 'trim_generic_residence.py'"));
        assert!(rendered.contains("& python -u $script.FullName"));
        assert!(!rendered.starts_with("run "));
    }

    #[cfg(target_os = "windows")]
    #[tokio::test]
    async fn resolves_an_azure_provider_registration_check_through_the_real_sidecar() {
        let sidecar = spawn_real_sidecar();
        let state = configured_ready_state(&sidecar).await;
        let proxy = SidecarProxy::new().expect("the proxy should build");

        let rendered = try_interpret(
            "Check if the Azure Storage provider is registered",
            &windows_test_profile(),
            &state,
            &proxy,
        )
        .await
        .expect("the sidecar request should complete")
        .expect("the real sidecar should match this request and it should render");

        assert!(rendered.starts_with("az account show --output none"));
        assert!(rendered.contains(
            "az provider show --namespace 'Microsoft.Storage' --query registrationState"
        ));
    }

    #[cfg(target_os = "windows")]
    #[tokio::test]
    async fn resolves_an_azure_account_clear_request_through_the_real_sidecar() {
        let sidecar = spawn_real_sidecar();
        let state = configured_ready_state(&sidecar).await;
        let proxy = SidecarProxy::new().expect("the proxy should build");

        let rendered = try_interpret(
            "Clear my Azure login",
            &windows_test_profile(),
            &state,
            &proxy,
        )
        .await
        .expect("the sidecar request should complete")
        .expect("the real sidecar should match this request and it should render");

        assert_eq!(rendered, "az account clear");
    }

    #[cfg(target_os = "windows")]
    #[tokio::test]
    async fn resolves_a_find_files_request_through_the_real_sidecar() {
        let sidecar = spawn_real_sidecar();
        let state = configured_ready_state(&sidecar).await;
        let proxy = SidecarProxy::new().expect("the proxy should build");

        let rendered = try_interpret(
            "show me all the .md files",
            &windows_test_profile(),
            &state,
            &proxy,
        )
        .await
        .expect("the sidecar request should complete")
        .expect("the real sidecar should match this request and it should render");

        assert_eq!(
            rendered,
            "Get-ChildItem -LiteralPath '.' -Filter '*.md' -File -Recurse -ErrorAction SilentlyContinue"
        );

        let shorthand = try_interpret(
            "show all .py",
            &windows_test_profile(),
            &state,
            &proxy,
        )
        .await
        .expect("the shorthand sidecar request should complete")
        .expect("the shorthand extension request should render");

        assert_eq!(
            shorthand,
            "Get-ChildItem -LiteralPath '.' -Filter '*.py' -File -Recurse -ErrorAction SilentlyContinue"
        );
    }

    #[cfg(target_os = "windows")]
    #[tokio::test]
    async fn resolves_a_view_lines_request_through_the_real_sidecar() {
        let sidecar = spawn_real_sidecar();
        let state = configured_ready_state(&sidecar).await;
        let proxy = SidecarProxy::new().expect("the proxy should build");

        let rendered = try_interpret(
            "show me lines 20-100 of sample.txt",
            &windows_test_profile(),
            &state,
            &proxy,
        )
        .await
        .expect("the sidecar request should complete")
        .expect("the real sidecar should match this request and it should render");

        assert_eq!(
            rendered,
            "Get-Content -LiteralPath 'sample.txt' | Select-Object -Skip 19 -First 81"
        );
    }

    #[cfg(target_os = "windows")]
    #[tokio::test]
    async fn resolves_a_go_up_directory_request_through_the_real_sidecar() {
        let sidecar = spawn_real_sidecar();
        let state = configured_ready_state(&sidecar).await;
        let proxy = SidecarProxy::new().expect("the proxy should build");

        let rendered = try_interpret("go back", &windows_test_profile(), &state, &proxy)
            .await
            .expect("the sidecar request should complete")
            .expect("the real sidecar should match this request and it should render");

        assert_eq!(rendered, "Set-Location -LiteralPath '..'");
    }

    #[cfg(target_os = "windows")]
    #[tokio::test]
    async fn normalizes_workspace_relative_navigation_through_the_real_sidecar() {
        let sidecar = spawn_real_sidecar();
        let state = configured_ready_state(&sidecar).await;
        let proxy = SidecarProxy::new().expect("the proxy should build");

        let rendered = try_interpret(
            r"go to \backend\tests\",
            &windows_test_profile(),
            &state,
            &proxy,
        )
        .await
        .expect("the sidecar request should complete")
        .expect("the real sidecar should normalize and render navigation");

        assert_eq!(rendered, "Set-Location -LiteralPath 'backend\\tests'");
    }

    #[cfg(target_os = "windows")]
    #[tokio::test]
    async fn resolves_a_list_directory_request_through_the_real_sidecar() {
        let sidecar = spawn_real_sidecar();
        let state = configured_ready_state(&sidecar).await;
        let proxy = SidecarProxy::new().expect("the proxy should build");

        let rendered = try_interpret(
            "what's in this folder?",
            &windows_test_profile(),
            &state,
            &proxy,
        )
        .await
        .expect("the sidecar request should complete")
        .expect("the real sidecar should match this request and it should render");

        assert_eq!(rendered, "Get-ChildItem -LiteralPath '.'");
    }

    #[cfg(target_os = "windows")]
    #[tokio::test]
    async fn resolves_a_named_path_request_through_the_real_sidecar() {
        let sidecar = spawn_real_sidecar();
        let state = configured_ready_state(&sidecar).await;
        let proxy = SidecarProxy::new().expect("the proxy should build");

        let rendered = try_interpret(
            "List files .claude in this folder",
            &windows_test_profile(),
            &state,
            &proxy,
        )
        .await
        .expect("the sidecar request should complete")
        .expect("the real sidecar should match this request and it should render");

        assert_eq!(
            rendered,
            "Get-Item -LiteralPath '.claude' -ErrorAction Stop | ForEach-Object { $kind = if ($_.PSIsContainer) { 'Folder' } else { 'File' }; Write-Output \"Target: $($_.FullName)\"; Write-Output \"Type: $kind\"; if ($_.PSIsContainer) { Write-Output 'Contents:'; Get-ChildItem -LiteralPath $_.FullName } else { $_ | Format-List FullName,Length,LastWriteTime,Attributes } }"
        );
    }

    #[cfg(target_os = "windows")]
    #[tokio::test]
    async fn resolves_a_make_directory_request_through_the_real_sidecar() {
        let sidecar = spawn_real_sidecar();
        let state = configured_ready_state(&sidecar).await;
        let proxy = SidecarProxy::new().expect("the proxy should build");

        let rendered = try_interpret(
            "create a folder called drafts",
            &windows_test_profile(),
            &state,
            &proxy,
        )
        .await
        .expect("the sidecar request should complete")
        .expect("the real sidecar should match this request and it should render");

        assert_eq!(rendered, "New-Item -ItemType Directory -Path 'drafts'");
    }

    #[cfg(target_os = "windows")]
    #[tokio::test]
    async fn resolves_a_workspace_relative_create_file_through_the_real_sidecar() {
        let sidecar = spawn_real_sidecar();
        let state = configured_ready_state(&sidecar).await;
        let proxy = SidecarProxy::new().expect("the proxy should build");

        let rendered = try_interpret(
            r"create file \scripts\freeze_holdout_source_sample.py",
            &windows_test_profile(),
            &state,
            &proxy,
        )
        .await
        .expect("the sidecar request should complete")
        .expect("the real sidecar should match and render file creation");

        assert_eq!(
            rendered,
            "New-Item -ItemType File -Path 'scripts/freeze_holdout_source_sample.py' -ErrorAction Stop"
        );
    }

    #[cfg(target_os = "windows")]
    #[tokio::test]
    async fn resolves_a_workspace_relative_delete_file_through_the_real_sidecar() {
        let sidecar = spawn_real_sidecar();
        let state = configured_ready_state(&sidecar).await;
        let proxy = SidecarProxy::new().expect("the proxy should build");

        let rendered = try_interpret(
            r"remove file \scripts\freeze_holdout_source_sample.py",
            &windows_test_profile(),
            &state,
            &proxy,
        )
        .await
        .expect("the sidecar request should complete")
        .expect("the real sidecar should match and render file deletion");

        assert_eq!(
            rendered,
            "$target = Get-Item -LiteralPath 'scripts/freeze_holdout_source_sample.py' -ErrorAction Stop; if ($target.PSIsContainer) { throw 'The selected path is a folder. Use an explicit folder-deletion workflow instead.' }; Remove-Item -LiteralPath $target.FullName -ErrorAction Stop"
        );
    }

    #[cfg(target_os = "windows")]
    #[tokio::test]
    async fn resolves_an_external_file_copy_into_the_workspace() {
        let sidecar = spawn_real_sidecar();
        let state = configured_ready_state(&sidecar).await;
        let proxy = SidecarProxy::new().expect("the proxy should build");

        let rendered = try_interpret(
            r"copy file freeze_external_adjudication.py from D:\Data\Downloads to \scripts",
            &windows_test_profile(),
            &state,
            &proxy,
        )
        .await
        .expect("the sidecar request should complete")
        .expect("the real sidecar should match and render file copying");

        assert!(rendered.contains(
            "Get-Item -LiteralPath 'D:\\Data\\Downloads\\freeze_external_adjudication.py'"
        ));
        assert!(rendered.contains("Get-Item -LiteralPath 'scripts'"));
        assert!(rendered.contains("Copy-Item -LiteralPath $source.FullName"));
        assert!(rendered.contains("Destination file already exists"));
    }

    #[cfg(target_os = "windows")]
    #[tokio::test]
    async fn resolves_a_search_file_contents_request_through_the_real_sidecar() {
        let sidecar = spawn_real_sidecar();
        let state = configured_ready_state(&sidecar).await;
        let proxy = SidecarProxy::new().expect("the proxy should build");

        let rendered = try_interpret(
            "search for TODO in this folder",
            &windows_test_profile(),
            &state,
            &proxy,
        )
        .await
        .expect("the sidecar request should complete")
        .expect("the real sidecar should match this request and it should render");

        assert_eq!(
            rendered,
            "Get-ChildItem -LiteralPath '.' -Recurse -File -ErrorAction SilentlyContinue | Select-String -Pattern 'TODO'"
        );
    }

    #[tokio::test]
    async fn falls_back_to_none_when_the_sidecar_was_never_configured() {
        let state = AppState::default();
        let proxy = SidecarProxy::new().expect("the proxy should build");

        let result = try_interpret("where am i", &windows_test_profile(), &state, &proxy)
            .await
            .expect("an unavailable sidecar should fall back cleanly");

        assert!(result.is_none());
    }

    #[cfg(target_os = "windows")]
    #[tokio::test]
    async fn falls_back_to_none_for_an_unmatched_request_through_the_real_sidecar() {
        let sidecar = spawn_real_sidecar();
        let state = configured_ready_state(&sidecar).await;
        let proxy = SidecarProxy::new().expect("the proxy should build");

        let result = try_interpret(
            "please reticulate the splines",
            &windows_test_profile(),
            &state,
            &proxy,
        )
        .await
        .expect("an unmatched request should fall back cleanly");

        assert!(result.is_none());
    }

    #[cfg(target_os = "windows")]
    #[tokio::test]
    async fn returns_guidance_for_an_incomplete_request_through_the_real_sidecar() {
        let sidecar = spawn_real_sidecar();
        let state = configured_ready_state(&sidecar).await;
        let proxy = SidecarProxy::new().expect("the proxy should build");

        let error = try_interpret(
            "Inspect a specific port",
            &windows_test_profile(),
            &state,
            &proxy,
        )
        .await
        .expect_err("an incomplete request should ask for the missing port");

        assert!(error.contains("Which port"));
    }
}
