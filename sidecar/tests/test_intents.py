import pytest


@pytest.mark.parametrize(
    ("message", "action", "parameters"),
    [
        ("Show Azure CLI version", "azure_cli_version", {}),
        ("Show Terraform version", "terraform_version", {}),
        ("Show current Azure account", "azure_account_show", {}),
        ("List Azure subscriptions", "azure_subscription_list", {}),
        ("Sign in to Azure", "azure_login", {}),
        ("Sign in to Azure with device code", "azure_login_device_code", {}),
        (
            "Switch Azure subscription to Production Sub",
            "azure_subscription_set",
            {"subscription": "Production Sub"},
        ),
        (
            "Generate SSH key deploy-key with comment terminalmate",
            "generate_ssh_key",
            {"key_name": "deploy-key", "comment": "terminalmate"},
        ),
        (
            "Show the public key named deploy-key",
            "show_ssh_public_key",
            {"key_name": "deploy-key"},
        ),
        (
            "Show SSH public key for deploy-key",
            "show_ssh_public_key",
            {"key_name": "deploy-key"},
        ),
        (
            "Create Azure resource group demo-rg in australiaeast",
            "azure_resource_group_create",
            {"resource_group": "demo-rg", "location": "australiaeast"},
        ),
        (
            "Create Azure storage account demostore in resource group demo-rg",
            "azure_storage_account_create",
            {"account_name": "demostore", "resource_group": "demo-rg"},
        ),
        (
            "Create Azure storage container uploads in storage account demostore",
            "azure_storage_container_create",
            {"container_name": "uploads", "account_name": "demostore"},
        ),
        (
            "List Azure storage account keys for demostore in resource group demo-rg",
            "azure_storage_keys_list",
            {"account_name": "demostore", "resource_group": "demo-rg"},
        ),
        ("Terraform init", "terraform_init", {}),
        ("Format Terraform files", "terraform_fmt", {}),
        ("Validate Terraform", "terraform_validate", {}),
        ("Terraform plan", "terraform_plan", {}),
        ("Terraform apply", "terraform_apply", {}),
        ("Terraform destroy", "terraform_destroy", {}),
        ("List Terraform state", "terraform_state_list", {}),
        ("Show Terraform outputs", "terraform_output", {}),
        (
            "Terraform state show azurerm_resource_group.demo",
            "terraform_state_show",
            {"address": "azurerm_resource_group.demo"},
        ),
        (
            "Terraform import azurerm_resource_group.demo /subscriptions/example/resourceGroups/demo-rg",
            "terraform_import",
            {
                "address": "azurerm_resource_group.demo",
                "resource_id": "/subscriptions/example/resourceGroups/demo-rg",
            },
        ),
        ("List Azure VMs", "azure_vm_list", {}),
        (
            "Show Azure VM app-vm in resource group demo-rg",
            "azure_vm_show",
            {"vm_name": "app-vm", "resource_group": "demo-rg"},
        ),
        ("List Azure NSGs", "azure_nsg_list", {}),
        (
            "List Azure resources in resource group demo-rg",
            "azure_resource_list",
            {"resource_group": "demo-rg"},
        ),
        (
            "Start Azure VM app-vm in resource group demo-rg",
            "azure_vm_start",
            {"vm_name": "app-vm", "resource_group": "demo-rg"},
        ),
        (
            "Stop Azure VM app-vm in resource group demo-rg",
            "azure_vm_stop",
            {"vm_name": "app-vm", "resource_group": "demo-rg"},
        ),
        (
            "Deallocate Azure VM app-vm in resource group demo-rg",
            "azure_vm_deallocate",
            {"vm_name": "app-vm", "resource_group": "demo-rg"},
        ),
        (
            "SSH admin@203.0.113.5 using key deploy-key",
            "ssh_vm",
            {"admin": "admin", "ip": "203.0.113.5", "key_name": "deploy-key"},
        ),
        (
            "Delete Azure resource group demo-rg",
            "azure_resource_group_delete",
            {"resource_group": "demo-rg"},
        ),
        ("Clear my Azure login", "azure_account_clear", {}),
        ("Log out of Azure completely", "azure_account_clear", {}),
        (
            "Check if the Azure Storage provider is registered",
            "azure_provider_show",
            {"namespace": "Microsoft.Storage"},
        ),
        (
            "Is the az Storage provider registered?",
            "azure_provider_show",
            {"namespace": "Microsoft.Storage"},
        ),
        (
            "Register the Azure Storage provider",
            "azure_provider_register",
            {"namespace": "Microsoft.Storage"},
        ),
        (
            "Register the az Microsoft.Sql provider",
            "azure_provider_register",
            {"namespace": "Microsoft.Sql"},
        ),
    ],
)
def test_matches_azure_and_terraform_intents(
    client, auth_headers, windows_profile, message, action, parameters
):
    response = client.post(
        "/v1/intents/interpret",
        headers=auth_headers,
        json={"message": message, "execution_profile": windows_profile},
    )

    assert response.status_code == 200
    payload = response.json()
    assert payload["matched"] is True
    assert payload["intent"] == {
        "schema_version": 1,
        "action": action,
        "parameters": parameters,
    }


def test_azure_resource_group_creation_requests_missing_parameters(
    client, auth_headers, windows_profile
):
    response = client.post(
        "/v1/intents/interpret",
        headers=auth_headers,
        json={
            "message": "Create Azure resource group",
            "execution_profile": windows_profile,
        },
    )

    assert response.status_code == 200
    payload = response.json()
    assert payload["matched"] is False
    assert payload["requires_clarification"] is True
    assert "region" in payload["message"]


@pytest.mark.parametrize(
    "message",
    [
        "Start vm app-vm in resource group demo-rg",
        "Deallocate vm app-vm in resource group demo-rg",
        "Create resource group demo-rg in australiaeast",
        "Is the Storage provider registered?",
    ],
)
def test_azure_vm_and_resource_requests_without_a_cloud_marker_do_not_match(
    client, auth_headers, windows_profile, message
):
    # Every Azure pattern requires a literal "az"/"azure" marker so a future
    # AWS adapter can use the identical verb/noun shape ("start aws vm X")
    # without colliding with these — a bare, marker-less phrase should not
    # be silently assumed to mean Azure.
    response = client.post(
        "/v1/intents/interpret",
        headers=auth_headers,
        json={"message": message, "execution_profile": windows_profile},
    )

    assert response.status_code == 200
    payload = response.json()
    assert payload["matched"] is False


def test_matches_workspace_navigation(client, auth_headers, windows_profile):
    response = client.post(
        "/v1/intents/interpret",
        headers=auth_headers,
        json={
            "message": "Take me to the terminal-mate directory.",
            "execution_profile": windows_profile,
        },
    )

    assert response.status_code == 200
    payload = response.json()
    assert payload["matched"] is True
    assert payload["source"] == "local"
    assert payload["intent"] == {
        "schema_version": 1,
        "action": "change_directory",
        "parameters": {"target": "terminal-mate"},
    }


def test_normalizes_workspace_relative_windows_navigation(
    client,
    auth_headers,
    windows_profile,
):
    response = client.post(
        "/v1/intents/interpret",
        headers=auth_headers,
        json={
            "message": "Go to \\backend\\tests\\",
            "execution_profile": windows_profile,
        },
    )

    assert response.status_code == 200
    assert response.json()["intent"] == {
        "schema_version": 1,
        "action": "change_directory",
        "parameters": {"target": r"backend\tests"},
    }


def test_preserves_absolute_navigation_roots(client, auth_headers, windows_profile):
    drive_response = client.post(
        "/v1/intents/interpret",
        headers=auth_headers,
        json={
            "message": "Go to D:\\tools\\scripts\\",
            "execution_profile": windows_profile,
        },
    )
    unc_response = client.post(
        "/v1/intents/interpret",
        headers=auth_headers,
        json={
            "message": "Go to \\\\server\\share\\scripts\\",
            "execution_profile": windows_profile,
        },
    )
    linux_response = client.post(
        "/v1/intents/interpret",
        headers=auth_headers,
        json={
            "message": "Go to /opt/tools/scripts/",
            "execution_profile": windows_profile,
        },
    )

    assert drive_response.json()["intent"]["parameters"]["target"] == r"D:\tools\scripts"
    assert unc_response.json()["intent"]["parameters"]["target"] == r"\\server\share\scripts"
    assert linux_response.json()["intent"]["parameters"]["target"] == "/opt/tools/scripts"


def test_matches_workspace_navigation_to_home_directory(client, auth_headers, windows_profile):
    response = client.post(
        "/v1/intents/interpret",
        headers=auth_headers,
        json={
            "message": "Take me to the home directory.",
            "execution_profile": windows_profile,
        },
    )

    assert response.status_code == 200
    assert response.json()["intent"] == {
        "schema_version": 1,
        "action": "change_directory",
        "parameters": {"target": "home"},
    }


def test_matches_single_script_execution(client, auth_headers, windows_profile):
    response = client.post(
        "/v1/intents/interpret",
        headers=auth_headers,
        json={
            "message": "Run scripts/seed_data.py.",
            "execution_profile": windows_profile,
        },
    )

    assert response.status_code == 200
    assert response.json()["intent"] == {
        "schema_version": 1,
        "action": "run_script",
        "parameters": {"path": "scripts/seed_data.py"},
    }


def test_matches_python_script_execution_with_arguments(
    client, auth_headers, windows_profile
):
    response = client.post(
        "/v1/intents/interpret",
        headers=auth_headers,
        json={
            "message": (
                "Run python collect_manual.py --prefixes R "
                "--out ../generic-residence"
            ),
            "execution_profile": windows_profile,
        },
    )

    assert response.status_code == 200
    assert response.json()["intent"] == {
        "schema_version": 1,
        "action": "run_script",
        "parameters": {
            "path": "collect_manual.py",
            "arguments": ["--prefixes", "R", "--out", "../generic-residence"],
            "interpreter": "python",
        },
    }


def test_preserves_quoted_script_argument_as_one_value(
    client, auth_headers, windows_profile
):
    response = client.post(
        "/v1/intents/interpret",
        headers=auth_headers,
        json={
            "message": 'Run scripts/report.py --title "Residence category".',
            "execution_profile": windows_profile,
        },
    )

    assert response.status_code == 200
    assert response.json()["intent"]["parameters"] == {
        "path": "scripts/report.py",
        "arguments": ["--title", "Residence category"],
    }


def test_script_shortcut_does_not_capture_a_generic_run_request(
    client, auth_headers, windows_profile
):
    response = client.post(
        "/v1/intents/interpret",
        headers=auth_headers,
        json={"message": "Run docker ps.", "execution_profile": windows_profile},
    )

    assert response.status_code == 200
    assert response.json()["matched"] is False


@pytest.mark.parametrize(
    ("message", "expected_order", "recursive", "extension"),
    [
        ("Run all scripts in this folder.", "modified_desc", False, None),
        ("Run all scripts in this folder by name.", "name", False, None),
        ("Run all scripts in this folder oldest first.", "modified_asc", False, None),
        ("Run all Python scripts recursively.", "modified_desc", True, ".py"),
    ],
)
def test_matches_ordered_script_collection_execution(
    client,
    auth_headers,
    windows_profile,
    message,
    expected_order,
    recursive,
    extension,
):
    response = client.post(
        "/v1/intents/interpret",
        headers=auth_headers,
        json={"message": message, "execution_profile": windows_profile},
    )

    assert response.status_code == 200
    parameters = {
        "root": ".",
        "order": expected_order,
        "recursive": recursive,
    }
    if extension:
        parameters["extension"] = extension
    assert response.json()["intent"] == {
        "schema_version": 1,
        "action": "run_scripts",
        "parameters": parameters,
    }


def test_matches_script_order_preview(client, auth_headers, windows_profile):
    response = client.post(
        "/v1/intents/interpret",
        headers=auth_headers,
        json={
            "message": "Show script execution order in this folder by filename.",
            "execution_profile": windows_profile,
        },
    )

    assert response.status_code == 200
    assert response.json()["intent"] == {
        "schema_version": 1,
        "action": "list_scripts",
        "parameters": {"root": ".", "order": "name", "recursive": False},
    }


def test_matches_recursive_shell_file_search(client, auth_headers, windows_profile):
    response = client.post(
        "/v1/intents/interpret",
        headers=auth_headers,
        json={
            "message": "Show me all the .sh files.",
            "execution_profile": windows_profile,
        },
    )

    assert response.status_code == 200
    assert response.json()["intent"] == {
        "schema_version": 1,
        "action": "find_files",
        "parameters": {
            "root": ".",
            "pattern": "*.sh",
            "recursive": True,
        },
    }


def test_matches_recursive_text_file_search(client, auth_headers, windows_profile):
    response = client.post(
        "/v1/intents/interpret",
        headers=auth_headers,
        json={
            "message": "Show me all the .txt files.",
            "execution_profile": windows_profile,
        },
    )

    assert response.status_code == 200
    assert response.json()["intent"] == {
        "schema_version": 1,
        "action": "find_files",
        "parameters": {
            "root": ".",
            "pattern": "*.txt",
            "recursive": True,
        },
    }


def test_matches_view_file_lines_range_first(client, auth_headers, windows_profile):
    response = client.post(
        "/v1/intents/interpret",
        headers=auth_headers,
        json={
            "message": "Show me lines 20-100 of sample.txt.",
            "execution_profile": windows_profile,
        },
    )

    assert response.status_code == 200
    assert response.json()["intent"] == {
        "schema_version": 1,
        "action": "view_file_lines",
        "parameters": {
            "path": "sample.txt",
            "start_line": 20,
            "end_line": 100,
        },
    }


def test_matches_view_file_lines_path_first(client, auth_headers, windows_profile):
    response = client.post(
        "/v1/intents/interpret",
        headers=auth_headers,
        json={
            "message": "Show me the content of sample.txt lines 20 to 100.",
            "execution_profile": windows_profile,
        },
    )

    assert response.status_code == 200
    assert response.json()["intent"] == {
        "schema_version": 1,
        "action": "view_file_lines",
        "parameters": {
            "path": "sample.txt",
            "start_line": 20,
            "end_line": 100,
        },
    }


def test_matches_go_up_one_directory(client, auth_headers, windows_profile):
    response = client.post(
        "/v1/intents/interpret",
        headers=auth_headers,
        json={
            "message": "Go up one directory.",
            "execution_profile": windows_profile,
        },
    )

    assert response.status_code == 200
    assert response.json()["intent"] == {
        "schema_version": 1,
        "action": "change_directory",
        "parameters": {"target": ".."},
    }


def test_matches_go_back(client, auth_headers, windows_profile):
    response = client.post(
        "/v1/intents/interpret",
        headers=auth_headers,
        json={
            "message": "Go back.",
            "execution_profile": windows_profile,
        },
    )

    assert response.status_code == 200
    assert response.json()["intent"] == {
        "schema_version": 1,
        "action": "change_directory",
        "parameters": {"target": ".."},
    }


def test_matches_list_directory(client, auth_headers, windows_profile):
    response = client.post(
        "/v1/intents/interpret",
        headers=auth_headers,
        json={
            "message": "List the files here.",
            "execution_profile": windows_profile,
        },
    )

    assert response.status_code == 200
    assert response.json()["intent"] == {
        "schema_version": 1,
        "action": "list_directory",
        "parameters": {"root": "."},
    }


@pytest.mark.parametrize(
    "message",
    [
        "Show hidden files with name and mode.",
        "List all files and folders, including hidden items, and show their name and mode.",
    ],
)
def test_matches_hidden_items_with_name_and_mode(
    client,
    auth_headers,
    windows_profile,
    message,
):
    response = client.post(
        "/v1/intents/interpret",
        headers=auth_headers,
        json={
            "message": message,
            "execution_profile": windows_profile,
        },
    )

    assert response.status_code == 200
    assert response.json()["intent"] == {
        "schema_version": 1,
        "action": "list_hidden_items_with_details",
        "parameters": {"root": "."},
    }


@pytest.mark.parametrize(
    "message",
    [
        "List files .claude in this folder.",
        "List .claude in this folder.",
    ],
)
def test_matches_inspect_named_path(
    client,
    auth_headers,
    windows_profile,
    message,
):
    response = client.post(
        "/v1/intents/interpret",
        headers=auth_headers,
        json={
            "message": message,
            "execution_profile": windows_profile,
        },
    )

    assert response.status_code == 200
    assert response.json()["intent"] == {
        "schema_version": 1,
        "action": "inspect_path",
        "parameters": {"path": ".claude"},
    }


def test_matches_whats_in_this_folder(client, auth_headers, windows_profile):
    response = client.post(
        "/v1/intents/interpret",
        headers=auth_headers,
        json={
            "message": "What's in this folder?",
            "execution_profile": windows_profile,
        },
    )

    assert response.status_code == 200
    assert response.json()["intent"] == {
        "schema_version": 1,
        "action": "list_directory",
        "parameters": {"root": "."},
    }


def test_matches_make_directory(client, auth_headers, windows_profile):
    response = client.post(
        "/v1/intents/interpret",
        headers=auth_headers,
        json={
            "message": "Create a folder called drafts.",
            "execution_profile": windows_profile,
        },
    )

    assert response.status_code == 200
    assert response.json()["intent"] == {
        "schema_version": 1,
        "action": "make_directory",
        "parameters": {"path": "drafts"},
    }


@pytest.mark.parametrize(
    ("message", "action", "paths"),
    [
        (
            "Create folders components, api, components",
            "make_directories",
            ["components", "api"],
        ),
        (
            "Create files src/index.ts, src/app.ts, README.md",
            "create_files",
            ["src/index.ts", "src/app.ts", "README.md"],
        ),
    ],
)
def test_matches_comma_delimited_creation(
    client, auth_headers, windows_profile, message, action, paths
):
    response = client.post(
        "/v1/intents/interpret",
        headers=auth_headers,
        json={"message": message, "execution_profile": windows_profile},
    )

    assert response.status_code == 200
    assert response.json()["intent"] == {
        "schema_version": 1,
        "action": action,
        "parameters": {"paths": paths},
    }


@pytest.mark.parametrize(
    ("message", "expected_path"),
    [
        ("Create file freeze_holdout_source_sample.py", "freeze_holdout_source_sample.py"),
        (
            r"Create file \scripts\freeze_holdout_source_sample.py",
            "scripts/freeze_holdout_source_sample.py",
        ),
    ],
)
def test_matches_workspace_relative_create_file(
    client, auth_headers, windows_profile, message, expected_path
):
    response = client.post(
        "/v1/intents/interpret",
        headers=auth_headers,
        json={"message": message, "execution_profile": windows_profile},
    )

    assert response.status_code == 200
    assert response.json()["intent"] == {
        "schema_version": 1,
        "action": "create_file",
        "parameters": {"path": expected_path},
    }


@pytest.mark.parametrize("verb", ["delete", "remove"])
def test_matches_workspace_relative_delete_file(
    client, auth_headers, windows_profile, verb
):
    response = client.post(
        "/v1/intents/interpret",
        headers=auth_headers,
        json={
            "message": rf"{verb} file \scripts\freeze_holdout_source_sample.py",
            "execution_profile": windows_profile,
        },
    )

    assert response.status_code == 200
    assert response.json()["intent"] == {
        "schema_version": 1,
        "action": "delete_file",
        "parameters": {"path": "scripts/freeze_holdout_source_sample.py"},
    }


@pytest.mark.parametrize(
    "message",
    [
        r"Create file ..\outside.py",
        r"Delete file D:\outside.py",
        r"Remove file \\server\share\outside.py",
    ],
)
def test_refuses_file_mutations_outside_the_workspace(
    client, auth_headers, windows_profile, message
):
    response = client.post(
        "/v1/intents/interpret",
        headers=auth_headers,
        json={"message": message, "execution_profile": windows_profile},
    )

    assert response.status_code == 200
    body = response.json()
    assert body["matched"] is False
    assert body["requires_clarification"] is True
    assert "inside the active workspace" in body["message"]


@pytest.mark.parametrize(
    ("message", "action", "source"),
    [
        (
            r"copy file freeze_external_adjudication.py from D:\Data\Downloads to \scripts",
            "copy_file",
            r"D:\Data\Downloads\freeze_external_adjudication.py",
        ),
        (
            r"cut fle D:\Data\Downloads\freeze_external_adjudication.py to \scripts",
            "move_file",
            r"D:\Data\Downloads\freeze_external_adjudication.py",
        ),
    ],
)
def test_matches_file_transfer_into_workspace(
    client, auth_headers, windows_profile, message, action, source
):
    response = client.post(
        "/v1/intents/interpret",
        headers=auth_headers,
        json={"message": message, "execution_profile": windows_profile},
    )

    assert response.status_code == 200
    assert response.json()["intent"] == {
        "schema_version": 1,
        "action": action,
        "parameters": {"source": source, "destination": "scripts"},
    }


def test_refuses_file_transfer_to_destination_outside_workspace(
    client, auth_headers, windows_profile
):
    response = client.post(
        "/v1/intents/interpret",
        headers=auth_headers,
        json={
            "message": r"copy file D:\Data\Downloads\report.py to ..\outside",
            "execution_profile": windows_profile,
        },
    )

    assert response.status_code == 200
    body = response.json()
    assert body["matched"] is False
    assert body["requires_clarification"] is True
    assert "destination folder inside the active workspace" in body["message"]


@pytest.mark.parametrize(
    "message",
    [
        (
            "Back up app/api/routes/ask.py as "
            "app/api/routes/ask_before_evidence_adequacy.py"
        ),
        (
            "Back up app/api/routes/ask.py "
            "app/api/routes/ask_before_evidence_adequacy.py"
        ),
        "Backup app/api/routes/ask.py ask_before_evidence_adequacy.py",
    ],
)
def test_matches_file_backup_with_optional_as(
    client, auth_headers, windows_profile, message
):
    response = client.post(
        "/v1/intents/interpret",
        headers=auth_headers,
        json={"message": message, "execution_profile": windows_profile},
    )

    assert response.status_code == 200
    assert response.json()["intent"] == {
        "schema_version": 1,
        "action": "backup_file",
        "parameters": {
            "source": "app/api/routes/ask.py",
            "backup": "app/api/routes/ask_before_evidence_adequacy.py",
        },
    }


@pytest.mark.parametrize(
    "message",
    [
        (
            "Replace app/api/routes/ask.py with "
            r"D:\Data\Downloads\ask_evidence_adequacy_candidate.py"
        ),
        (
            "Replace app/api/routes/ask.py "
            r"D:\Data\Downloads\ask_evidence_adequacy_candidate.py"
        ),
    ],
)
def test_matches_file_replacement_with_optional_with(
    client, auth_headers, windows_profile, message
):
    response = client.post(
        "/v1/intents/interpret",
        headers=auth_headers,
        json={"message": message, "execution_profile": windows_profile},
    )

    assert response.status_code == 200
    assert response.json()["intent"] == {
        "schema_version": 1,
        "action": "replace_file",
        "parameters": {
            "target": "app/api/routes/ask.py",
            "source": r"D:\Data\Downloads\ask_evidence_adequacy_candidate.py",
        },
    }


@pytest.mark.parametrize(
    "message",
    [
        (
            "Back up app/api/routes/ask.py as "
            "app/api/routes/ask_before_evidence_adequacy.py, then replace "
            "the original with "
            r"D:\Data\Downloads\ask_evidence_adequacy_candidate.py"
        ),
        (
            "Back up app/api/routes/ask.py "
            "app/api/routes/ask_before_evidence_adequacy.py then replace "
            "original "
            r"D:\Data\Downloads\ask_evidence_adequacy_candidate.py"
        ),
        (
            "Backup app/api/routes/ask.py ask_before_evidence_adequacy.py and then "
            "replace app/api/routes/ask.py "
            r"D:\Data\Downloads\ask_evidence_adequacy_candidate.py"
        ),
    ],
)
def test_matches_backup_then_replace_with_optional_connector_words(
    client, auth_headers, windows_profile, message
):
    response = client.post(
        "/v1/intents/interpret",
        headers=auth_headers,
        json={"message": message, "execution_profile": windows_profile},
    )

    assert response.status_code == 200
    assert response.json()["intent"] == {
        "schema_version": 1,
        "action": "backup_and_replace_file",
        "parameters": {
            "target": "app/api/routes/ask.py",
            "backup": "app/api/routes/ask_before_evidence_adequacy.py",
            "source": r"D:\Data\Downloads\ask_evidence_adequacy_candidate.py",
        },
    }


def test_matches_search_file_contents(client, auth_headers, windows_profile):
    response = client.post(
        "/v1/intents/interpret",
        headers=auth_headers,
        json={
            "message": "Search for TODO in this folder.",
            "execution_profile": windows_profile,
        },
    )

    assert response.status_code == 200
    assert response.json()["intent"] == {
        "schema_version": 1,
        "action": "search_file_contents",
        "parameters": {"term": "TODO", "root": "."},
    }


def test_matches_search_file_contents_with_the_text_phrasing(client, auth_headers, windows_profile):
    response = client.post(
        "/v1/intents/interpret",
        headers=auth_headers,
        json={
            "message": "Find the text TODO in files.",
            "execution_profile": windows_profile,
        },
    )

    assert response.status_code == 200
    assert response.json()["intent"] == {
        "schema_version": 1,
        "action": "search_file_contents",
        "parameters": {"term": "TODO", "root": "."},
    }


def test_matches_read_file(client, auth_headers, windows_profile):
    response = client.post(
        "/v1/intents/interpret",
        headers=auth_headers,
        json={
            "message": "Show me the contents of package.json.",
            "execution_profile": windows_profile,
        },
    )

    assert response.status_code == 200
    assert response.json()["intent"] == {
        "schema_version": 1,
        "action": "read_file",
        "parameters": {"path": "package.json"},
    }


def test_matches_file_start_preview(client, auth_headers, windows_profile):
    response = client.post(
        "/v1/intents/interpret",
        headers=auth_headers,
        json={
            "message": "Show the first 30 lines of README.md.",
            "execution_profile": windows_profile,
        },
    )

    assert response.status_code == 200
    assert response.json()["intent"] == {
        "schema_version": 1,
        "action": "view_file_start",
        "parameters": {"path": "README.md", "line_count": 30},
    }


def test_matches_file_end_preview_with_default_count(client, auth_headers, windows_profile):
    response = client.post(
        "/v1/intents/interpret",
        headers=auth_headers,
        json={
            "message": "Show the last lines of app.log.",
            "execution_profile": windows_profile,
        },
    )

    assert response.status_code == 200
    assert response.json()["intent"] == {
        "schema_version": 1,
        "action": "view_file_end",
        "parameters": {"path": "app.log", "line_count": 20},
    }


def test_matches_first_and_last_file_preview(client, auth_headers, windows_profile):
    response = client.post(
        "/v1/intents/interpret",
        headers=auth_headers,
        json={
            "message": "Preview first/last lines of .env.",
            "execution_profile": windows_profile,
        },
    )

    assert response.status_code == 200
    assert response.json()["intent"] == {
        "schema_version": 1,
        "action": "view_file_edges",
        "parameters": {"path": ".env", "line_count": 20},
    }


def test_matches_first_and_last_file_preview_without_preposition(
    client, auth_headers, windows_profile
):
    response = client.post(
        "/v1/intents/interpret",
        headers=auth_headers,
        json={
            "message": "Preview first/last lines .env",
            "execution_profile": windows_profile,
        },
    )

    assert response.status_code == 200
    assert response.json()["intent"] == {
        "schema_version": 1,
        "action": "view_file_edges",
        "parameters": {"path": ".env", "line_count": 20},
    }


def test_requests_a_filename_for_an_incomplete_preview(
    client, auth_headers, windows_profile
):
    response = client.post(
        "/v1/intents/interpret",
        headers=auth_headers,
        json={
            "message": "Preview first/last lines.",
            "execution_profile": windows_profile,
        },
    )

    assert response.status_code == 200
    payload = response.json()
    assert payload["matched"] is False
    assert payload["requires_clarification"] is True
    assert "Which file" in payload["message"]


def test_matches_find_file_by_name(client, auth_headers, windows_profile):
    response = client.post(
        "/v1/intents/interpret",
        headers=auth_headers,
        json={
            "message": "Find the file package.json.",
            "execution_profile": windows_profile,
        },
    )

    assert response.status_code == 200
    assert response.json()["intent"] == {
        "schema_version": 1,
        "action": "find_files",
        "parameters": {"root": ".", "pattern": "package.json", "recursive": True},
    }


def test_matches_list_directories(client, auth_headers, windows_profile):
    response = client.post(
        "/v1/intents/interpret",
        headers=auth_headers,
        json={
            "message": "List the folders here.",
            "execution_profile": windows_profile,
        },
    )

    assert response.status_code == 200
    assert response.json()["intent"] == {
        "schema_version": 1,
        "action": "list_directories",
        "parameters": {"root": "."},
    }


def test_matches_extension_listing_without_files_suffix(
    client, auth_headers, windows_profile
):
    response = client.post(
        "/v1/intents/interpret",
        headers=auth_headers,
        json={
            "message": "show all .py",
            "execution_profile": windows_profile,
        },
    )

    assert response.status_code == 200
    assert response.json()["intent"] == {
        "schema_version": 1,
        "action": "find_files",
        "parameters": {"root": ".", "pattern": "*.py", "recursive": True},
    }


def test_matches_list_directories_only_shorthand(client, auth_headers, windows_profile):
    response = client.post(
        "/v1/intents/interpret",
        headers=auth_headers,
        json={
            "message": "list folders only",
            "execution_profile": windows_profile,
        },
    )

    assert response.status_code == 200
    assert response.json()["intent"] == {
        "schema_version": 1,
        "action": "list_directories",
        "parameters": {"root": "."},
    }


def test_matches_singular_list_folder_only_shorthand(
    client, auth_headers, windows_profile
):
    response = client.post(
        "/v1/intents/interpret",
        headers=auth_headers,
        json={
            "message": "list folder only",
            "execution_profile": windows_profile,
        },
    )

    assert response.status_code == 200
    assert response.json()["intent"] == {
        "schema_version": 1,
        "action": "list_directories",
        "parameters": {"root": "."},
    }


def test_matches_inspect_file(client, auth_headers, windows_profile):
    response = client.post(
        "/v1/intents/interpret",
        headers=auth_headers,
        json={
            "message": "Show file info for package.json.",
            "execution_profile": windows_profile,
        },
    )

    assert response.status_code == 200
    assert response.json()["intent"] == {
        "schema_version": 1,
        "action": "inspect_file",
        "parameters": {"path": "package.json"},
    }


def test_matches_recursive_file_count(client, auth_headers, windows_profile):
    response = client.post(
        "/v1/intents/interpret",
        headers=auth_headers,
        json={
            "message": "Count all files in this folder recursively.",
            "execution_profile": windows_profile,
        },
    )

    assert response.status_code == 200
    assert response.json()["intent"] == {
        "schema_version": 1,
        "action": "count_files",
        "parameters": {"root": ".", "recursive": True},
    }


def test_matches_list_processes(client, auth_headers, windows_profile):
    response = client.post(
        "/v1/intents/interpret",
        headers=auth_headers,
        json={
            "message": "Show me the running processes.",
            "execution_profile": windows_profile,
        },
    )

    assert response.status_code == 200
    assert response.json()["intent"] == {
        "schema_version": 1,
        "action": "list_processes",
        "parameters": {},
    }


def test_matches_list_listening_ports(client, auth_headers, windows_profile):
    response = client.post(
        "/v1/intents/interpret",
        headers=auth_headers,
        json={
            "message": "Show listening ports.",
            "execution_profile": windows_profile,
        },
    )

    assert response.status_code == 200
    assert response.json()["intent"] == {
        "schema_version": 1,
        "action": "list_listening_ports",
        "parameters": {},
    }


def test_matches_inspect_port(client, auth_headers, windows_profile):
    response = client.post(
        "/v1/intents/interpret",
        headers=auth_headers,
        json={
            "message": "What is using port 3000?",
            "execution_profile": windows_profile,
        },
    )

    assert response.status_code == 200
    assert response.json()["intent"] == {
        "schema_version": 1,
        "action": "inspect_port",
        "parameters": {"port": 3000},
    }


def test_matches_direct_inspect_port_phrase(client, auth_headers, windows_profile):
    response = client.post(
        "/v1/intents/interpret",
        headers=auth_headers,
        json={
            "message": "Inspect port 3000.",
            "execution_profile": windows_profile,
        },
    )

    assert response.status_code == 200
    assert response.json()["intent"] == {
        "schema_version": 1,
        "action": "inspect_port",
        "parameters": {"port": 3000},
    }


def test_matches_inspect_a_port_phrase(client, auth_headers, windows_profile):
    response = client.post(
        "/v1/intents/interpret",
        headers=auth_headers,
        json={
            "message": "Inspect a port 8000.",
            "execution_profile": windows_profile,
        },
    )

    assert response.status_code == 200
    assert response.json()["intent"] == {
        "schema_version": 1,
        "action": "inspect_port",
        "parameters": {"port": 8000},
    }


@pytest.mark.parametrize(
    "message",
    [
        "Find port 8100.",
        "Locate port 8100.",
        "Show port 8100.",
        "Find process using port 8100.",
        "Find the process using port 8100.",
        "Which process uses port 8100?",
        "Which process owns port 8100?",
    ],
)
def test_matches_related_find_port_phrases(
    client, auth_headers, windows_profile, message
):
    response = client.post(
        "/v1/intents/interpret",
        headers=auth_headers,
        json={"message": message, "execution_profile": windows_profile},
    )

    assert response.status_code == 200
    assert response.json()["intent"] == {
        "schema_version": 1,
        "action": "inspect_port",
        "parameters": {"port": 8100},
    }


def test_requests_a_port_for_incomplete_inspection(
    client, auth_headers, windows_profile
):
    response = client.post(
        "/v1/intents/interpret",
        headers=auth_headers,
        json={
            "message": "Inspect a specific port.",
            "execution_profile": windows_profile,
        },
    )

    assert response.status_code == 200
    payload = response.json()
    assert payload["matched"] is False
    assert payload["requires_clarification"] is True
    assert "Which port" in payload["message"]


def test_returns_unsupported_for_unknown_request(client, auth_headers, windows_profile):
    response = client.post(
        "/v1/intents/interpret",
        headers=auth_headers,
        json={
            "message": "Deploy everything everywhere.",
            "execution_profile": windows_profile,
        },
    )

    assert response.status_code == 200
    payload = response.json()
    assert payload["matched"] is False
    assert payload["intent"] is None
