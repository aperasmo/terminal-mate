import {
  CircleHelp,
  ClipboardCopy,
  CornerDownLeft,
  History,
  Pencil,
  Search,
  ShieldCheck,
  X,
} from "lucide-react";
import { useEffect, useMemo, useRef, useState } from "react";

import {
  HELP_CATEGORIES,
  type HelpCategory,
  type HelpRequest,
} from "../lib/helpRequests";

type CommandCategory = Exclude<HelpCategory, "All">;
type HelpSafety = "Safe" | "Approval" | "High Risk" | "Explain Only";

interface HelpCommand {
  category: CommandCategory;
  description: string;
  example: string;
  safety: HelpSafety;
}

interface HelpModalProps {
  initialRequest: HelpRequest;
  onClose: () => void;
  onUseCommand: (command: string) => void;
}

const COMMANDS: HelpCommand[] = [
  {
    category: "Everyday",
    example: "Where am I?",
    description: "Show the active working directory.",
    safety: "Safe",
  },

  {
    category: "GitHub",
    example: "Show GitHub authentication status.",
    description: "Check whether GitHub CLI is authenticated.",
    safety: "Safe",
  },
  {
    category: "GitHub",
    example: "List GitHub workflows.",
    description:
      "List recent workflow runs. TerminalMate asks for the repository and workflow when they are missing.",
    safety: "Safe",
  },
  {
    category: "GitHub",
    example: "Trigger workflow <workflow> for <owner/repo> on <branch>.",
    description: "Start a GitHub Actions workflow after approval.",
    safety: "Approval",
  },
  {
    category: "GitHub",
    example: "List GitHub Actions secret names for <owner/repo>.",
    description: "List secret names without exposing secret values.",
    safety: "Safe",
  },

  {
    category: "Docker",
    example: "Docker images.",
    description: "List locally available Docker images.",
    safety: "Safe",
  },
  {
    category: "Docker",
    example: "List running Docker containers.",
    description: "Show currently running containers with docker ps.",
    safety: "Safe",
  },
  {
    category: "Docker",
    example: "Show Docker container logs <container>.",
    description: "Display logs for a named container.",
    safety: "Safe",
  },
  {
    category: "Docker",
    example: "Stop Docker container <container>.",
    description: "Stop a running container after approval.",
    safety: "Approval",
  },
  {
    category: "Docker",
    example: "Remove Docker container <container>.",
    description:
      "Remove a container after high-risk confirmation. Force removal is never added implicitly.",
    safety: "High Risk",
  },

  {
    category: "Git",
    example: "Git status.",
    description: "Show branch and concise working-tree status.",
    safety: "Safe",
  },
  {
    category: "Git",
    example: "Show Git diff.",
    description: "Inspect unstaged changes.",
    safety: "Safe",
  },
  {
    category: "Git",
    example: "Show staged Git diff.",
    description: "Inspect changes currently staged for commit.",
    safety: "Safe",
  },
  {
    category: "Git",
    example: "Stage Git files <files>.",
    description: "Stage only explicitly named files after approval.",
    safety: "Approval",
  },
  {
    category: "Git",
    example: "Hard reset Git to <target>.",
    description:
      "Explain this history-rewriting operation without executing it automatically.",
    safety: "Explain Only",
  },

  {
    category: "PowerShell",
    example: "Show current directory.",
    description: "Display the current PowerShell location.",
    safety: "Safe",
  },

  {
    category: "Security",
    example: "Check for any leaks.",
    description:
      "Run a redacted Gitleaks scan across all Git history and save a JSON report in the temporary directory using the active repository name.",
    safety: "Safe",
  },
  {
    category: "Security",
    example: "Check current files for leaks.",
    description:
      "Scan the current working tree with Gitleaks without traversing Git history.",
    safety: "Safe",
  },
  {
    category: "Security",
    example: "Check latest commit for leaks.",
    description: "Scan only the latest Git commit with redacted output.",
    safety: "Safe",
  },
  {
    category: "Security",
    example: "Show Gitleaks report.",
    description:
      "Read the repository-specific JSON report from the temporary directory.",
    safety: "Safe",
  },
  {
    category: "Security",
    example: "Gitleaks version.",
    description: "Show the installed Gitleaks version.",
    safety: "Safe",
  },
  {
    category: "PowerShell",
    example: "Check whether path <path> exists.",
    description: "Test a filesystem path without changing it.",
    safety: "Safe",
  },
  {
    category: "PowerShell",
    example: "Check whether command <command> exists.",
    description: "Find a command available in the current environment.",
    safety: "Safe",
  },
  {
    category: "PowerShell",
    example: "Test TCP port <host> <port>.",
    description: "Test network reachability to one host and TCP port.",
    safety: "Safe",
  },
  {
    category: "Everyday",
    example: "Take me to the terminal-mate folder.",
    description:
      "Change to a named folder. A single leading backslash is treated as workspace-relative.",
    safety: "Safe",
  },
  {
    category: "Everyday",
    example: "Go up one directory.",
    description: "Move to the parent directory.",
    safety: "Safe",
  },
  {
    category: "Everyday",
    example: "List the files here.",
    description: "Show files and folders in the current directory.",
    safety: "Safe",
  },
  {
    category: "Everyday",
    example: "Show hidden files with name and mode.",
    description:
      "Include hidden items and show their names and filesystem modes.",
    safety: "Safe",
  },
  {
    category: "Everyday",
    example: "List the folders here.",
    description: "Show only folders in the current directory.",
    safety: "Safe",
  },
  {
    category: "Everyday",
    example: "List files .claude in this folder.",
    description:
      "Inspect a named file or folder and list folder contents when appropriate.",
    safety: "Safe",
  },
  {
    category: "Everyday",
    example: "Show me all the .sh files.",
    description: "Find shell scripts below the current directory.",
    safety: "Safe",
  },
  {
    category: "Everyday",
    example: "Find the file package.json.",
    description:
      "Search recursively for a named file. Inaccessible cache or system folders are skipped when needed instead of stopping the whole search.",
    safety: "Safe",
  },
  {
    category: "Everyday",
    example: "Show me the contents of package.json.",
    description: "Read a text file in the terminal.",
    safety: "Safe",
  },
  {
    category: "Everyday",
    example: "Show me lines 20-100 of sample.txt.",
    description: "Read a selected range from a text file.",
    safety: "Safe",
  },
  {
    category: "Everyday",
    example: "Search for TODO in this folder.",
    description:
      "Search file contents recursively for text. Permission-denied folders can be retried with safe skip-inaccessible handling.",
    safety: "Safe",
  },
  {
    category: "Everyday",
    example: "Create a folder called drafts.",
    description: "Create a directory after approval.",
    safety: "Approval",
  },
  {
    category: "Everyday",
    example: "Create folders components, api, services.",
    description:
      "Create multiple workspace folders in one approved action. Separate paths with commas; duplicate paths are removed.",
    safety: "Approval",
  },
  {
    category: "Everyday",
    example: "Create file scripts/example.py.",
    description:
      "Create a new file inside the active workspace after approval. A leading slash or backslash is treated as workspace-relative.",
    safety: "Approval",
  },
  {
    category: "Everyday",
    example: "Create files src/index.ts, src/app.ts, README.md.",
    description:
      "Create multiple workspace files in one approved action. TerminalMate checks every path before creating any file.",
    safety: "Approval",
  },
  {
    category: "Everyday",
    example: "Delete file scripts/example.py.",
    description:
      "Delete one file after high-risk confirmation. Folder paths and paths outside the workspace are refused.",
    safety: "High Risk",
  },
  {
    category: "Everyday",
    example: "Copy file report.py from D:\\Downloads to \\scripts.",
    description:
      "Copy one external file into a folder in the active workspace. Existing destination files are never overwritten silently.",
    safety: "Approval",
  },
  {
    category: "Everyday",
    example: "Cut file D:\\Downloads\\report.py to \\scripts.",
    description:
      "Move one external file into a folder in the active workspace after approval. The source is removed only when the move succeeds.",
    safety: "Approval",
  },
  {
    category: "Everyday",
    example: "Back up app/api/routes/ask.py as app/api/routes/ask_before.py.",
    description:
      "Copy a workspace file to a new backup path without overwriting an existing backup. The word 'as' is optional.",
    safety: "Approval",
  },
  {
    category: "Everyday",
    example:
      "Replace app/api/routes/ask.py with D:\\Downloads\\ask_candidate.py.",
    description:
      "Replace one existing workspace file after typed RUN confirmation. The word 'with' is optional.",
    safety: "High Risk",
  },
  {
    category: "Everyday",
    example:
      "Back up app/api/routes/ask.py as app/api/routes/ask_before.py, then replace the original with D:\\Downloads\\ask_candidate.py.",
    description:
      "Validate all paths, create a non-overwriting backup, then replace the original in one reviewed High Risk workflow. 'As' and 'with' are optional.",
    safety: "High Risk",
  },
  {
    category: "Everyday",
    example: "Show me the running processes.",
    description: "List active processes on this machine.",
    safety: "Safe",
  },
  {
    category: "Everyday",
    example: "Show listening ports.",
    description: "List ports accepting network connections.",
    safety: "Safe",
  },
  {
    category: "Everyday",
    example: "Find port 8100.",
    description:
      "Inspect the process or connection using a specific port. Check, inspect, locate, show, 'find process using port', and 'which process owns port' are also supported.",
    safety: "Safe",
  },
  {
    category: "Everyday",
    example: "Run scripts/seed_data.py.",
    description:
      "Run one supported script with the interpreter for the active workspace runtime.",
    safety: "Approval",
  },
  {
    category: "Everyday",
    example:
      "Run python collect_manual.py --prefixes R --out ../generic-residence",
    description:
      "Run one Python script with arguments, preserving each argument across PowerShell and WSL.",
    safety: "Approval",
  },
  {
    category: "Everyday",
    example: "Run all scripts in this folder.",
    description:
      "Run supported scripts sequentially, newest modified first, and stop at the first failure.",
    safety: "Approval",
  },
  {
    category: "Everyday",
    example: "Run all scripts in this folder by name.",
    description: "Run supported scripts sequentially in filename order.",
    safety: "Approval",
  },
  {
    category: "Everyday",
    example: "Run all scripts in this folder oldest first.",
    description: "Run supported scripts sequentially from oldest to newest.",
    safety: "Approval",
  },
  {
    category: "Everyday",
    example: "Run all Python scripts recursively.",
    description:
      "Run one script type from this folder and its subfolders after approval.",
    safety: "Approval",
  },
  {
    category: "Everyday",
    example: "Show script execution order in this folder.",
    description: "Preview the script order without executing anything.",
    safety: "Safe",
  },

  {
    category: "AWS",
    example: "Show my AWS identity.",
    description: "Show the AWS account and caller used by later commands.",
    safety: "Safe",
  },
  {
    category: "AWS",
    example:
      "Create an AWS monthly budget named Waypoint-Monthly-Budget for 5 USD in account <12-digit-account-id>.",
    description:
      "Create a COST budget after TerminalMate confirms every required value and obtains typed high-risk approval.",
    safety: "High Risk",
  },
  {
    category: "AWS",
    example:
      "Create an AWS monthly budget alert named Waypoint-Monthly-Budget for 5 USD in account <12-digit-account-id>, notifying <email> when ACTUAL spend exceeds 80 percent.",
    description:
      "Create a COST budget and email notification together. TerminalMate asks for every missing account, amount, threshold, and recipient value before typed approval.",
    safety: "High Risk",
  },
  {
    category: "AWS",
    example: "List AWS EC2 instances in <region>.",
    description: "List EC2 instance state without modifying infrastructure.",
    safety: "Safe",
  },
  {
    category: "AWS",
    example:
      "Create 1 AWS EC2 instance using image <ami-id>, type <instance-type>, key <key-name>, security groups <sg-id>, and subnet <subnet-id> in <region>.",
    description:
      "Create an EC2 instance only after all image, network, key, and region inputs are supplied and approved.",
    safety: "High Risk",
  },

  {
    category: "Azure",
    example: "Show Azure CLI version.",
    description: "Confirm that Azure CLI is installed.",
    safety: "Safe",
  },
  {
    category: "Azure",
    example: "Show current Azure account.",
    description: "Display the active Azure account and subscription.",
    safety: "Safe",
  },
  {
    category: "Azure",
    example: "List Azure subscriptions.",
    description: "List subscriptions available to the signed-in account.",
    safety: "Safe",
  },
  {
    category: "Azure",
    example: "Sign in to Azure.",
    description: "Start the Azure CLI authentication flow.",
    safety: "Approval",
  },
  {
    category: "Azure",
    example: "Sign in to Azure with device code.",
    description: "Authenticate Azure CLI using a browser device code.",
    safety: "Approval",
  },
  {
    category: "Azure",
    example: "Switch Azure subscription to <subscription>.",
    description: "Select the subscription used by later Azure commands.",
    safety: "Approval",
  },
  {
    category: "Azure",
    example: "Create Azure resource group <group> in <location>.",
    description: "Create a resource group in an Azure region.",
    safety: "High Risk",
  },
  {
    category: "Azure",
    example:
      "Create Azure storage account <account> in resource group <group>.",
    description:
      "Create an Azure Storage account after typed high-risk confirmation.",
    safety: "High Risk",
  },
  {
    category: "Azure",
    example:
      "Create Azure storage container <container> in storage account <account>.",
    description: "Create a blob container after typed high-risk confirmation.",
    safety: "High Risk",
  },
  {
    category: "Azure",
    example: "List Azure storage keys for <account> in resource group <group>.",
    description:
      "Explain how to retrieve sensitive storage credentials without executing it.",
    safety: "Explain Only",
  },
  {
    category: "Azure",
    example: "List Azure VMs.",
    description: "List virtual machines visible in the active subscription.",
    safety: "Safe",
  },
  {
    category: "Azure",
    example: "Show Azure VM <vm> in resource group <group>.",
    description: "Inspect one virtual machine.",
    safety: "Safe",
  },
  {
    category: "Azure",
    example: "List Azure resources in resource group <group>.",
    description: "List resources belonging to a resource group.",
    safety: "Safe",
  },
  {
    category: "Azure",
    example: "List Azure network security groups.",
    description: "List network security groups in the active subscription.",
    safety: "Safe",
  },
  {
    category: "Azure",
    example: "Start Azure VM <vm> in resource group <group>.",
    description: "Start a stopped Azure virtual machine after approval.",
    safety: "Approval",
  },
  {
    category: "Azure",
    example: "Stop Azure VM <vm> in resource group <group>.",
    description: "Stop an Azure virtual machine after approval.",
    safety: "Approval",
  },
  {
    category: "Azure",
    example: "Deallocate Azure VM <vm> in resource group <group>.",
    description:
      "Release a VM's compute allocation after high-risk confirmation.",
    safety: "High Risk",
  },
  {
    category: "Azure",
    example: "Delete Azure resource group <group>.",
    description: "Explain this destructive operation without executing it.",
    safety: "Explain Only",
  },

  {
    category: "Azure",
    example:
      "Create an Azure monthly budget named <name> for <amount> from <YYYY-MM-DD> to <YYYY-MM-DD>.",
    description:
      "Create an Azure cost budget after its amount, dates, and scope are reviewed.",
    safety: "High Risk",
  },
  {
    category: "Azure",
    example:
      "Create Azure VM <name> in resource group <group> using image <image> and admin <username>.",
    description:
      "Create a virtual machine after required identity, image, and resource values are supplied.",
    safety: "High Risk",
  },

  {
    category: "Google Cloud",
    example: "Show my active Google Cloud account and project.",
    description: "Display the active gcloud identity and project.",
    safety: "Safe",
  },
  {
    category: "Google Cloud",
    example:
      "Create a Google Cloud budget named Waypoint-Monthly-Budget for 5 USD on billing account <billing-account-id>.",
    description:
      "Create a Cloud Billing budget after the billing account and amount are reviewed.",
    safety: "High Risk",
  },
  {
    category: "Google Cloud",
    example: "List Google Cloud compute instances in zone <zone>.",
    description: "List Compute Engine instances without modifying them.",
    safety: "Safe",
  },
  {
    category: "Google Cloud",
    example:
      "Create Google Compute Engine instance <name> in zone <zone> using machine type <type>.",
    description:
      "Create a Compute Engine VM only after the instance, zone, and machine type are supplied and approved.",
    safety: "High Risk",
  },

  {
    category: "Terraform",
    example: "Show Terraform version.",
    description: "Confirm that Terraform is installed.",
    safety: "Safe",
  },
  {
    category: "Terraform",
    example: "Terraform init.",
    description: "Initialize the current Terraform working directory.",
    safety: "Approval",
  },
  {
    category: "Terraform",
    example: "Terraform fmt.",
    description: "Format Terraform configuration files.",
    safety: "Safe",
  },
  {
    category: "Terraform",
    example: "Terraform validate.",
    description: "Validate the current Terraform configuration.",
    safety: "Safe",
  },
  {
    category: "Terraform",
    example: "Terraform plan.",
    description: "Preview infrastructure changes without applying them.",
    safety: "Safe",
  },
  {
    category: "Terraform",
    example: "Terraform apply.",
    description:
      "Apply infrastructure after typed RUN confirmation. TerminalMate renders -auto-approve because command input is not interactive.",
    safety: "High Risk",
  },
  {
    category: "Terraform",
    example: "Terraform destroy.",
    description:
      "Destroy infrastructure after typed RUN confirmation. TerminalMate renders -auto-approve because command input is not interactive.",
    safety: "High Risk",
  },
  {
    category: "Terraform",
    example: "List Terraform state.",
    description: "List resources tracked in Terraform state.",
    safety: "Safe",
  },
  {
    category: "Terraform",
    example: "Show Terraform state <resource-address>.",
    description: "Inspect one resource in Terraform state.",
    safety: "Safe",
  },
  {
    category: "Terraform",
    example: "Show Terraform outputs.",
    description: "Display outputs from the current Terraform state.",
    safety: "Safe",
  },
  {
    category: "Terraform",
    example: "Terraform import <resource-address> <resource-id>.",
    description: "Import an existing resource into Terraform state.",
    safety: "High Risk",
  },

  {
    category: "SSH",
    example: 'Generate SSH key named <key-name> with comment "<comment>".',
    description: "Create a new SSH key pair after approval.",
    safety: "Approval",
  },
  {
    category: "SSH",
    example: "Connect to <user>@<host> and run <remote-command>.",
    description:
      "Run one non-interactive remote command over SSH, with optional key and port inputs, after approval.",
    safety: "Approval",
  },
  {
    category: "SSH",
    example: "Show public key named <key-name>.",
    description: "Display a public SSH key for copying to a service.",
    safety: "Safe",
  },
  {
    category: "SSH",
    example: "SSH <admin>@<ip> using key <key-name>.",
    description: "Connect to a remote host using a named private key.",
    safety: "Approval",
  },
];

function safetyClass(safety: HelpSafety) {
  return safety.toLowerCase().replaceAll(" ", "-");
}

export function HelpModal({
  initialRequest,
  onClose,
  onUseCommand,
}: HelpModalProps) {
  const [category, setCategory] = useState<HelpCategory>(
    initialRequest.category,
  );
  const [query, setQuery] = useState(initialRequest.query);
  const [topicNotice, setTopicNotice] = useState(
    initialRequest.unsupportedTopic,
  );
  const searchRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    searchRef.current?.focus();
    const closeOnEscape = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        onClose();
      }
    };
    window.addEventListener("keydown", closeOnEscape);
    return () => window.removeEventListener("keydown", closeOnEscape);
  }, [onClose]);

  const filteredCommands = useMemo(() => {
    const normalizedQuery = query.trim().toLowerCase();
    return COMMANDS.filter((command) => {
      const categoryMatches =
        category === "All" || command.category === category;
      const queryMatches =
        !normalizedQuery ||
        `${command.example} ${command.description} ${command.safety}`
          .toLowerCase()
          .includes(normalizedQuery);
      return categoryMatches && queryMatches;
    });
  }, [category, query]);

  return (
    <div
      className="help-backdrop"
      onMouseDown={(event) => {
        if (event.target === event.currentTarget) {
          onClose();
        }
      }}
    >
      <section
        aria-labelledby="help-title"
        aria-modal="true"
        className="help-dialog"
        role="dialog"
      >
        <header className="help-header">
          <div>
            <p className="help-eyebrow">Command reference</p>
            <h2 id="help-title">
              {topicNotice
                ? `${topicNotice} command help`
                : category === "All"
                  ? "What can I ask TerminalMate?"
                  : `${category} command help`}
            </h2>
          </div>
          <button
            className="icon-button"
            type="button"
            onClick={onClose}
            aria-label="Close help"
            title="Close help"
          >
            <X size={19} />
          </button>
        </header>

        <div className="help-safety-note">
          <ShieldCheck size={18} />
          <span>
            TerminalMate checks hand-tuned patterns and 144 deterministic
            catalog entries before using AI fallback. Missing required values
            produce a local question instead of a guessed command. Every result
            is checked against the active environment and the same safety
            policy.
          </span>
        </div>

        <div className="help-controls-region">
          {topicNotice ? (
            <div className="help-topic-notice" role="status">
              <CircleHelp size={18} aria-hidden="true" />
              <div>
                <strong>{topicNotice} commands are not available yet.</strong>
                <span>
                  TerminalMate will not invent commands for an unsupported tool.
                  Browse the currently supported categories below.
                </span>
              </div>
            </div>
          ) : null}

          <div className="help-workflow-note" aria-label="Runs and output help">
            <div>
              <Search size={17} aria-hidden="true" />
              <span>
                <strong>Deterministic command catalog</strong>
                Azure, AWS, GitHub, Docker, Git, and PowerShell requests are
                matched locally first, including short phrases such as “git
                status”, “docker images”, “show lambda quota”, and “check
                consumption usage”. AI fallback is used only when no supported
                local intent or clarification applies.
              </span>
            </div>
            <div>
              <Pencil size={17} aria-hidden="true" />
              <span>
                <strong>Workspace aliases</strong>
                Use the pencil beside a workspace to rename its saved alias. The
                same folder can be added more than once under different aliases,
                with a separate runtime, terminal session, warnings, and Recent
                runs for every entry.
              </span>
            </div>
            <div>
              <History size={17} aria-hidden="true" />
              <span>
                <strong>Recent runs</strong>
                Keeps the latest 10 runs per workspace across restarts. Select a
                run to place its original request back in the command line.
              </span>
            </div>
            <div>
              <ClipboardCopy size={17} aria-hidden="true" />
              <span>
                <strong>Complete output</strong>
                The terminal shows the latest 1,000 lines with a local
                dd/mm/yyyy 24-hour timestamp. Each command appears in green
                before its output, and command runs are separated by a divider.
                Whitespace-only rows are hidden from this view. These display
                changes are UI-only and are not added to command log files. Use
                Open log for the complete raw file or Copy log to copy all raw
                output to the clipboard.
              </span>
            </div>
            <div>
              <ShieldCheck size={17} aria-hidden="true" />
              <span>
                <strong>Guided recovery</strong>
                Known failures offer a safe next action when possible. Recursive
                PowerShell searches that hit inaccessible folders can be retried
                while skipping only those folders; mutating follow-ups still use
                their normal approval level.
              </span>
            </div>
          </div>

          <div className="help-controls">
            <label className="help-search">
              <Search size={17} aria-hidden="true" />
              <span className="sr-only">Search commands</span>
              <input
                ref={searchRef}
                value={query}
                onChange={(event) => {
                  setQuery(event.target.value);
                  setTopicNotice(undefined);
                }}
                placeholder="Search commands, tools, or actions..."
                type="search"
              />
            </label>
            <div
              className="help-categories"
              role="tablist"
              aria-label="Command category"
            >
              {HELP_CATEGORIES.map((item) => (
                <button
                  aria-selected={category === item}
                  className={category === item ? "active" : ""}
                  key={item}
                  onClick={() => {
                    setCategory(item);
                    setQuery("");
                    setTopicNotice(undefined);
                  }}
                  role="tab"
                  type="button"
                >
                  {item}
                </button>
              ))}
            </div>
          </div>
        </div>

        <div className="help-results-summary">
          {filteredCommands.length} supported request
          {filteredCommands.length === 1 ? "" : "s"}
          <span>PowerShell + WSL</span>
        </div>

        <div className="help-command-list">
          {filteredCommands.map((command) => (
            <article
              className="help-command-row"
              key={`${command.category}-${command.example}`}
            >
              <div className="help-command-copy">
                <div className="help-command-heading">
                  <span className="help-command-category">
                    {command.category}
                  </span>
                  <span
                    className={`help-safety-badge ${safetyClass(command.safety)}`}
                  >
                    {command.safety}
                  </span>
                </div>
                <code>{command.example}</code>
                <p>{command.description}</p>
              </div>
              <button
                className="help-use-command"
                onClick={() => onUseCommand(command.example)}
                type="button"
                title="Place this request in the command line"
              >
                Use
                <CornerDownLeft size={15} />
              </button>
            </article>
          ))}
          {filteredCommands.length === 0 ? (
            <div className="help-empty">
              <CircleHelp size={24} aria-hidden="true" />
              <strong>No supported requests match this topic.</strong>
              <span>
                Try Everyday, AWS, Azure, GitHub, Docker, Git, PowerShell,
                Security, Google Cloud, Terraform, or SSH.
              </span>
            </div>
          ) : null}
        </div>

        <footer className="help-footer">
          Replace values inside <code>&lt;angle brackets&gt;</code> before
          running a request. Exact Bash <code>\</code> continuations, PowerShell{" "}
          <code>`</code> continuations, pipelines continued after a trailing{" "}
          <code>|</code>, and balanced multiline PowerShell expressions are
          accepted. PowerShell variable setup plus one
          <code>foreach</code> block, or a range pipeline using
          <code>ForEach-Object</code>, is also treated as one reviewed script so
          variables remain available throughout the block. A PowerShell block
          beginning with an <code>$env:NAME = value</code> assignment is
          likewise kept as one reviewed, process-scoped script, so its commands
          share the temporary environment and an optional cleanup line can see
          it. Ordinary dependent statements are kept together too, such as
          assigning
          <code>$container</code> before reading <code>$container.Mounts</code>,
          or preparing <code>$body</code> before an{" "}
          <code>Invoke-RestMethod</code>
          request. Windows PowerShell 5.1 compatibility failures such as an
          unavailable <code>RandomNumberGenerator.Fill</code> method receive a
          compatible reviewed retry. Independent commands pasted on separate
          lines are queued and run one at a time, with a safety decision for
          every command. Before the first command, choose Review each or
          explicitly auto-approve Caution commands in that pasted batch.
          High-risk commands still require typed confirmation, and blocked
          commands never run. This works for two commands or larger batches. The
          queue stops on the first blocked, cancelled, stopped, or failed
          command. Explain Only operations never execute.
        </footer>
      </section>
    </div>
  );
}
