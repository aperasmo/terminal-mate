from __future__ import annotations

from dataclasses import dataclass
from typing import Any

from app.schemas.intents import CommandIntent, IntentClarification


@dataclass(frozen=True)
class ActionSpec:
    description: str
    required: dict[str, type]
    optional: dict[str, type]
    request_example: str | None


def _spec(
    description: str,
    required: dict[str, type] | None = None,
    optional: dict[str, type] | None = None,
    request_example: str | None = None,
) -> ActionSpec:
    return ActionSpec(description, required or {}, optional or {}, request_example)


# This is the AI boundary, not merely prompt documentation. An AI response is
# rejected unless its action and every parameter conform to this catalog.
ACTION_CATALOG: dict[str, ActionSpec] = {
    "show_current_directory": _spec("Show the current working directory."),
    "change_directory": _spec("Change directory.", {"target": str}),
    "find_files": _spec(
        "Find files by wildcard pattern.",
        {"pattern": str},
        {"root": str, "recursive": bool},
    ),
    "view_file_lines": _spec(
        "Show an inclusive line range from a file.",
        {"path": str, "start_line": int, "end_line": int},
    ),
    "view_file_start": _spec("Show the first lines of a file.", {"path": str, "line_count": int}),
    "view_file_end": _spec("Show the last lines of a file.", {"path": str, "line_count": int}),
    "view_file_edges": _spec("Show the first and last lines of a file.", {"path": str, "line_count": int}),
    "read_file": _spec("Read a file.", {"path": str}),
    "list_directory": _spec("List items in a directory.", optional={"root": str}),
    "list_hidden_items_with_details": _spec("List hidden items with names and modes.", optional={"root": str}),
    "list_directories": _spec("List folders only.", optional={"root": str}),
    "make_directory": _spec("Create one directory.", {"path": str}),
    "make_directories": _spec("Create multiple directories.", {"paths": list}),
    "create_file": _spec("Create one empty file.", {"path": str}),
    "create_files": _spec("Create multiple empty files.", {"paths": list}),
    "delete_file": _spec("Delete one file, never a folder.", {"path": str}),
    "copy_file": _spec("Copy a file.", {"source": str, "destination": str}),
    "move_file": _spec("Move a file.", {"source": str, "destination": str}),
    "backup_file": _spec("Create a backup copy of a file.", {"source": str, "backup": str}),
    "replace_file": _spec("Replace a file with another file.", {"source": str, "target": str}),
    "backup_and_replace_file": _spec(
        "Back up a destination file, then replace it with a source file.",
        {"source": str, "target": str, "backup": str},
    ),
    "search_file_contents": _spec("Search file contents for text.", {"term": str}, {"root": str}),
    "inspect_file": _spec("Show file metadata.", {"path": str}),
    "inspect_path": _spec("Identify a path as a file or folder and inspect it.", {"path": str}),
    "count_files": _spec("Count files.", optional={"root": str, "recursive": bool}),
    "list_processes": _spec("List running processes."),
    "list_listening_ports": _spec("List listening TCP ports."),
    "inspect_port": _spec("Inspect a TCP port.", {"port": int}),
    "run_script": _spec("Run one script.", {"path": str}, {"arguments": list}),
    "list_scripts": _spec(
        "List runnable scripts.",
        optional={"root": str, "recursive": bool, "order": str, "extension": str},
    ),
    "run_scripts": _spec(
        "Run scripts sequentially.",
        optional={"root": str, "recursive": bool, "order": str, "extension": str},
    ),
    "azure_cli_version": _spec("Show Azure CLI version."),
    "terraform_version": _spec("Show Terraform version."),
    "azure_account_show": _spec("Show the active Azure account."),
    "azure_subscription_list": _spec("List Azure subscriptions."),
    "azure_login": _spec("Sign in to Azure."),
    "azure_login_device_code": _spec("Sign in to Azure with a device code."),
    "azure_account_clear": _spec("Clear the Azure CLI account cache."),
    "azure_subscription_set": _spec("Select an Azure subscription.", {"subscription": str}),
    "azure_provider_show": _spec("Show Azure provider registration.", {"namespace": str}),
    "azure_provider_register": _spec("Register an Azure provider.", {"namespace": str}),
    "generate_ssh_key": _spec("Generate an SSH key.", {"key_name": str, "comment": str}),
    "show_ssh_public_key": _spec("Show an SSH public key.", {"key_name": str}),
    "azure_resource_group_create": _spec(
        "Create an Azure resource group.", {"resource_group": str, "location": str}
    ),
    "azure_storage_account_create": _spec(
        "Create an Azure storage account.", {"account_name": str, "resource_group": str}
    ),
    "azure_storage_container_create": _spec(
        "Create an Azure storage container.", {"container_name": str, "account_name": str}
    ),
    "azure_storage_keys_list": _spec(
        "List Azure storage account keys.", {"account_name": str, "resource_group": str}
    ),
    "terraform_init": _spec("Initialize Terraform."),
    "terraform_fmt": _spec("Format Terraform files."),
    "terraform_validate": _spec("Validate Terraform configuration."),
    "terraform_plan": _spec("Create a Terraform plan."),
    "terraform_apply": _spec("Apply Terraform configuration."),
    "terraform_destroy": _spec("Destroy Terraform-managed infrastructure."),
    "terraform_state_list": _spec("List Terraform state."),
    "terraform_state_show": _spec("Show a Terraform state address.", {"address": str}),
    "terraform_output": _spec("Show Terraform outputs."),
    "terraform_import": _spec(
        "Import a resource into Terraform state.", {"address": str, "resource_id": str}
    ),
    "azure_vm_list": _spec("List Azure virtual machines."),
    "azure_vm_show": _spec("Show an Azure VM.", {"vm_name": str, "resource_group": str}),
    "azure_nsg_list": _spec("List Azure network security groups."),
    "azure_resource_list": _spec("List Azure resources.", {"resource_group": str}),
    "azure_vm_start": _spec("Start an Azure VM.", {"vm_name": str, "resource_group": str}),
    "azure_vm_stop": _spec("Stop an Azure VM.", {"vm_name": str, "resource_group": str}),
    "azure_vm_deallocate": _spec("Deallocate an Azure VM.", {"vm_name": str, "resource_group": str}),
    "ssh_vm": _spec("Connect to a VM over SSH.", {"admin": str, "ip": str, "key_name": str}),
    "azure_resource_group_delete": _spec("Delete an Azure resource group.", {"resource_group": str}),
    "aws_identity_show": _spec("Show the active AWS caller identity."),
    "aws_budget_create": _spec(
        "Create an AWS cost budget. Never guess the account ID, budget name, or amount.",
        {"account_id": str, "budget_name": str, "amount": str},
        {"currency": str, "time_unit": str},
        "Create an AWS monthly budget named <name> for <amount> USD in account <12-digit-account-id>.",
    ),
    "aws_budget_alert_create": _spec(
        "Create an AWS cost budget with an email notification. Never guess the account ID, budget name, amount, threshold, or recipient.",
        {
            "account_id": str,
            "budget_name": str,
            "amount": str,
            "threshold": str,
            "notification_email": str,
        },
        {
            "currency": str,
            "time_unit": str,
            "notification_type": str,
            "threshold_type": str,
            "comparison_operator": str,
        },
        "Create an AWS monthly budget alert named <name> for <amount> USD in account <12-digit-account-id>, notifying <email> when ACTUAL spend exceeds <threshold> percent.",
    ),
    "aws_ec2_instance_list": _spec(
        "List AWS EC2 instances.", optional={"region": str}
    ),
    "aws_ec2_instance_create": _spec(
        "Create an AWS EC2 instance. Never guess image, networking, or key parameters.",
        {
            "image_id": str,
            "instance_type": str,
            "key_name": str,
            "security_group_ids": list,
            "subnet_id": str,
        },
        {"region": str, "count": int, "name": str},
        "Create 1 AWS EC2 instance using image <ami-id>, type <instance-type>, key <key-name>, security groups <sg-id>, and subnet <subnet-id> in <region>.",
    ),
    "azure_budget_create": _spec(
        "Create an Azure cost budget. Never guess dates, amount, or scope.",
        {"budget_name": str, "amount": str, "start_date": str, "end_date": str},
        {"resource_group": str, "time_grain": str},
        "Create an Azure monthly budget named <name> for <amount> from <YYYY-MM-DD> to <YYYY-MM-DD>, optionally in resource group <group>.",
    ),
    "azure_vm_create": _spec(
        "Create an Azure virtual machine.",
        {"vm_name": str, "resource_group": str, "image": str, "admin_username": str},
        {"location": str, "size": str, "generate_ssh_keys": bool},
        "Create Azure VM <name> in resource group <group> using image <image> and admin <username>.",
    ),
    "gcp_identity_show": _spec("Show the active Google Cloud account and project."),
    "gcp_budget_create": _spec(
        "Create a Google Cloud billing budget. Never guess the billing account or amount.",
        {"billing_account": str, "display_name": str, "amount": str},
        {"currency": str},
        "Create a Google Cloud budget named <name> for <amount> USD on billing account <billing-account-id>.",
    ),
    "gcp_compute_instance_list": _spec(
        "List Google Compute Engine instances.", optional={"project": str, "zone": str}
    ),
    "gcp_compute_instance_create": _spec(
        "Create a Google Compute Engine instance.",
        {"instance_name": str, "zone": str, "machine_type": str},
        {"project": str, "image_family": str, "image_project": str},
        "Create Google Compute Engine instance <name> in zone <zone> using machine type <type>, optionally in project <project>.",
    ),
    "ssh_connect": _spec(
        "Connect to a remote host over SSH with a non-interactive command.",
        {"host": str, "remote_command": str},
        {"user": str, "identity_file": str, "port": int},
        "Connect to <user>@<host> over SSH and run <remote-command>, optionally using key <path> and port <port>.",
    ),
}


def prompt_catalog() -> list[dict[str, Any]]:
    return [
        {
            "action": action,
            "description": spec.description,
            "required_parameters": {name: kind.__name__ for name, kind in spec.required.items()},
            "optional_parameters": {name: kind.__name__ for name, kind in spec.optional.items()},
            "request_example": spec.request_example,
        }
        for action, spec in ACTION_CATALOG.items()
    ]


def validate_intent(intent: CommandIntent) -> CommandIntent:
    spec = ACTION_CATALOG.get(intent.action)
    if spec is None:
        raise ValueError(f"Unsupported AI action: {intent.action}")

    allowed = set(spec.required) | set(spec.optional)
    unexpected = set(intent.parameters) - allowed
    missing = set(spec.required) - set(intent.parameters)
    if unexpected:
        raise ValueError(f"Unexpected parameters for {intent.action}: {sorted(unexpected)}")
    if missing:
        raise ValueError(f"Missing parameters for {intent.action}: {sorted(missing)}")

    for name, value in intent.parameters.items():
        expected = spec.required.get(name) or spec.optional[name]
        if expected is int and isinstance(value, bool):
            raise ValueError(f"Parameter {name} must be an integer.")
        if not isinstance(value, expected):
            raise ValueError(f"Parameter {name} must be {expected.__name__}.")
        if isinstance(value, str) and (not value.strip() or len(value) > 2_000):
            raise ValueError(f"Parameter {name} must contain 1 to 2000 characters.")
        if isinstance(value, list):
            if not value or len(value) > 100 or not all(isinstance(item, str) and item.strip() for item in value):
                raise ValueError(f"Parameter {name} must be a non-empty string list.")

    if intent.action == "inspect_port" and not 1 <= intent.parameters["port"] <= 65_535:
        raise ValueError("Port must be between 1 and 65535.")
    _validate_action_values(intent.action, intent.parameters)
    return intent


def validate_clarification(clarification: IntentClarification) -> IntentClarification:
    spec = ACTION_CATALOG.get(clarification.action)
    if spec is None:
        raise ValueError(f"Unsupported AI action: {clarification.action}")

    allowed = set(spec.required) | set(spec.optional)
    unexpected = set(clarification.parameters) - allowed
    if unexpected:
        raise ValueError(
            f"Unexpected parameters for {clarification.action}: {sorted(unexpected)}"
        )

    expected_missing = set(spec.required) - set(clarification.parameters)
    if not expected_missing or set(clarification.missing_parameters) != expected_missing:
        raise ValueError("AI clarification did not identify the exact missing required parameters.")

    partial = CommandIntent(
        schema_version=1,
        action=clarification.action,
        parameters=clarification.parameters,
    )
    for name, value in partial.parameters.items():
        expected = spec.required.get(name) or spec.optional[name]
        if expected is int and isinstance(value, bool):
            raise ValueError(f"Parameter {name} must be an integer.")
        if not isinstance(value, expected):
            raise ValueError(f"Parameter {name} must be {expected.__name__}.")
    _validate_action_values(clarification.action, clarification.parameters)
    return clarification


def clarification_message(clarification: IntentClarification) -> str:
    spec = ACTION_CATALOG[clarification.action]
    labels = ", ".join(name.replace("_", " ") for name in clarification.missing_parameters)
    message = f"{clarification.question.strip()} Missing: {labels}."
    if spec.request_example:
        message += f" Try: {spec.request_example}"
    return message


def _validate_action_values(action: str, parameters: dict[str, Any]) -> None:
    if "port" in parameters and not 1 <= parameters["port"] <= 65_535:
        raise ValueError("Port must be between 1 and 65535.")
    if action in {"aws_budget_create", "aws_budget_alert_create"}:
        account_id = parameters.get("account_id")
        if account_id is not None and (not account_id.isdigit() or len(account_id) != 12):
            raise ValueError("AWS account ID must contain exactly 12 digits.")
        time_unit = parameters.get("time_unit", "MONTHLY").upper()
        if time_unit not in {"DAILY", "MONTHLY", "QUARTERLY", "ANNUALLY"}:
            raise ValueError("AWS budget time unit is not supported.")
    if action == "aws_budget_alert_create":
        notification_type = parameters.get("notification_type", "ACTUAL").upper()
        if notification_type not in {"ACTUAL", "FORECASTED"}:
            raise ValueError("AWS notification type must be ACTUAL or FORECASTED.")
        threshold_type = parameters.get("threshold_type", "PERCENTAGE").upper()
        if threshold_type not in {"PERCENTAGE", "ABSOLUTE_VALUE"}:
            raise ValueError("AWS threshold type must be PERCENTAGE or ABSOLUTE_VALUE.")
        comparison = parameters.get("comparison_operator", "GREATER_THAN").upper()
        if comparison not in {"GREATER_THAN", "LESS_THAN", "EQUAL_TO"}:
            raise ValueError("AWS comparison operator is not supported.")
        email = parameters.get("notification_email")
        if email is not None and ("@" not in email or email.startswith("@") or email.endswith("@")):
            raise ValueError("AWS budget notification email is invalid.")
        threshold = parameters.get("threshold")
        if threshold is not None:
            try:
                if float(threshold) <= 0:
                    raise ValueError
            except (TypeError, ValueError) as error:
                raise ValueError("AWS budget alert threshold must be a positive number.") from error
    if action == "aws_ec2_instance_create" and "count" in parameters:
        if not 1 <= parameters["count"] <= 20:
            raise ValueError("EC2 instance count must be between 1 and 20.")
    if action in {
        "aws_budget_create",
        "aws_budget_alert_create",
        "azure_budget_create",
        "gcp_budget_create",
    }:
        amount = parameters.get("amount")
        if amount is not None:
            try:
                if float(amount) <= 0:
                    raise ValueError
            except (TypeError, ValueError) as error:
                raise ValueError("Budget amount must be a positive number.") from error
