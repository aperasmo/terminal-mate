# Generated from TERMINAL_MATE_COMMAND_CATALOG_PROMPT.md; edit the source catalog and regenerate.
# ruff: noqa: E501
from __future__ import annotations

COMMAND_CATALOG: list[dict[str, object]] = [
  {
    "id": "azure_show_azure_cli_version",
    "provider": "azure",
    "title": "Show Azure CLI version",
    "risk": "AzureSafe",
    "command": "az --version",
    "aliases": [
      "azure version",
      "check azure cli",
      "show az version",
      "is azure cli installed",
      "Show Azure CLI version"
    ],
    "required_parameters": []
  },
  {
    "id": "azure_show_current_azure_account",
    "provider": "azure",
    "title": "Show current Azure account",
    "risk": "AzureSafe",
    "command": "az account show --output table",
    "aliases": [
      "show azure account",
      "check azure account",
      "current subscription",
      "current azure subscription",
      "who am i in azure",
      "Show current Azure account"
    ],
    "required_parameters": []
  },
  {
    "id": "azure_show_current_azure_account_summary",
    "provider": "azure",
    "title": "Show current Azure account summary",
    "risk": "AzureSafe",
    "command": "az account show --query \"{Name:name,State:state,SubscriptionId:id,TenantId:tenantId}\" --output table",
    "aliases": [
      "azure account summary",
      "check subscription state",
      "show subscription id",
      "Show current Azure account summary"
    ],
    "required_parameters": []
  },
  {
    "id": "azure_list_azure_subscriptions",
    "provider": "azure",
    "title": "List Azure subscriptions",
    "risk": "AzureSafe",
    "command": "az account list --output table",
    "aliases": [
      "List Azure subscriptions"
    ],
    "required_parameters": []
  },
  {
    "id": "azure_sign_in_to_azure",
    "provider": "azure",
    "title": "Sign in to Azure",
    "risk": "AzureApproval",
    "command": "az login",
    "aliases": [
      "Sign in to Azure"
    ],
    "required_parameters": []
  },
  {
    "id": "azure_sign_in_to_azure_with_device_code",
    "provider": "azure",
    "title": "Sign in to Azure with device code",
    "risk": "AzureApproval",
    "command": "az login --use-device-code",
    "aliases": [
      "Sign in to Azure with device code"
    ],
    "required_parameters": []
  },
  {
    "id": "azure_switch_azure_subscription",
    "provider": "azure",
    "title": "Switch Azure subscription",
    "risk": "AzureApproval",
    "command": "az account set --subscription \"<subscription>\"",
    "aliases": [
      "switch subscription",
      "use subscription <subscription>",
      "set azure subscription <subscription>",
      "Switch Azure subscription"
    ],
    "required_parameters": [
      "subscription"
    ]
  },
  {
    "id": "azure_list_azure_locations",
    "provider": "azure",
    "title": "List Azure locations",
    "risk": "AzureSafe",
    "command": "az account list-locations --output table",
    "aliases": [
      "List Azure locations"
    ],
    "required_parameters": []
  },
  {
    "id": "azure_check_azure_location",
    "provider": "azure",
    "title": "Check Azure location",
    "risk": "AzureSafe",
    "command": "az account list-locations --query \"[?name=='<location>'].{Name:name,DisplayName:displayName}\" --output table",
    "aliases": [
      "check region <location>",
      "does azure have <location>",
      "show region <location>",
      "Check Azure location"
    ],
    "required_parameters": [
      "location"
    ]
  },
  {
    "id": "azure_show_azure_resource_groups",
    "provider": "azure",
    "title": "Show Azure resource groups",
    "risk": "AzureSafe",
    "command": "az group list --output table",
    "aliases": [
      "list resource groups",
      "show azure groups",
      "check resource groups",
      "show groups",
      "Show Azure resource groups"
    ],
    "required_parameters": []
  },
  {
    "id": "azure_check_whether_azure_resource_group_exists",
    "provider": "azure",
    "title": "Check whether Azure resource group exists",
    "risk": "AzureSafe",
    "command": "az group exists --name <group>",
    "aliases": [
      "check group <group>",
      "does group <group> exist",
      "verify resource group <group>",
      "Check whether Azure resource group exists"
    ],
    "required_parameters": [
      "group"
    ]
  },
  {
    "id": "azure_show_azure_resource_group",
    "provider": "azure",
    "title": "Show Azure resource group",
    "risk": "AzureSafe",
    "command": "az group show --name <group> --output table",
    "aliases": [
      "Show Azure resource group"
    ],
    "required_parameters": [
      "group"
    ]
  },
  {
    "id": "azure_create_azure_resource_group",
    "provider": "azure",
    "title": "Create Azure resource group",
    "risk": "AzureHigh Risk",
    "command": "az group create --name <group> --location <location> --query \"{Name:name,Location:location,ProvisioningState:properties.provisioningState}\" --output table",
    "aliases": [
      "Create Azure resource group"
    ],
    "required_parameters": [
      "group",
      "location"
    ]
  },
  {
    "id": "azure_delete_azure_resource_group",
    "provider": "azure",
    "title": "Delete Azure resource group",
    "risk": "AzureExplain Only",
    "command": "az group delete --name <group> --yes",
    "aliases": [
      "Delete Azure resource group"
    ],
    "required_parameters": [
      "group"
    ]
  },
  {
    "id": "azure_list_azure_resources",
    "provider": "azure",
    "title": "List Azure resources",
    "risk": "AzureSafe",
    "command": "az resource list --output table",
    "aliases": [
      "List Azure resources"
    ],
    "required_parameters": []
  },
  {
    "id": "azure_list_azure_resources_in_resource_group",
    "provider": "azure",
    "title": "List Azure resources in resource group",
    "risk": "AzureSafe",
    "command": "az resource list --resource-group <group> --output table",
    "aliases": [
      "List Azure resources in resource group"
    ],
    "required_parameters": [
      "group"
    ]
  },
  {
    "id": "azure_delete_arbitrary_azure_resource",
    "provider": "azure",
    "title": "Delete arbitrary Azure resource",
    "risk": "AzureExplain Only",
    "command": "az resource delete --ids \"<resource-id>\"",
    "aliases": [
      "Delete arbitrary Azure resource"
    ],
    "required_parameters": [
      "resource_id"
    ]
  },
  {
    "id": "azure_list_azure_vms",
    "provider": "azure",
    "title": "List Azure VMs",
    "risk": "AzureSafe",
    "command": "az vm list --show-details --output table",
    "aliases": [
      "List Azure VMs"
    ],
    "required_parameters": []
  },
  {
    "id": "azure_show_azure_vm",
    "provider": "azure",
    "title": "Show Azure VM",
    "risk": "AzureSafe",
    "command": "az vm show --name <vm> --resource-group <group> --show-details --output table",
    "aliases": [
      "Show Azure VM"
    ],
    "required_parameters": [
      "vm",
      "group"
    ]
  },
  {
    "id": "azure_show_azure_vm_power_state",
    "provider": "azure",
    "title": "Show Azure VM power state",
    "risk": "AzureSafe",
    "command": "az vm get-instance-view --name <vm> --resource-group <group> --query \"instanceView.statuses[?starts_with(code,'PowerState/')].displayStatus\" --output tsv",
    "aliases": [
      "vm status",
      "check vm state",
      "is vm running",
      "show vm power",
      "Show Azure VM power state"
    ],
    "required_parameters": [
      "vm",
      "group"
    ]
  },
  {
    "id": "azure_start_azure_vm",
    "provider": "azure",
    "title": "Start Azure VM",
    "risk": "AzureApproval",
    "command": "az vm start --name <vm> --resource-group <group>",
    "aliases": [
      "Start Azure VM"
    ],
    "required_parameters": [
      "vm",
      "group"
    ]
  },
  {
    "id": "azure_stop_azure_vm",
    "provider": "azure",
    "title": "Stop Azure VM",
    "risk": "AzureApproval",
    "command": "az vm stop --name <vm> --resource-group <group>",
    "aliases": [
      "Stop Azure VM"
    ],
    "required_parameters": [
      "vm",
      "group"
    ]
  },
  {
    "id": "azure_restart_azure_vm",
    "provider": "azure",
    "title": "Restart Azure VM",
    "risk": "AzureApproval",
    "command": "az vm restart --name <vm> --resource-group <group>",
    "aliases": [
      "Restart Azure VM"
    ],
    "required_parameters": [
      "vm",
      "group"
    ]
  },
  {
    "id": "azure_deallocate_azure_vm",
    "provider": "azure",
    "title": "Deallocate Azure VM",
    "risk": "AzureHigh Risk",
    "command": "az vm deallocate --name <vm> --resource-group <group>",
    "aliases": [
      "Deallocate Azure VM"
    ],
    "required_parameters": [
      "vm",
      "group"
    ]
  },
  {
    "id": "azure_create_azure_vm",
    "provider": "azure",
    "title": "Create Azure VM",
    "risk": "AzureHigh Risk",
    "command": "az vm create --name <name> --resource-group <group> --image <image> --admin-username <username>",
    "aliases": [
      "Create Azure VM"
    ],
    "required_parameters": [
      "name",
      "group",
      "image",
      "username"
    ]
  },
  {
    "id": "azure_list_azure_managed_disks",
    "provider": "azure",
    "title": "List Azure managed disks",
    "risk": "AzureSafe",
    "command": "az disk list --output table",
    "aliases": [
      "List Azure managed disks"
    ],
    "required_parameters": []
  },
  {
    "id": "azure_show_azure_disk",
    "provider": "azure",
    "title": "Show Azure disk",
    "risk": "AzureSafe",
    "command": "az disk show --name <disk> --resource-group <group> --output table",
    "aliases": [
      "Show Azure disk"
    ],
    "required_parameters": [
      "disk",
      "group"
    ]
  },
  {
    "id": "azure_list_azure_public_ip_addresses",
    "provider": "azure",
    "title": "List Azure public IP addresses",
    "risk": "AzureSafe",
    "command": "az network public-ip list --output table",
    "aliases": [
      "List Azure public IP addresses"
    ],
    "required_parameters": []
  },
  {
    "id": "azure_show_azure_public_ip",
    "provider": "azure",
    "title": "Show Azure public IP",
    "risk": "AzureSafe",
    "command": "az network public-ip show --name <name> --resource-group <group> --output table",
    "aliases": [
      "Show Azure public IP"
    ],
    "required_parameters": [
      "name",
      "group"
    ]
  },
  {
    "id": "azure_list_azure_network_security_groups",
    "provider": "azure",
    "title": "List Azure network security groups",
    "risk": "AzureSafe",
    "command": "az network nsg list --output table",
    "aliases": [
      "List Azure network security groups"
    ],
    "required_parameters": []
  },
  {
    "id": "azure_create_azure_storage_account",
    "provider": "azure",
    "title": "Create Azure storage account",
    "risk": "AzureHigh Risk",
    "command": "az storage account create --name <account> --resource-group <group> --location <location> --sku Standard_LRS",
    "aliases": [
      "Create Azure storage account"
    ],
    "required_parameters": [
      "account",
      "group",
      "location"
    ]
  },
  {
    "id": "azure_create_azure_storage_container",
    "provider": "azure",
    "title": "Create Azure storage container",
    "risk": "AzureHigh Risk",
    "command": "az storage container create --name <container> --account-name <account> --auth-mode login",
    "aliases": [
      "Create Azure storage container"
    ],
    "required_parameters": [
      "container",
      "account"
    ]
  },
  {
    "id": "azure_list_azure_storage_keys",
    "provider": "azure",
    "title": "List Azure storage keys",
    "risk": "AzureExplain Only",
    "command": "az storage account keys list --account-name <account> --resource-group <group>",
    "aliases": [
      "List Azure storage keys"
    ],
    "required_parameters": [
      "account",
      "group"
    ]
  },
  {
    "id": "azure_list_azure_static_web_apps",
    "provider": "azure",
    "title": "List Azure Static Web Apps",
    "risk": "AzureSafe",
    "command": "az staticwebapp list --output table",
    "aliases": [
      "list static web apps",
      "show static apps",
      "check azure frontend",
      "show swa",
      "List Azure Static Web Apps"
    ],
    "required_parameters": []
  },
  {
    "id": "azure_show_azure_static_web_app",
    "provider": "azure",
    "title": "Show Azure Static Web App",
    "risk": "AzureSafe",
    "command": "az staticwebapp show --name <name> --resource-group <group> --query \"{Name:name,DefaultHostname:defaultHostname,Sku:sku.name,RepositoryUrl:repositoryUrl,Branch:branch}\" --output table",
    "aliases": [
      "Show Azure Static Web App"
    ],
    "required_parameters": [
      "name",
      "group"
    ]
  },
  {
    "id": "azure_list_azure_static_web_app_hostnames",
    "provider": "azure",
    "title": "List Azure Static Web App hostnames",
    "risk": "AzureSafe",
    "command": "az staticwebapp hostname list --name <name> --resource-group <group> --query \"[].{Domain:domainName,Status:status}\" --output table",
    "aliases": [
      "check static app domains",
      "show azure hostnames",
      "check custom domain",
      "show swa domains",
      "List Azure Static Web App hostnames"
    ],
    "required_parameters": [
      "name",
      "group"
    ]
  },
  {
    "id": "azure_add_custom_hostname_to_static_web_app",
    "provider": "azure",
    "title": "Add custom hostname to Static Web App",
    "risk": "AzureHigh Risk",
    "command": "az staticwebapp hostname set --name <name> --resource-group <group> --hostname <hostname> --validation-method cname-delegation --output table",
    "aliases": [
      "Add custom hostname to Static Web App"
    ],
    "required_parameters": [
      "name",
      "group",
      "hostname"
    ]
  },
  {
    "id": "azure_list_static_web_app_environments",
    "provider": "azure",
    "title": "List Static Web App environments",
    "risk": "AzureSafe",
    "command": "az staticwebapp environment list --name <name> --resource-group <group> --query \"[].{Name:name,Hostname:hostname,SourceBranch:sourceBranch,Status:status}\" --output table",
    "aliases": [
      "List Static Web App environments"
    ],
    "required_parameters": [
      "name",
      "group"
    ]
  },
  {
    "id": "azure_show_static_web_app_environment",
    "provider": "azure",
    "title": "Show Static Web App environment",
    "risk": "AzureSafe",
    "command": "az staticwebapp environment show --name <name> --resource-group <group> --environment-name <environment> --output json",
    "aliases": [
      "Show Static Web App environment"
    ],
    "required_parameters": [
      "name",
      "group",
      "environment"
    ]
  },
  {
    "id": "azure_create_azure_static_web_app",
    "provider": "azure",
    "title": "Create Azure Static Web App",
    "risk": "AzureHigh Risk",
    "command": "az staticwebapp create --name <name> --resource-group <group> --location <location> --sku Free --output table",
    "aliases": [
      "Create Azure Static Web App"
    ],
    "required_parameters": [
      "name",
      "group",
      "location"
    ]
  },
  {
    "id": "azure_list_azure_consumption_usage",
    "provider": "azure",
    "title": "List Azure consumption usage",
    "risk": "AzureSafe",
    "command": "az consumption usage list --start-date <start-date> --end-date <end-date>",
    "aliases": [
      "check consumption list",
      "check consumption usage",
      "show azure usage",
      "show consumption",
      "show billed resources",
      "list azure usage",
      "List Azure consumption usage"
    ],
    "required_parameters": [
      "start_date",
      "end_date"
    ]
  },
  {
    "id": "azure_show_azure_consumption_details_by_resource",
    "provider": "azure",
    "title": "Show Azure consumption details by resource",
    "risk": "AzureSafe",
    "command": "az consumption usage list --start-date <start-date> --end-date <end-date> --query \"[].{Resource:instanceName,ResourceGroup:resourceGroup,Product:product,Cost:pretaxCost,Currency:currency}\" --output table",
    "aliases": [
      "check consumption details",
      "show usage by resource",
      "show billed resource details",
      "check azure charges",
      "Show Azure consumption details by resource"
    ],
    "required_parameters": [
      "start_date",
      "end_date"
    ]
  },
  {
    "id": "azure_show_current_month_azure_consumption",
    "provider": "azure",
    "title": "Show current-month Azure consumption",
    "risk": "AzureSafe",
    "command": "az consumption usage list --start-date <YYYY-MM-01> --end-date <YYYY-MM-DD> --query \"[].{Resource:instanceName,ResourceGroup:resourceGroup,Product:product,Cost:pretaxCost,Currency:currency}\" --output table",
    "aliases": [
      "check azure cost",
      "check this month cost",
      "show current azure usage",
      "monthly azure usage",
      "current azure spend",
      "Show current-month Azure consumption"
    ],
    "required_parameters": []
  },
  {
    "id": "azure_show_azure_budget_list",
    "provider": "azure",
    "title": "Show Azure budget list",
    "risk": "AzureSafe",
    "command": "az consumption budget list --output table",
    "aliases": [
      "Show Azure budget list"
    ],
    "required_parameters": []
  },
  {
    "id": "azure_show_azure_budget",
    "provider": "azure",
    "title": "Show Azure budget",
    "risk": "AzureSafe",
    "command": "az consumption budget show --budget-name <name> --output json",
    "aliases": [
      "Show Azure budget"
    ],
    "required_parameters": [
      "name"
    ]
  },
  {
    "id": "aws_show_aws_cli_version",
    "provider": "aws",
    "title": "Show AWS CLI version",
    "risk": "AWSSafe",
    "command": "aws --version",
    "aliases": [
      "Show AWS CLI version"
    ],
    "required_parameters": []
  },
  {
    "id": "aws_show_current_aws_identity",
    "provider": "aws",
    "title": "Show current AWS identity",
    "risk": "AWSSafe",
    "command": "aws sts get-caller-identity",
    "aliases": [
      "Show current AWS identity"
    ],
    "required_parameters": []
  },
  {
    "id": "aws_show_aws_configured_region",
    "provider": "aws",
    "title": "Show AWS configured region",
    "risk": "AWSSafe",
    "command": "aws configure get region",
    "aliases": [
      "Show AWS configured region"
    ],
    "required_parameters": []
  },
  {
    "id": "aws_list_lambda_functions",
    "provider": "aws",
    "title": "List Lambda functions",
    "risk": "AWSSafe",
    "command": "aws lambda list-functions --region <region> --query \"Functions[].{Name:FunctionName,Runtime:Runtime,Memory:MemorySize,Timeout:Timeout}\" --output table",
    "aliases": [
      "List Lambda functions"
    ],
    "required_parameters": [
      "region"
    ]
  },
  {
    "id": "aws_show_lambda_function",
    "provider": "aws",
    "title": "Show Lambda function",
    "risk": "AWSSafe",
    "command": "aws lambda get-function-configuration --function-name <function> --region <region> --query \"{FunctionName:FunctionName,State:State,LastUpdateStatus:LastUpdateStatus,MemorySize:MemorySize,Timeout:Timeout,Architecture:Architectures}\" --output json",
    "aliases": [
      "Show Lambda function"
    ],
    "required_parameters": [
      "function",
      "region"
    ]
  },
  {
    "id": "aws_wait_for_lambda_update",
    "provider": "aws",
    "title": "Wait for Lambda update",
    "risk": "AWSSafe",
    "command": "aws lambda wait function-updated-v2 --function-name <function> --region <region>",
    "aliases": [
      "Wait for Lambda update"
    ],
    "required_parameters": [
      "function",
      "region"
    ]
  },
  {
    "id": "aws_show_lambda_concurrency",
    "provider": "aws",
    "title": "Show Lambda concurrency",
    "risk": "AWSSafe",
    "command": "aws lambda get-function-concurrency --function-name <function> --region <region>",
    "aliases": [
      "Show Lambda concurrency"
    ],
    "required_parameters": [
      "function",
      "region"
    ]
  },
  {
    "id": "aws_set_lambda_reserved_concurrency",
    "provider": "aws",
    "title": "Set Lambda reserved concurrency",
    "risk": "AWSHigh Risk",
    "command": "aws lambda put-function-concurrency --function-name <function> --reserved-concurrent-executions <count> --region <region>",
    "aliases": [
      "Set Lambda reserved concurrency"
    ],
    "required_parameters": [
      "function",
      "count",
      "region"
    ]
  },
  {
    "id": "aws_show_lambda_concurrent_executions_quota",
    "provider": "aws",
    "title": "Show Lambda concurrent executions quota",
    "risk": "AWSSafe",
    "command": "aws service-quotas get-service-quota --service-code lambda --quota-code L-B99A9384 --region <region>",
    "aliases": [
      "Show Lambda concurrent executions quota"
    ],
    "required_parameters": [
      "region"
    ]
  },
  {
    "id": "aws_check_lambda_quota_increase_request",
    "provider": "aws",
    "title": "Check Lambda quota increase request",
    "risk": "AWSSafe",
    "command": "aws service-quotas list-requested-service-quota-change-history-by-quota --service-code lambda --quota-code L-B99A9384 --region <region> --query \"RequestedQuotas[0].{Status:Status,DesiredValue:DesiredValue,Created:Created}\" --output table",
    "aliases": [
      "check lambda quota request",
      "quota approval status",
      "check concurrency request",
      "lambda quota status",
      "Check Lambda quota increase request"
    ],
    "required_parameters": [
      "region"
    ]
  },
  {
    "id": "aws_list_ecr_repositories",
    "provider": "aws",
    "title": "List ECR repositories",
    "risk": "AWSSafe",
    "command": "aws ecr describe-repositories --region <region> --query \"repositories[].{Name:repositoryName,Uri:repositoryUri,ScanOnPush:imageScanningConfiguration.scanOnPush}\" --output table",
    "aliases": [
      "List ECR repositories"
    ],
    "required_parameters": [
      "region"
    ]
  },
  {
    "id": "aws_list_ecr_images",
    "provider": "aws",
    "title": "List ECR images",
    "risk": "AWSSafe",
    "command": "aws ecr describe-images --repository-name <repo> --region <region> --query \"imageDetails[].{Tags:imageTags,Digest:imageDigest,Pushed:imagePushedAt,Size:imageSizeInBytes}\" --output table",
    "aliases": [
      "List ECR images"
    ],
    "required_parameters": [
      "repo",
      "region"
    ]
  },
  {
    "id": "aws_show_ecr_image_scan_findings",
    "provider": "aws",
    "title": "Show ECR image scan findings",
    "risk": "AWSSafe",
    "command": "aws ecr describe-image-scan-findings --repository-name <repo> --image-id imageTag=<tag> --region <region> --query \"imageScanFindings.findingSeverityCounts\" --output table",
    "aliases": [
      "Show ECR image scan findings"
    ],
    "required_parameters": [
      "repo",
      "tag",
      "region"
    ]
  },
  {
    "id": "aws_authenticate_docker_to_ecr",
    "provider": "aws",
    "title": "Authenticate Docker to ECR",
    "risk": "AWSApproval",
    "command": "aws ecr get-login-password --region <region> | docker login --username AWS --password-stdin <registry>",
    "aliases": [
      "Authenticate Docker to ECR"
    ],
    "required_parameters": [
      "region",
      "registry"
    ]
  },
  {
    "id": "aws_list_http_apis",
    "provider": "aws",
    "title": "List HTTP APIs",
    "risk": "AWSSafe",
    "command": "aws apigatewayv2 get-apis --region <region> --query \"Items[].{Name:Name,ApiId:ApiId,Endpoint:ApiEndpoint,ProtocolType:ProtocolType}\" --output table",
    "aliases": [
      "List HTTP APIs"
    ],
    "required_parameters": [
      "region"
    ]
  },
  {
    "id": "aws_list_api_routes",
    "provider": "aws",
    "title": "List API routes",
    "risk": "AWSSafe",
    "command": "aws apigatewayv2 get-routes --api-id <api-id> --region <region> --query \"Items[].{RouteKey:RouteKey,Target:Target}\" --output table",
    "aliases": [
      "List API routes"
    ],
    "required_parameters": [
      "api_id",
      "region"
    ]
  },
  {
    "id": "aws_list_api_stages",
    "provider": "aws",
    "title": "List API stages",
    "risk": "AWSSafe",
    "command": "aws apigatewayv2 get-stages --api-id <api-id> --region <region> --output table",
    "aliases": [
      "List API stages"
    ],
    "required_parameters": [
      "api_id",
      "region"
    ]
  },
  {
    "id": "aws_list_cloudwatch_log_groups",
    "provider": "aws",
    "title": "List CloudWatch log groups",
    "risk": "AWSSafe",
    "command": "aws logs describe-log-groups --region <region> --query \"logGroups[].{Name:logGroupName,StoredBytes:storedBytes,Retention:retentionInDays}\" --output table",
    "aliases": [
      "List CloudWatch log groups"
    ],
    "required_parameters": [
      "region"
    ]
  },
  {
    "id": "aws_tail_lambda_logs",
    "provider": "aws",
    "title": "Tail Lambda logs",
    "risk": "AWSSafe",
    "command": "aws logs tail \"/aws/lambda/<function>\" --region <region> --since 10m",
    "aliases": [
      "tail lambda logs",
      "show recent lambda logs",
      "check backend logs",
      "Tail Lambda logs"
    ],
    "required_parameters": [
      "function",
      "region"
    ]
  },
  {
    "id": "aws_list_aws_budgets",
    "provider": "aws",
    "title": "List AWS budgets",
    "risk": "AWSSafe",
    "command": "aws budgets describe-budgets --account-id <account-id> --query \"Budgets[].{Name:BudgetName,Limit:BudgetLimit.Amount,Unit:BudgetLimit.Unit,TimeUnit:TimeUnit,Type:BudgetType}\" --output table",
    "aliases": [
      "List AWS budgets"
    ],
    "required_parameters": [
      "account_id"
    ]
  },
  {
    "id": "aws_show_aws_budget",
    "provider": "aws",
    "title": "Show AWS budget",
    "risk": "AWSSafe",
    "command": "aws budgets describe-budget --account-id <account-id> --budget-name <name> --query \"Budget.{Name:BudgetName,Limit:BudgetLimit.Amount,Unit:BudgetLimit.Unit,TimeUnit:TimeUnit,Type:BudgetType}\" --output table",
    "aliases": [
      "Show AWS budget"
    ],
    "required_parameters": [
      "account_id",
      "name"
    ]
  },
  {
    "id": "aws_show_aws_budget_notifications",
    "provider": "aws",
    "title": "Show AWS budget notifications",
    "risk": "AWSSafe",
    "command": "aws budgets describe-notifications-for-budget --account-id <account-id> --budget-name <name> --query \"Notifications[].{Type:NotificationType,Threshold:Threshold,ThresholdType:ThresholdType,Comparison:ComparisonOperator}\" --output table",
    "aliases": [
      "Show AWS budget notifications"
    ],
    "required_parameters": [
      "account_id",
      "name"
    ]
  },
  {
    "id": "aws_create_aws_monthly_cost_budget",
    "provider": "aws",
    "title": "Create AWS monthly cost budget",
    "risk": "AWSHigh Risk",
    "command": "$f=Join-Path $env:TEMP \"aws-budget.json\"; $json=@{BudgetName=\"<name>\";BudgetLimit=@{Amount=\"<amount>\";Unit=\"USD\"};TimeUnit=\"MONTHLY\";BudgetType=\"COST\"} | ConvertTo-Json -Depth 4; [System.IO.File]::WriteAllText($f,$json,(New-Object System.Text.UTF8Encoding($False))); aws budgets create-budget --account-id <account-id> --budget \"file://$f\"; Remove-Item $f -Force",
    "aliases": [
      "Create AWS monthly cost budget"
    ],
    "required_parameters": [
      "name",
      "amount",
      "account_id"
    ]
  },
  {
    "id": "github_show_github_cli_version",
    "provider": "github",
    "title": "Show GitHub CLI version",
    "risk": "GitHubSafe",
    "command": "gh --version",
    "aliases": [
      "Show GitHub CLI version"
    ],
    "required_parameters": []
  },
  {
    "id": "github_show_github_authentication_status",
    "provider": "github",
    "title": "Show GitHub authentication status",
    "risk": "GitHubSafe",
    "command": "gh auth status",
    "aliases": [
      "Show GitHub authentication status"
    ],
    "required_parameters": []
  },
  {
    "id": "github_sign_in_to_github_cli",
    "provider": "github",
    "title": "Sign in to GitHub CLI",
    "risk": "GitHubApproval",
    "command": "gh auth login --hostname github.com --git-protocol https --web",
    "aliases": [
      "Sign in to GitHub CLI"
    ],
    "required_parameters": []
  },
  {
    "id": "github_show_repository",
    "provider": "github",
    "title": "Show repository",
    "risk": "GitHubSafe",
    "command": "gh repo view <owner/repo>",
    "aliases": [
      "Show repository"
    ],
    "required_parameters": [
      "owner_repo"
    ]
  },
  {
    "id": "github_show_repository_json_summary",
    "provider": "github",
    "title": "Show repository JSON summary",
    "risk": "GitHubSafe",
    "command": "gh repo view <owner/repo> --json nameWithOwner,defaultBranchRef",
    "aliases": [
      "Show repository JSON summary"
    ],
    "required_parameters": [
      "owner_repo"
    ]
  },
  {
    "id": "github_list_workflow_runs",
    "provider": "github",
    "title": "List workflow runs",
    "risk": "GitHubSafe",
    "command": "gh run list --repo <owner/repo> --workflow <workflow> --limit 5",
    "aliases": [
      "check actions",
      "show workflow runs",
      "list github actions",
      "check deployment workflow",
      "List workflow runs"
    ],
    "required_parameters": [
      "owner_repo",
      "workflow"
    ]
  },
  {
    "id": "github_watch_latest_workflow_run",
    "provider": "github",
    "title": "Watch latest workflow run",
    "risk": "GitHubSafe",
    "command": "$runId=(gh run list --repo <owner/repo> --workflow <workflow> --limit 1 --json databaseId | ConvertFrom-Json).databaseId; gh run watch $runId --repo <owner/repo> --exit-status",
    "aliases": [
      "Watch latest workflow run"
    ],
    "required_parameters": [
      "owner_repo",
      "workflow"
    ]
  },
  {
    "id": "github_trigger_workflow",
    "provider": "github",
    "title": "Trigger workflow",
    "risk": "GitHubApproval",
    "command": "gh workflow run <workflow> --repo <owner/repo> --ref <branch>",
    "aliases": [
      "Trigger workflow"
    ],
    "required_parameters": [
      "workflow",
      "owner_repo",
      "branch"
    ]
  },
  {
    "id": "github_list_github_actions_secret_names",
    "provider": "github",
    "title": "List GitHub Actions secret names",
    "risk": "GitHubSafe",
    "command": "gh secret list --repo <owner/repo>",
    "aliases": [
      "List GitHub Actions secret names"
    ],
    "required_parameters": [
      "owner_repo"
    ]
  },
  {
    "id": "github_set_github_actions_secret",
    "provider": "github",
    "title": "Set GitHub Actions secret",
    "risk": "GitHubHigh Risk",
    "command": "gh secret set <name> --repo <owner/repo>",
    "aliases": [
      "Set GitHub Actions secret"
    ],
    "required_parameters": [
      "name",
      "owner_repo"
    ]
  },
  {
    "id": "github_list_github_actions_variables",
    "provider": "github",
    "title": "List GitHub Actions variables",
    "risk": "GitHubSafe",
    "command": "gh variable list --repo <owner/repo>",
    "aliases": [
      "List GitHub Actions variables"
    ],
    "required_parameters": [
      "owner_repo"
    ]
  },
  {
    "id": "github_set_github_actions_variable",
    "provider": "github",
    "title": "Set GitHub Actions variable",
    "risk": "GitHubApproval",
    "command": "gh variable set <name> --body \"<value>\" --repo <owner/repo>",
    "aliases": [
      "Set GitHub Actions variable"
    ],
    "required_parameters": [
      "name",
      "value",
      "owner_repo"
    ]
  },
  {
    "id": "docker_show_docker_version",
    "provider": "docker",
    "title": "Show Docker version",
    "risk": "DockerSafe",
    "command": "docker version",
    "aliases": [
      "Show Docker version"
    ],
    "required_parameters": []
  },
  {
    "id": "docker_list_docker_images",
    "provider": "docker",
    "title": "List Docker images",
    "risk": "DockerSafe",
    "command": "docker images",
    "aliases": [
      "List Docker images"
    ],
    "required_parameters": []
  },
  {
    "id": "docker_list_running_docker_containers",
    "provider": "docker",
    "title": "List running Docker containers",
    "risk": "DockerSafe",
    "command": "docker ps",
    "aliases": [
      "List running Docker containers"
    ],
    "required_parameters": []
  },
  {
    "id": "docker_list_all_docker_containers",
    "provider": "docker",
    "title": "List all Docker containers",
    "risk": "DockerSafe",
    "command": "docker ps -a",
    "aliases": [
      "List all Docker containers"
    ],
    "required_parameters": []
  },
  {
    "id": "docker_show_docker_container_logs",
    "provider": "docker",
    "title": "Show Docker container logs",
    "risk": "DockerSafe",
    "command": "docker logs <container>",
    "aliases": [
      "Show Docker container logs"
    ],
    "required_parameters": [
      "container"
    ]
  },
  {
    "id": "docker_follow_docker_container_logs",
    "provider": "docker",
    "title": "Follow Docker container logs",
    "risk": "DockerSafe",
    "command": "docker logs -f <container>",
    "aliases": [
      "Follow Docker container logs"
    ],
    "required_parameters": [
      "container"
    ]
  },
  {
    "id": "docker_start_docker_container",
    "provider": "docker",
    "title": "Start Docker container",
    "risk": "DockerApproval",
    "command": "docker start <container>",
    "aliases": [
      "Start Docker container"
    ],
    "required_parameters": [
      "container"
    ]
  },
  {
    "id": "docker_stop_docker_container",
    "provider": "docker",
    "title": "Stop Docker container",
    "risk": "DockerApproval",
    "command": "docker stop <container>",
    "aliases": [
      "Stop Docker container"
    ],
    "required_parameters": [
      "container"
    ]
  },
  {
    "id": "docker_remove_docker_container",
    "provider": "docker",
    "title": "Remove Docker container",
    "risk": "DockerHigh Risk",
    "command": "docker rm <container>",
    "aliases": [
      "Remove Docker container"
    ],
    "required_parameters": [
      "container"
    ]
  },
  {
    "id": "docker_remove_docker_image",
    "provider": "docker",
    "title": "Remove Docker image",
    "risk": "DockerHigh Risk",
    "command": "docker rmi <image>",
    "aliases": [
      "Remove Docker image"
    ],
    "required_parameters": [
      "image"
    ]
  },
  {
    "id": "docker_build_docker_image",
    "provider": "docker",
    "title": "Build Docker image",
    "risk": "DockerApproval",
    "command": "docker build -f <dockerfile> -t <tag> .",
    "aliases": [
      "Build Docker image"
    ],
    "required_parameters": [
      "dockerfile",
      "tag"
    ]
  },
  {
    "id": "docker_build_lambda_compatible_docker_image",
    "provider": "docker",
    "title": "Build Lambda-compatible Docker image",
    "risk": "DockerApproval",
    "command": "docker build --platform linux/amd64 --provenance=False --sbom=False -f <dockerfile> -t <tag> .",
    "aliases": [
      "Build Lambda-compatible Docker image"
    ],
    "required_parameters": [
      "dockerfile",
      "tag"
    ]
  },
  {
    "id": "docker_show_docker_compose_services",
    "provider": "docker",
    "title": "Show Docker Compose services",
    "risk": "DockerSafe",
    "command": "docker compose ps",
    "aliases": [
      "Show Docker Compose services"
    ],
    "required_parameters": []
  },
  {
    "id": "docker_start_docker_compose_stack",
    "provider": "docker",
    "title": "Start Docker Compose stack",
    "risk": "DockerApproval",
    "command": "docker compose up -d",
    "aliases": [
      "Start Docker Compose stack"
    ],
    "required_parameters": []
  },
  {
    "id": "docker_stop_docker_compose_stack",
    "provider": "docker",
    "title": "Stop Docker Compose stack",
    "risk": "DockerApproval",
    "command": "docker compose down",
    "aliases": [
      "Stop Docker Compose stack"
    ],
    "required_parameters": []
  },
  {
    "id": "git_show_git_status",
    "provider": "git",
    "title": "Show Git status",
    "risk": "GitSafe",
    "command": "git status --short --branch",
    "aliases": [
      "git status",
      "check repo",
      "working tree status",
      "Show Git status"
    ],
    "required_parameters": []
  },
  {
    "id": "git_show_git_diff",
    "provider": "git",
    "title": "Show Git diff",
    "risk": "GitSafe",
    "command": "git diff",
    "aliases": [
      "Show Git diff"
    ],
    "required_parameters": []
  },
  {
    "id": "git_show_staged_git_diff",
    "provider": "git",
    "title": "Show staged Git diff",
    "risk": "GitSafe",
    "command": "git diff --cached",
    "aliases": [
      "Show staged Git diff"
    ],
    "required_parameters": []
  },
  {
    "id": "git_check_git_diff_for_whitespace_errors",
    "provider": "git",
    "title": "Check Git diff for whitespace errors",
    "risk": "GitSafe",
    "command": "git diff --check",
    "aliases": [
      "Check Git diff for whitespace errors"
    ],
    "required_parameters": []
  },
  {
    "id": "git_check_staged_git_diff_for_whitespace_errors",
    "provider": "git",
    "title": "Check staged Git diff for whitespace errors",
    "risk": "GitSafe",
    "command": "git diff --cached --check",
    "aliases": [
      "Check staged Git diff for whitespace errors"
    ],
    "required_parameters": []
  },
  {
    "id": "git_show_git_diff_statistics",
    "provider": "git",
    "title": "Show Git diff statistics",
    "risk": "GitSafe",
    "command": "git diff --stat",
    "aliases": [
      "Show Git diff statistics"
    ],
    "required_parameters": []
  },
  {
    "id": "git_show_latest_git_commit",
    "provider": "git",
    "title": "Show latest Git commit",
    "risk": "GitSafe",
    "command": "git log -1 --oneline",
    "aliases": [
      "Show latest Git commit"
    ],
    "required_parameters": []
  },
  {
    "id": "git_show_files_changed_in_latest_commit",
    "provider": "git",
    "title": "Show files changed in latest commit",
    "risk": "GitSafe",
    "command": "git show --stat --name-only --oneline HEAD",
    "aliases": [
      "Show files changed in latest commit"
    ],
    "required_parameters": []
  },
  {
    "id": "git_show_git_remotes",
    "provider": "git",
    "title": "Show Git remotes",
    "risk": "GitSafe",
    "command": "git remote -v",
    "aliases": [
      "Show Git remotes"
    ],
    "required_parameters": []
  },
  {
    "id": "git_compare_local_head_with_remote_main",
    "provider": "git",
    "title": "Compare local HEAD with remote main",
    "risk": "GitSafe",
    "command": "\"LOCAL:  $(git rev-parse HEAD)\"; \"REMOTE: $((git ls-remote origin refs/heads/main).Split(\"`t\")[0])\"",
    "aliases": [
      "check if pushed",
      "compare local remote",
      "is main synced",
      "Compare local HEAD with remote main"
    ],
    "required_parameters": []
  },
  {
    "id": "git_stage_explicit_git_files",
    "provider": "git",
    "title": "Stage explicit Git files",
    "risk": "GitApproval",
    "command": "git add <files>",
    "aliases": [
      "Stage explicit Git files"
    ],
    "required_parameters": [
      "files"
    ]
  },
  {
    "id": "git_commit_staged_changes",
    "provider": "git",
    "title": "Commit staged changes",
    "risk": "GitApproval",
    "command": "git commit -m \"<message>\"",
    "aliases": [
      "Commit staged changes"
    ],
    "required_parameters": [
      "message"
    ]
  },
  {
    "id": "git_push_git_branch",
    "provider": "git",
    "title": "Push Git branch",
    "risk": "GitApproval",
    "command": "git push origin <branch>",
    "aliases": [
      "Push Git branch"
    ],
    "required_parameters": [
      "branch"
    ]
  },
  {
    "id": "git_pull_git_branch",
    "provider": "git",
    "title": "Pull Git branch",
    "risk": "GitApproval",
    "command": "git pull origin <branch>",
    "aliases": [
      "Pull Git branch"
    ],
    "required_parameters": [
      "branch"
    ]
  },
  {
    "id": "git_create_git_branch",
    "provider": "git",
    "title": "Create Git branch",
    "risk": "GitApproval",
    "command": "git switch -c <branch>",
    "aliases": [
      "Create Git branch"
    ],
    "required_parameters": [
      "branch"
    ]
  },
  {
    "id": "git_switch_git_branch",
    "provider": "git",
    "title": "Switch Git branch",
    "risk": "GitApproval",
    "command": "git switch <branch>",
    "aliases": [
      "Switch Git branch"
    ],
    "required_parameters": [
      "branch"
    ]
  },
  {
    "id": "git_hard_reset_git",
    "provider": "git",
    "title": "Hard reset Git",
    "risk": "GitExplain Only",
    "command": "git reset --hard <target>",
    "aliases": [
      "Hard reset Git"
    ],
    "required_parameters": [
      "target"
    ]
  },
  {
    "id": "git_clean_untracked_files",
    "provider": "git",
    "title": "Clean untracked files",
    "risk": "GitExplain Only",
    "command": "git clean -fd",
    "aliases": [
      "Clean untracked files"
    ],
    "required_parameters": []
  },
  {
    "id": "powershell_show_current_directory",
    "provider": "powershell",
    "title": "Show current directory",
    "risk": "PowerShellSafe",
    "command": "Get-Location",
    "aliases": [
      "Show current directory"
    ],
    "required_parameters": []
  },
  {
    "id": "powershell_list_files",
    "provider": "powershell",
    "title": "List files",
    "risk": "PowerShellSafe",
    "command": "Get-ChildItem",
    "aliases": [
      "List files"
    ],
    "required_parameters": []
  },
  {
    "id": "powershell_list_files_recursively",
    "provider": "powershell",
    "title": "List files recursively",
    "risk": "PowerShellSafe",
    "command": "Get-ChildItem <path> -Recurse",
    "aliases": [
      "List files recursively"
    ],
    "required_parameters": [
      "path"
    ]
  },
  {
    "id": "powershell_check_whether_path_exists",
    "provider": "powershell",
    "title": "Check whether path exists",
    "risk": "PowerShellSafe",
    "command": "Test-Path \"<path>\"",
    "aliases": [
      "Check whether path exists"
    ],
    "required_parameters": [
      "path"
    ]
  },
  {
    "id": "powershell_list_environment_variables",
    "provider": "powershell",
    "title": "List environment variables",
    "risk": "PowerShellSafe",
    "command": "Get-ChildItem Env:",
    "aliases": [
      "List environment variables"
    ],
    "required_parameters": []
  },
  {
    "id": "powershell_show_one_environment_variable",
    "provider": "powershell",
    "title": "Show one environment variable",
    "risk": "PowerShellSafe / Explain Only if secret-like",
    "command": "$env:<name>",
    "aliases": [
      "Show one environment variable"
    ],
    "required_parameters": [
      "name"
    ]
  },
  {
    "id": "powershell_set_temporary_environment_variable",
    "provider": "powershell",
    "title": "Set temporary environment variable",
    "risk": "PowerShellApproval",
    "command": "$env:<name>=\"<value>\"",
    "aliases": [
      "Set temporary environment variable"
    ],
    "required_parameters": [
      "name",
      "value"
    ]
  },
  {
    "id": "powershell_remove_temporary_environment_variable",
    "provider": "powershell",
    "title": "Remove temporary environment variable",
    "risk": "PowerShellApproval",
    "command": "Remove-Item Env:<name> -ErrorAction SilentlyContinue",
    "aliases": [
      "Remove temporary environment variable"
    ],
    "required_parameters": [
      "name"
    ]
  },
  {
    "id": "powershell_check_whether_command_exists",
    "provider": "powershell",
    "title": "Check whether command exists",
    "risk": "PowerShellSafe",
    "command": "Get-Command <command> -ErrorAction SilentlyContinue",
    "aliases": [
      "Check whether command exists"
    ],
    "required_parameters": [
      "command"
    ]
  },
  {
    "id": "powershell_show_process",
    "provider": "powershell",
    "title": "Show process",
    "risk": "PowerShellSafe",
    "command": "Get-Process -Name <name> -ErrorAction SilentlyContinue",
    "aliases": [
      "Show process"
    ],
    "required_parameters": [
      "name"
    ]
  },
  {
    "id": "powershell_stop_process",
    "provider": "powershell",
    "title": "Stop process",
    "risk": "PowerShellHigh Risk",
    "command": "Stop-Process -Name <name>",
    "aliases": [
      "Stop process"
    ],
    "required_parameters": [
      "name"
    ]
  },
  {
    "id": "powershell_test_tcp_port",
    "provider": "powershell",
    "title": "Test TCP port",
    "risk": "PowerShellSafe",
    "command": "Test-NetConnection <host> -Port <port>",
    "aliases": [
      "check port",
      "test connection",
      "is port open",
      "Test TCP port"
    ],
    "required_parameters": [
      "host",
      "port"
    ]
  },
  {
    "id": "powershell_show_listening_tcp_ports",
    "provider": "powershell",
    "title": "Show listening TCP ports",
    "risk": "PowerShellSafe",
    "command": "Get-NetTCPConnection -State Listen | Sort-Object LocalPort",
    "aliases": [
      "Show listening TCP ports"
    ],
    "required_parameters": []
  },
  {
    "id": "powershell_http_head_request",
    "provider": "powershell",
    "title": "HTTP HEAD request",
    "risk": "PowerShellSafe",
    "command": "curl.exe -I \"<url>\"",
    "aliases": [
      "check url",
      "show headers",
      "http head",
      "is site up",
      "HTTP HEAD request"
    ],
    "required_parameters": [
      "url"
    ]
  },
  {
    "id": "powershell_http_get_request",
    "provider": "powershell",
    "title": "HTTP GET request",
    "risk": "PowerShellSafe",
    "command": "curl.exe -i \"<url>\"",
    "aliases": [
      "HTTP GET request"
    ],
    "required_parameters": [
      "url"
    ]
  },
  {
    "id": "powershell_cors_preflight_check",
    "provider": "powershell",
    "title": "CORS preflight check",
    "risk": "PowerShellSafe",
    "command": "curl.exe -i -X OPTIONS \"<url>\" -H \"Origin: <origin>\" -H \"Access-Control-Request-Method: POST\" -H \"Access-Control-Request-Headers: Content-Type\"",
    "aliases": [
      "check cors",
      "test preflight",
      "cors test",
      "CORS preflight check"
    ],
    "required_parameters": [
      "url",
      "origin"
    ]
  },
  {
    "id": "waypoint_project_shortcuts_check_waypoint_azure_frontend",
    "provider": "waypoint_project_shortcuts",
    "title": "Check Waypoint Azure frontend",
    "risk": "AzureSafe",
    "command": "az staticwebapp show --name waypoint-web --resource-group waypoint-rg --query \"{Name:name,DefaultHostname:defaultHostname,Sku:sku.name,RepositoryUrl:repositoryUrl,Branch:branch}\" --output table",
    "aliases": [
      "Check Waypoint Azure frontend"
    ],
    "required_parameters": []
  },
  {
    "id": "waypoint_project_shortcuts_check_waypoint_azure_domains",
    "provider": "waypoint_project_shortcuts",
    "title": "Check Waypoint Azure domains",
    "risk": "AzureSafe",
    "command": "az staticwebapp hostname list --name waypoint-web --resource-group waypoint-rg --query \"[].{Domain:domainName,Status:status}\" --output table",
    "aliases": [
      "Check Waypoint Azure domains"
    ],
    "required_parameters": []
  },
  {
    "id": "waypoint_project_shortcuts_check_waypoint_azure_deployment",
    "provider": "waypoint_project_shortcuts",
    "title": "Check Waypoint Azure deployment",
    "risk": "AzureSafe",
    "command": "az staticwebapp environment list --name waypoint-web --resource-group waypoint-rg --query \"[].{Name:name,Hostname:hostname,SourceBranch:sourceBranch,Status:status}\" --output table",
    "aliases": [
      "Check Waypoint Azure deployment"
    ],
    "required_parameters": []
  },
  {
    "id": "waypoint_project_shortcuts_check_waypoint_frontend_url",
    "provider": "waypoint_project_shortcuts",
    "title": "Check Waypoint frontend URL",
    "risk": "PowerShellSafe",
    "command": "curl.exe -I https://waypoint.aperasmo.com",
    "aliases": [
      "Check Waypoint frontend URL"
    ],
    "required_parameters": []
  },
  {
    "id": "waypoint_project_shortcuts_check_waypoint_azure_generated_url",
    "provider": "waypoint_project_shortcuts",
    "title": "Check Waypoint Azure-generated URL",
    "risk": "PowerShellSafe",
    "command": "curl.exe -I https://kind-island-002a04700.7.azurestaticapps.net",
    "aliases": [
      "Check Waypoint Azure-generated URL"
    ],
    "required_parameters": []
  },
  {
    "id": "waypoint_project_shortcuts_check_waypoint_backend_health",
    "provider": "waypoint_project_shortcuts",
    "title": "Check Waypoint backend health",
    "risk": "PowerShellSafe",
    "command": "curl.exe -i https://p6frwmkmw8.execute-api.ap-southeast-2.amazonaws.com/prod/health",
    "aliases": [
      "Check Waypoint backend health"
    ],
    "required_parameters": []
  },
  {
    "id": "waypoint_project_shortcuts_check_waypoint_backend_categories",
    "provider": "waypoint_project_shortcuts",
    "title": "Check Waypoint backend categories",
    "risk": "PowerShellSafe",
    "command": "curl.exe -i https://p6frwmkmw8.execute-api.ap-southeast-2.amazonaws.com/prod/browse/categories",
    "aliases": [
      "Check Waypoint backend categories"
    ],
    "required_parameters": []
  },
  {
    "id": "waypoint_project_shortcuts_check_waypoint_frontend_deployment_workflow",
    "provider": "waypoint_project_shortcuts",
    "title": "Check Waypoint frontend deployment workflow",
    "risk": "GitHubSafe",
    "command": "gh run list --repo aperasmo/waypoint --workflow deploy-frontend.yml --limit 5",
    "aliases": [
      "Check Waypoint frontend deployment workflow"
    ],
    "required_parameters": []
  },
  {
    "id": "waypoint_project_shortcuts_trigger_waypoint_frontend_deployment",
    "provider": "waypoint_project_shortcuts",
    "title": "Trigger Waypoint frontend deployment",
    "risk": "GitHubApproval",
    "command": "gh workflow run deploy-frontend.yml --repo aperasmo/waypoint --ref main",
    "aliases": [
      "Trigger Waypoint frontend deployment"
    ],
    "required_parameters": []
  },
  {
    "id": "waypoint_project_shortcuts_watch_waypoint_frontend_deployment",
    "provider": "waypoint_project_shortcuts",
    "title": "Watch Waypoint frontend deployment",
    "risk": "GitHubSafe",
    "command": "$runId=(gh run list --repo aperasmo/waypoint --workflow deploy-frontend.yml --limit 1 --json databaseId | ConvertFrom-Json).databaseId; gh run watch $runId --repo aperasmo/waypoint --exit-status",
    "aliases": [
      "Watch Waypoint frontend deployment"
    ],
    "required_parameters": []
  },
  {
    "id": "waypoint_project_shortcuts_check_waypoint_lambda_configuration",
    "provider": "waypoint_project_shortcuts",
    "title": "Check Waypoint Lambda configuration",
    "risk": "AWSSafe",
    "command": "aws lambda get-function-configuration --function-name waypoint-backend --region ap-southeast-2 --query \"{FunctionName:FunctionName,State:State,LastUpdateStatus:LastUpdateStatus,MemorySize:MemorySize,Timeout:Timeout,Architecture:Architectures}\" --output json",
    "aliases": [
      "Check Waypoint Lambda configuration"
    ],
    "required_parameters": []
  },
  {
    "id": "waypoint_project_shortcuts_check_waypoint_lambda_quota_request",
    "provider": "waypoint_project_shortcuts",
    "title": "Check Waypoint Lambda quota request",
    "risk": "AWSSafe",
    "command": "aws service-quotas list-requested-service-quota-change-history-by-quota --service-code lambda --quota-code L-B99A9384 --region ap-southeast-2 --query \"RequestedQuotas[0].{Status:Status,DesiredValue:DesiredValue,Created:Created}\" --output table",
    "aliases": [
      "Check Waypoint Lambda quota request"
    ],
    "required_parameters": []
  },
  {
    "id": "waypoint_project_shortcuts_check_waypoint_aws_budget",
    "provider": "waypoint_project_shortcuts",
    "title": "Check Waypoint AWS budget",
    "risk": "AWSSafe",
    "command": "aws budgets describe-budget --account-id 658855381080 --budget-name Waypoint-Monthly-Budget --query \"Budget.{Name:BudgetName,Limit:BudgetLimit.Amount,Unit:BudgetLimit.Unit,TimeUnit:TimeUnit,Type:BudgetType}\" --output table",
    "aliases": [
      "Check Waypoint AWS budget"
    ],
    "required_parameters": []
  },
  {
    "id": "waypoint_project_shortcuts_check_waypoint_aws_budget_alerts",
    "provider": "waypoint_project_shortcuts",
    "title": "Check Waypoint AWS budget alerts",
    "risk": "AWSSafe",
    "command": "aws budgets describe-notifications-for-budget --account-id 658855381080 --budget-name Waypoint-Monthly-Budget --query \"Notifications[].{Type:NotificationType,Threshold:Threshold,ThresholdType:ThresholdType,Comparison:ComparisonOperator}\" --output table",
    "aliases": [
      "Check Waypoint AWS budget alerts"
    ],
    "required_parameters": []
  },
  {
    "id": "waypoint_project_shortcuts_check_waypoint_azure_consumption",
    "provider": "waypoint_project_shortcuts",
    "title": "Check Waypoint Azure consumption",
    "risk": "AzureSafe",
    "command": "az consumption usage list --start-date <start-date> --end-date <end-date> --query \"[?contains(instanceName, 'waypoint') || contains(resourceGroup, 'waypoint')].{Resource:instanceName,ResourceGroup:resourceGroup,Product:product,Cost:pretaxCost,Currency:currency}\" --output table",
    "aliases": [
      "Check Waypoint Azure consumption"
    ],
    "required_parameters": [
      "start_date",
      "end_date"
    ]
  }
]
