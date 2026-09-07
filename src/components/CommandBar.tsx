import { invoke } from "@tauri-apps/api/core";
import {
  ArrowLeft,
  ArrowRight,
  CircleHelp,
  Clock3,
  Copy,
  CornerDownLeft,
  FileText,
  ShieldAlert,
  ShieldX,
  Square,
} from "lucide-react";
import { ClipboardEvent, FormEvent, useEffect, useRef, useState } from "react";

import { formatDuration } from "../lib/duration";
import { parseHelpRequest, type HelpRequest } from "../lib/helpRequests";
import {
  loadDirectoryHistory,
  saveDirectoryHistory,
  type StoredDirectoryHistory,
} from "../lib/storage";
import {
  parseSequentialCommands,
  shouldAutoApproveSequentialCommand,
  type SequentialApprovalMode,
} from "../lib/sequentialCommands";
import type {
  AzureSubscription,
  AzureVirtualMachine,
  CommandFailureDiagnosis,
  CommandHistoryEntry,
  CommandRunFinishedEvent,
  CommandRunSummary,
  CreatedVirtualMachine,
  ResolvedCommand,
  RiskDecision,
  SessionSummary,
  Workspace,
} from "../lib/types";
import { parseVmActionRequest, type VmAction } from "../lib/vmActionRequest";

interface CommandBarProps {
  workspace?: Workspace;
  session?: SessionSummary;
  activeRun?: CommandRunSummary;
  lastFinished?: CommandRunFinishedEvent;
  lastHistoryEntry?: CommandHistoryEntry;
  currentDirectory?: string;
  recalledRequest?: {
    request: string;
    token: number;
  } | null;
  onRunCommand: (
    request: string,
    command: string,
    decision: RiskDecision,
  ) => Promise<CommandRunSummary>;
  onNavigateDirectory: (directory: string) => Promise<string>;
  onStopCommand: () => Promise<void>;
  onSendSessionInput: (input: string) => Promise<void>;
  onOpenLog: (path: string) => Promise<void>;
  onOpenHelp: (request: HelpRequest) => void;
  onOpenFileEditor: (path: string, workingDirectory: string) => void;
  onOpenExternalTerminal: (
    workingDirectory: string,
    shell: string,
    command: string,
  ) => void;
}

type Phase = "idle" | "classifying" | "pending-approval";

interface SequentialCommandQueue {
  current: string;
  remaining: string[];
  completed: number;
  total: number;
  activeRunId: string | null;
  approvalMode: SequentialApprovalMode;
}

type AzureRecoveryStage =
  | "checking"
  | "choosing"
  | "setting"
  | "reauthRequired"
  | "signingIn"
  | "retryReady"
  | "retrying"
  | "deadEnd";

interface AzureRecovery {
  originalRequest: string;
  originalCommand: string;
  failedSubscriptionId?: string;
  subscriptions: AzureSubscription[];
  stage: AzureRecoveryStage;
  actionRequest: string;
  reauthAttempts: number;
}

const AZURE_SUBSCRIPTION_CHECK_REQUEST = "Check available Azure subscriptions";
const AZURE_SUBSCRIPTION_CHECK_COMMAND = `az account list --query "[?state=='Enabled'].{name:name,id:id,tenantId:tenantId,isDefault:isDefault}" --output json`;
const AZURE_LOGIN_REQUEST = "Sign in to Azure with device code";
const AZURE_VM_LIST_REQUEST = "List Azure virtual machines";
// --output json rather than the --output table a user might picture: the
// point of this lookup is to programmatically resolve/disambiguate a VM,
// which needs structured data, not a rendered table (matches the same
// choice already made for the subscription-check lookup above).
// --show-details adds powerState, so the picker can show which VMs are
// already running vs. deallocated instead of leaving that to guesswork.
const AZURE_VM_LIST_COMMAND = "az vm list --show-details --output json";
const VM_ACTION_LABEL: Record<VmAction, string> = {
  start: "Start",
  deallocate: "Deallocate",
};
const VM_STATUS_CHECK_REQUEST = "Check Azure VM status";
// --output table here, unlike the --output json used to resolve/pick a VM
// above: this run has nothing left to parse, it is purely for the user to
// read the resulting state directly in the terminal output.
const VM_STATUS_CHECK_COMMAND = "az vm list --show-details --output table";
const VM_START_OR_DEALLOCATE_PATTERN = /\baz\s+vm\s+(start|deallocate)\b/i;

interface VmActionResolution {
  action: VmAction;
  vmName?: string;
  stage: "listing" | "choosing";
  candidates: AzureVirtualMachine[];
}

const RISK_LABEL: Record<RiskDecision["level"], string> = {
  safe: "SAFE",
  caution: "CAUTION",
  highRisk: "HIGH RISK",
  blocked: "BLOCKED",
};

function errorText(error: unknown, fallback: string) {
  if (error instanceof Error) {
    return error.message;
  }
  if (typeof error === "string" && error.trim()) {
    return error;
  }
  return fallback;
}

function normalizeDirectoryPath(path: string) {
  let normalized = path
    .trim()
    .replace(/^Microsoft\.PowerShell\.Core\\FileSystem::/i, "");

  if (/^\\\\\?\\UNC\\/i.test(normalized)) {
    normalized = `\\\\${normalized.slice(8)}`;
  } else {
    normalized = normalized.replace(/^\\\\\?\\/, "");
  }

  if (normalized !== "/" && !/^[A-Za-z]:[\\/]$/.test(normalized)) {
    normalized = normalized.replace(/[\\/]+$/, "");
  }
  return normalized;
}

function comparableDirectory(path: string) {
  const normalized = normalizeDirectoryPath(path).replace(/\\/g, "/");
  return /^[A-Za-z]:\//.test(normalized) || normalized.startsWith("//")
    ? normalized.toLocaleLowerCase()
    : normalized;
}

function sameDirectory(left: string, right: string) {
  return comparableDirectory(left) === comparableDirectory(right);
}

function initialDirectoryHistory(rootPath: string, currentPath: string) {
  const root = normalizeDirectoryPath(rootPath);
  const current = normalizeDirectoryPath(currentPath);
  const comparableRoot = comparableDirectory(root);
  const comparableCurrent = comparableDirectory(current);

  if (comparableRoot === comparableCurrent) {
    return { entries: [root], index: 0 };
  }

  if (!comparableCurrent.startsWith(`${comparableRoot}/`)) {
    return { entries: [current], index: 0 };
  }

  const normalizedCurrent = current.replace(/\\/g, "/");
  const normalizedRoot = root.replace(/\\/g, "/");
  const relativeParts = normalizedCurrent
    .slice(normalizedRoot.length)
    .replace(/^\/+/, "")
    .split("/")
    .filter(Boolean);
  const separator = current.includes("\\") ? "\\" : "/";
  const entries = [root];
  let cursor = root;
  for (const part of relativeParts) {
    cursor = cursor.endsWith(separator)
      ? `${cursor}${part}`
      : `${cursor}${separator}${part}`;
    entries.push(cursor);
  }
  return { entries, index: entries.length - 1 };
}

export function CommandBar({
  workspace,
  session,
  activeRun,
  lastFinished,
  lastHistoryEntry,
  currentDirectory,
  recalledRequest,
  onRunCommand,
  onNavigateDirectory,
  onStopCommand,
  onSendSessionInput,
  onOpenLog,
  onOpenHelp,
  onOpenFileEditor,
  onOpenExternalTerminal,
}: CommandBarProps) {
  const [message, setMessage] = useState("");
  const [sessionInput, setSessionInput] = useState("");
  const [sendingSessionInput, setSendingSessionInput] = useState(false);
  const sessionInputRef = useRef<HTMLInputElement>(null);
  const [phase, setPhase] = useState<Phase>("idle");
  const [pendingRequest, setPendingRequest] = useState<string | null>(null);
  const [pendingCommand, setPendingCommand] = useState<string | null>(null);
  const [pendingDecision, setPendingDecision] = useState<RiskDecision | null>(
    null,
  );
  const [sequentialQueue, setSequentialQueue] =
    useState<SequentialCommandQueue | null>(null);
  const [highRiskConfirmation, setHighRiskConfirmation] = useState("");
  const [explainOnly, setExplainOnly] = useState<ResolvedCommand | null>(null);
  const [interpretedNote, setInterpretedNote] = useState<string | null>(null);
  const [notice, setNotice] = useState<{
    level: RiskDecision["level"];
    text: string;
    editablePath?: string;
    externalTerminalCommand?: string;
  } | null>(null);
  const [diagnosis, setDiagnosis] = useState<CommandFailureDiagnosis | null>(
    null,
  );
  const [azureRecovery, setAzureRecovery] = useState<AzureRecovery | null>(
    null,
  );
  const [vmLifecycle, setVmLifecycle] = useState<CreatedVirtualMachine | null>(
    null,
  );
  const [vmActionResolution, setVmActionResolution] =
    useState<VmActionResolution | null>(null);
  const inputRef = useRef<HTMLInputElement | null>(null);
  const approvalCancelRef = useRef<HTMLButtonElement | null>(null);
  const batchReviewRef = useRef<HTMLButtonElement | null>(null);
  const highRiskConfirmationRef = useRef<HTMLInputElement | null>(null);
  const processedRecoveryRunId = useRef<string | null>(null);
  const processedVmDetectionRunId = useRef<string | null>(null);
  const processedVmActionRunId = useRef<string | null>(null);
  const processedVmStatusFollowupRunId = useRef<string | null>(null);
  const [clockMs, setClockMs] = useState(Date.now());
  const [copiedLogRunId, setCopiedLogRunId] = useState<string | null>(null);
  const [copyingLog, setCopyingLog] = useState(false);
  const [directoryHistoryByWorkspace, setDirectoryHistoryByWorkspace] =
    useState<Record<string, StoredDirectoryHistory>>(loadDirectoryHistory);
  const [navigatingDirectory, setNavigatingDirectory] = useState(false);

  const running = Boolean(activeRun);
  const enabled =
    Boolean(workspace) && Boolean(session) && phase === "idle" && !running;
  const shell = workspace?.profile.shell.toLowerCase() ?? "";
  const isPowerShell = shell.includes("powershell");
  const promptRuntime = isPowerShell
    ? "PS"
    : (workspace?.profile.runtimeName ?? "terminal");
  const promptPath = normalizeDirectoryPath(
    currentDirectory ?? workspace?.profile.workingDirectory ?? "~/workspace",
  );
  const promptSymbol = isPowerShell ? ">" : "$";
  const activeDurationMs = activeRun
    ? Math.max(0, clockMs - activeRun.startedAtMs)
    : 0;
  const directoryHistoryKey = workspace
    ? `${workspace.id}:${workspace.profile.runtime}`
    : undefined;
  const directoryHistory = directoryHistoryKey
    ? directoryHistoryByWorkspace[directoryHistoryKey]
    : undefined;
  const backDirectory =
    directoryHistory && directoryHistory.index > 0
      ? directoryHistory.entries[directoryHistory.index - 1]
      : undefined;
  const forwardDirectory =
    directoryHistory &&
    directoryHistory.index < directoryHistory.entries.length - 1
      ? directoryHistory.entries[directoryHistory.index + 1]
      : undefined;

  useEffect(() => {
    if (!session || !directoryHistoryKey || !promptPath) {
      return;
    }

    setDirectoryHistoryByWorkspace((current) => {
      const existing = current[directoryHistoryKey];
      if (!existing) {
        return {
          ...current,
          [directoryHistoryKey]: initialDirectoryHistory(
            workspace?.profile.workingDirectory ?? promptPath,
            promptPath,
          ),
        };
      }

      const activeEntry = existing.entries[existing.index];
      if (activeEntry && sameDirectory(activeEntry, promptPath)) {
        return current;
      }

      const existingIndex = existing.entries.findIndex((entry) =>
        sameDirectory(entry, promptPath),
      );
      if (existingIndex >= 0) {
        return {
          ...current,
          [directoryHistoryKey]: { ...existing, index: existingIndex },
        };
      }

      const entries = existing.entries.slice(0, existing.index + 1);
      if (!sameDirectory(entries[entries.length - 1], promptPath)) {
        entries.push(promptPath);
      }
      return {
        ...current,
        [directoryHistoryKey]: { entries, index: entries.length - 1 },
      };
    });
  }, [
    directoryHistoryKey,
    promptPath,
    session?.id,
    workspace?.profile.workingDirectory,
  ]);

  useEffect(() => {
    saveDirectoryHistory(directoryHistoryByWorkspace);
  }, [directoryHistoryByWorkspace]);

  function stopSequentialQueue(message?: string) {
    setSequentialQueue(null);
    if (message) {
      setNotice({ level: "caution", text: message });
    }
  }

  useEffect(() => {
    if (!enabled) {
      return;
    }
    const frame = requestAnimationFrame(() => inputRef.current?.focus());
    return () => cancelAnimationFrame(frame);
  }, [enabled, lastFinished?.id]);

  useEffect(() => {
    if (
      phase !== "pending-approval" ||
      !pendingDecision ||
      !pendingCommand ||
      !pendingRequest
    ) {
      return;
    }

    const frame = requestAnimationFrame(() => {
      if (pendingDecision.level === "highRisk") {
        highRiskConfirmationRef.current?.focus();
        return;
      }
      approvalCancelRef.current?.focus();
    });
    return () => cancelAnimationFrame(frame);
  }, [phase, pendingCommand, pendingDecision, pendingRequest]);

  useEffect(() => {
    if (sequentialQueue?.approvalMode !== "pending" || running) {
      return;
    }
    const frame = requestAnimationFrame(() => batchReviewRef.current?.focus());
    return () => cancelAnimationFrame(frame);
  }, [running, sequentialQueue?.approvalMode]);

  useEffect(() => {
    if (!activeRun) {
      return;
    }
    setSessionInput("");
    const frame = requestAnimationFrame(() => sessionInputRef.current?.focus());
    setClockMs(Date.now());
    const timer = window.setInterval(() => setClockMs(Date.now()), 250);
    return () => {
      cancelAnimationFrame(frame);
      window.clearInterval(timer);
    };
  }, [activeRun?.id]);

  async function submitSessionInput(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (!activeRun || sendingSessionInput) {
      return;
    }

    setSendingSessionInput(true);
    try {
      await onSendSessionInput(sessionInput);
      setSessionInput("");
      requestAnimationFrame(() => sessionInputRef.current?.focus());
    } catch (error) {
      setNotice({
        level: "caution",
        text: errorText(
          error,
          "The response could not be sent to the running command.",
        ),
      });
    } finally {
      setSendingSessionInput(false);
    }
  }

  useEffect(() => {
    setCopiedLogRunId(null);
    setCopyingLog(false);
  }, [lastFinished?.id]);

  async function copyFinishedLog() {
    if (!lastFinished || copyingLog) {
      return;
    }

    setCopyingLog(true);
    try {
      const content = await invoke<string>("read_log_file", {
        path: lastFinished.logPath,
      });
      await navigator.clipboard.writeText(content);
      setCopiedLogRunId(lastFinished.id);
    } catch (error) {
      setCopiedLogRunId(null);
      setNotice({
        level: "caution",
        text: errorText(error, "The complete command log could not be copied."),
      });
    } finally {
      setCopyingLog(false);
    }
  }

  useEffect(() => {
    if (
      !lastFinished ||
      !sequentialQueue?.activeRunId ||
      lastFinished.id !== sequentialQueue.activeRunId
    ) {
      return;
    }

    if (!lastFinished.success) {
      stopSequentialQueue(
        lastFinished.stoppedByUser
          ? "Command queue stopped by the user. Remaining commands were not run."
          : undefined,
      );
      return;
    }

    const completed = sequentialQueue.completed + 1;
    const [nextCommand, ...remaining] = sequentialQueue.remaining;
    if (!nextCommand) {
      const total = sequentialQueue.total;
      setSequentialQueue(null);
      setNotice({
        level: "safe",
        text: `Command queue completed. ${total} of ${total} commands succeeded.`,
      });
      return;
    }

    setSequentialQueue({
      current: nextCommand,
      remaining,
      completed,
      total: sequentialQueue.total,
      activeRunId: null,
      approvalMode: sequentialQueue.approvalMode,
    });
    setMessage(nextCommand);
    void planRequest(nextCommand, sequentialQueue.approvalMode).then(
      (accepted) => {
        if (!accepted) {
          stopSequentialQueue();
        }
      },
    );
  }, [lastFinished?.id, sequentialQueue?.activeRunId]);

  useEffect(() => {
    let cancelled = false;

    if (
      !lastFinished ||
      lastFinished.success ||
      lastFinished.stoppedByUser ||
      lastHistoryEntry?.id !== lastFinished.id
    ) {
      setDiagnosis(null);
      return;
    }

    void invoke<CommandFailureDiagnosis>("diagnose_command_failure", {
      path: lastFinished.logPath,
      command: lastHistoryEntry.command,
      exitCode: lastFinished.exitCode,
    })
      .then((result) => {
        if (!cancelled) {
          setDiagnosis(result);
        }
      })
      .catch(() => {
        if (!cancelled) {
          setDiagnosis(null);
        }
      });

    return () => {
      cancelled = true;
    };
  }, [
    lastFinished?.id,
    lastFinished?.success,
    lastFinished?.stoppedByUser,
    lastHistoryEntry?.id,
    lastHistoryEntry?.command,
  ]);

  useEffect(() => {
    if (!recalledRequest) {
      return;
    }
    setPendingRequest(null);
    setPendingCommand(null);
    setPendingDecision(null);
    setInterpretedNote(null);
    setNotice(null);
    setPhase("idle");
    setMessage(recalledRequest.request);
    const frame = requestAnimationFrame(() => inputRef.current?.focus());
    return () => cancelAnimationFrame(frame);
  }, [recalledRequest?.token]);

  useEffect(() => {
    setAzureRecovery(null);
    processedRecoveryRunId.current = null;
    setVmLifecycle(null);
    processedVmDetectionRunId.current = null;
    setVmActionResolution(null);
    processedVmActionRunId.current = null;
    processedVmStatusFollowupRunId.current = null;
  }, [session?.id]);

  useEffect(() => {
    if (
      !lastFinished ||
      !lastFinished.success ||
      !lastHistoryEntry ||
      lastHistoryEntry.id !== lastFinished.id ||
      !/\bterraform\s+apply\b/i.test(lastHistoryEntry.command) ||
      processedVmDetectionRunId.current === lastFinished.id
    ) {
      return;
    }

    processedVmDetectionRunId.current = lastFinished.id;
    let cancelled = false;

    void invoke<CreatedVirtualMachine | null>(
      "detect_created_virtual_machine",
      {
        path: lastFinished.logPath,
      },
    )
      .then((detected) => {
        if (!cancelled && detected) {
          setVmLifecycle(detected);
        }
      })
      .catch(() => {
        // No guided decision if the log cannot be read; the apply already
        // succeeded, so this is purely a missed convenience, not an error
        // worth surfacing to the user.
      });

    return () => {
      cancelled = true;
    };
  }, [
    lastFinished?.id,
    lastFinished?.success,
    lastFinished?.logPath,
    lastHistoryEntry?.id,
    lastHistoryEntry?.command,
  ]);

  function runVmAction(action: VmAction, vm: AzureVirtualMachine) {
    void classifyAndQueue(
      `${VM_ACTION_LABEL[action]} the Azure VM ${vm.name}`,
      `az vm ${action} --name ${shellQuote(vm.name)} --resource-group ${shellQuote(vm.resourceGroup)}`,
    );
  }

  useEffect(() => {
    if (
      !vmActionResolution ||
      vmActionResolution.stage !== "listing" ||
      !lastFinished ||
      lastHistoryEntry?.id !== lastFinished.id ||
      lastHistoryEntry.request !== AZURE_VM_LIST_REQUEST ||
      processedVmActionRunId.current === lastFinished.id
    ) {
      return;
    }

    processedVmActionRunId.current = lastFinished.id;

    if (!lastFinished.success) {
      setVmActionResolution(null);
      setNotice({
        level: "blocked",
        text: "Could not list Azure virtual machines to resolve this request.",
      });
      return;
    }

    const { action, vmName } = vmActionResolution;
    void invoke<AzureVirtualMachine[]>("read_azure_virtual_machines", {
      path: lastFinished.logPath,
    })
      .then((vms) => {
        if (vms.length === 0) {
          setVmActionResolution(null);
          setNotice({
            level: "blocked",
            text: "No Azure virtual machines were found in the active subscription.",
          });
          return;
        }

        // Only one VM exists at all, so there is nothing to disambiguate —
        // use it directly, even if its name does not exactly match what
        // was typed.
        if (vms.length === 1) {
          setVmActionResolution(null);
          runVmAction(action, vms[0]);
          return;
        }

        // No name was typed at all ("deallocate vm" alone) — nothing to
        // narrow the multiple VMs down by, so every one is a candidate.
        const normalizedName = vmName?.toLowerCase();
        const matches = normalizedName
          ? vms.filter((vm) => vm.name.toLowerCase() === normalizedName)
          : [];

        // Multiple VMs exist, but the typed name uniquely identifies one —
        // no need to make the user pick.
        if (matches.length === 1) {
          setVmActionResolution(null);
          runVmAction(action, matches[0]);
          return;
        }

        // Ambiguous: several VMs, and the typed name did not narrow it to
        // exactly one (no match, or the same name appears in more than one
        // resource group). Let the user choose.
        setVmActionResolution((current) =>
          current
            ? {
                ...current,
                stage: "choosing",
                candidates: matches.length > 0 ? matches : vms,
              }
            : null,
        );
      })
      .catch(() => {
        setVmActionResolution(null);
        setNotice({
          level: "blocked",
          text: "Azure virtual machines could not be read from the command result.",
        });
      });
  }, [
    vmActionResolution,
    lastFinished?.id,
    lastFinished?.success,
    lastFinished?.logPath,
    lastHistoryEntry?.id,
    lastHistoryEntry?.request,
  ]);

  useEffect(() => {
    if (
      !lastFinished ||
      !lastHistoryEntry ||
      lastHistoryEntry.id !== lastFinished.id ||
      !VM_START_OR_DEALLOCATE_PATTERN.test(lastHistoryEntry.command) ||
      processedVmStatusFollowupRunId.current === lastFinished.id
    ) {
      return;
    }

    processedVmStatusFollowupRunId.current = lastFinished.id;
    // Runs whether the start/deallocate itself succeeded or failed — the
    // resulting status is informative either way (e.g. a failure because
    // the VM was already in that state is exactly what this reveals).
    void classifyAndQueue(VM_STATUS_CHECK_REQUEST, VM_STATUS_CHECK_COMMAND);
  }, [lastFinished?.id, lastHistoryEntry?.id, lastHistoryEntry?.command]);

  useEffect(() => {
    if (
      !azureRecovery ||
      !lastFinished ||
      lastHistoryEntry?.id !== lastFinished.id ||
      lastHistoryEntry.request !== azureRecovery.actionRequest ||
      processedRecoveryRunId.current === lastFinished.id
    ) {
      return;
    }

    processedRecoveryRunId.current = lastFinished.id;

    if (azureRecovery.stage === "checking") {
      if (!lastFinished.success) {
        setAzureRecovery((current) =>
          current
            ? {
                ...current,
                stage: "deadEnd",
                actionRequest: "",
              }
            : null,
        );
        return;
      }

      void invoke<AzureSubscription[]>("read_azure_subscriptions", {
        path: lastFinished.logPath,
      })
        .then((subscriptions) => {
          setAzureRecovery((current) => {
            if (!current) {
              return null;
            }

            const alternatives = subscriptions.filter(
              (subscription) =>
                subscription.id !== current.failedSubscriptionId,
            );
            if (alternatives.length > 0) {
              return {
                ...current,
                subscriptions,
                stage: "choosing",
                actionRequest: "",
              };
            }
            if (current.reauthAttempts > 0 && subscriptions.length > 0) {
              return {
                ...current,
                subscriptions,
                stage: "retryReady",
                actionRequest: "",
              };
            }
            return {
              ...current,
              subscriptions,
              stage: current.reauthAttempts > 0 ? "deadEnd" : "reauthRequired",
              actionRequest: "",
            };
          });
        })
        .catch((error) => {
          setNotice({
            level: "blocked",
            text: errorText(
              error,
              "Azure subscriptions could not be read from the command result.",
            ),
          });
          setAzureRecovery((current) =>
            current
              ? { ...current, stage: "deadEnd", actionRequest: "" }
              : null,
          );
        });
      return;
    }

    if (azureRecovery.stage === "setting") {
      setAzureRecovery((current) =>
        current
          ? {
              ...current,
              stage: lastFinished.success ? "retryReady" : "reauthRequired",
              actionRequest: "",
            }
          : null,
      );
      return;
    }

    if (azureRecovery.stage === "signingIn") {
      if (!lastFinished.success) {
        setAzureRecovery((current) =>
          current ? { ...current, stage: "deadEnd", actionRequest: "" } : null,
        );
        return;
      }

      const nextRecovery = {
        ...azureRecovery,
        stage: "checking" as const,
        actionRequest: AZURE_SUBSCRIPTION_CHECK_REQUEST,
        reauthAttempts: azureRecovery.reauthAttempts + 1,
      };
      setAzureRecovery(nextRecovery);
      void classifyAndQueue(
        AZURE_SUBSCRIPTION_CHECK_REQUEST,
        AZURE_SUBSCRIPTION_CHECK_COMMAND,
      ).then((accepted) => {
        if (!accepted) {
          setAzureRecovery((current) =>
            current
              ? { ...current, stage: "deadEnd", actionRequest: "" }
              : null,
          );
        }
      });
      return;
    }

    if (azureRecovery.stage === "retrying") {
      if (lastFinished.success) {
        setAzureRecovery(null);
        setDiagnosis(null);
      } else {
        setAzureRecovery((current) =>
          current
            ? {
                ...current,
                stage:
                  current.reauthAttempts > 0 ? "deadEnd" : "reauthRequired",
                actionRequest: "",
              }
            : null,
        );
      }
    }
  }, [
    azureRecovery,
    lastFinished?.id,
    lastFinished?.success,
    lastFinished?.logPath,
    lastHistoryEntry?.id,
    lastHistoryEntry?.request,
  ]);

  async function executeNow(
    request: string,
    command: string,
    decision: RiskDecision,
  ): Promise<boolean> {
    let started = true;
    try {
      const summary = await onRunCommand(request, command, decision);
      setSequentialQueue((current) =>
        current
          ? { ...current, current: command, activeRunId: summary.id }
          : null,
      );
    } catch (error) {
      started = false;
      setSequentialQueue(null);
      setNotice({
        level: "blocked",
        text: errorText(error, "The command could not run."),
      });
    } finally {
      setMessage("");
      setPendingRequest(null);
      setPendingCommand(null);
      setPendingDecision(null);
      setHighRiskConfirmation("");
      setInterpretedNote(null);
      setPhase("idle");
    }
    return started;
  }

  async function classifyAndQueue(
    request: string,
    command: string,
    approvalMode: SequentialApprovalMode = "reviewEach",
  ): Promise<boolean> {
    setNotice(null);
    setInterpretedNote(null);
    setExplainOnly(null);
    setPhase("classifying");

    let decision: RiskDecision;
    try {
      decision = await invoke<RiskDecision>("classify_command_text", {
        command,
      });
    } catch (error) {
      setNotice({
        level: "blocked",
        text: errorText(error, "Could not classify this command."),
      });
      setPhase("idle");
      return false;
    }

    if (decision.level === "safe") {
      return executeNow(request, command, decision);
    }

    if (shouldAutoApproveSequentialCommand(approvalMode, decision.level)) {
      return executeNow(request, command, decision);
    }

    if (decision.level === "blocked") {
      setNotice({
        level: "blocked",
        text: decision.reason,
        editablePath: decision.editablePath ?? undefined,
        externalTerminalCommand: decision.externalTerminal
          ? command
          : undefined,
      });
      setMessage("");
      setPhase("idle");
      return false;
    }

    setPendingCommand(command);
    setPendingRequest(request);
    setPendingDecision(decision);
    setHighRiskConfirmation("");
    setPhase("pending-approval");
    return true;
  }

  async function planRequest(
    typed: string,
    approvalMode: SequentialApprovalMode = "reviewEach",
  ): Promise<boolean> {
    if (!workspace) {
      return false;
    }

    setNotice(null);
    setInterpretedNote(null);
    setExplainOnly(null);
    setPhase("classifying");

    let resolved: ResolvedCommand;
    try {
      resolved = await invoke<ResolvedCommand>("resolve_command", {
        message: typed,
        profile: workspace.profile,
      });
    } catch (error) {
      setNotice({
        level: "blocked",
        text: errorText(error, "Could not interpret this command."),
      });
      setPhase("idle");
      return false;
    }

    if (resolved.source !== "direct" && resolved.note) {
      setInterpretedNote(resolved.note);
    }
    if (resolved.executionMode === "explainOnly") {
      setExplainOnly(resolved);
      setMessage("");
      setPhase("idle");
      return false;
    }

    return classifyAndQueue(typed, resolved.command, approvalMode);
  }

  async function navigateDirectory(target: string, targetIndex: number) {
    if (!session || !directoryHistoryKey || !enabled || navigatingDirectory) {
      return;
    }

    setNavigatingDirectory(true);
    try {
      const workingDirectory = await onNavigateDirectory(target);
      setDirectoryHistoryByWorkspace((current) => {
        const history = current[directoryHistoryKey];
        if (!history || !sameDirectory(workingDirectory, target)) {
          return current;
        }
        return {
          ...current,
          [directoryHistoryKey]: { ...history, index: targetIndex },
        };
      });
      requestAnimationFrame(() => inputRef.current?.focus());
    } catch (error) {
      setNotice({
        level: "blocked",
        text: errorText(error, "Could not navigate to that directory."),
      });
    } finally {
      setNavigatingDirectory(false);
    }
  }

  function startSequentialQueue(approvalMode: SequentialApprovalMode) {
    if (!sequentialQueue || sequentialQueue.approvalMode !== "pending") {
      return;
    }

    const command = sequentialQueue.current;
    setSequentialQueue((current) =>
      current ? { ...current, approvalMode } : null,
    );
    setMessage(command);
    void planRequest(command, approvalMode).then((accepted) => {
      if (!accepted) {
        stopSequentialQueue();
      }
    });
  }

  async function handleSubmit(event: FormEvent) {
    event.preventDefault();
    const typed = message.trim();
    if (!typed) {
      return;
    }

    const helpRequest = sequentialQueue ? null : parseHelpRequest(typed);
    if (helpRequest) {
      setMessage("");
      setNotice(null);
      setInterpretedNote(null);
      setExplainOnly(null);
      setPhase("idle");
      onOpenHelp(helpRequest);
      return;
    }

    if (!enabled || !workspace) {
      return;
    }

    const vmActionRequest = sequentialQueue
      ? null
      : parseVmActionRequest(typed);
    if (vmActionRequest) {
      setMessage("");
      setNotice(null);
      setInterpretedNote(null);
      setExplainOnly(null);
      beginVmActionResolution(vmActionRequest.action, vmActionRequest.vmName);
      return;
    }

    if (sequentialQueue) {
      setSequentialQueue((current) =>
        current
          ? {
              ...current,
              current: typed,
              approvalMode:
                current.approvalMode === "pending"
                  ? "reviewEach"
                  : current.approvalMode,
            }
          : null,
      );
    }
    const approvalMode =
      sequentialQueue?.approvalMode === "autoApproveCaution"
        ? "autoApproveCaution"
        : "reviewEach";
    const accepted = await planRequest(typed, approvalMode);
    if (sequentialQueue && !accepted) {
      stopSequentialQueue();
    }
  }

  function beginAzureRecovery() {
    if (!diagnosis || !lastHistoryEntry) {
      return;
    }

    const recovery: AzureRecovery = {
      originalRequest: lastHistoryEntry.request,
      originalCommand: lastHistoryEntry.command,
      failedSubscriptionId: diagnosis.failedSubscriptionId,
      subscriptions: [],
      stage: "checking",
      actionRequest: AZURE_SUBSCRIPTION_CHECK_REQUEST,
      reauthAttempts: 0,
    };
    processedRecoveryRunId.current = null;
    setAzureRecovery(recovery);
    setDiagnosis(null);
    void classifyAndQueue(
      AZURE_SUBSCRIPTION_CHECK_REQUEST,
      AZURE_SUBSCRIPTION_CHECK_COMMAND,
    ).then((accepted) => {
      if (!accepted) {
        setAzureRecovery((current) =>
          current ? { ...current, stage: "deadEnd", actionRequest: "" } : null,
        );
      }
    });
  }

  function selectAzureSubscription(subscription: AzureSubscription) {
    if (!azureRecovery) {
      return;
    }

    const request = `Switch Azure subscription to ${subscription.id}`;
    setAzureRecovery({
      ...azureRecovery,
      stage: "setting",
      actionRequest: request,
    });
    void planRequest(request).then((accepted) => {
      if (!accepted) {
        setAzureRecovery((current) =>
          current ? { ...current, stage: "choosing", actionRequest: "" } : null,
        );
      }
    });
  }

  function refreshAzureSignIn() {
    if (!azureRecovery) {
      return;
    }

    setAzureRecovery({
      ...azureRecovery,
      stage: "signingIn",
      actionRequest: AZURE_LOGIN_REQUEST,
    });
    void planRequest(AZURE_LOGIN_REQUEST).then((accepted) => {
      if (!accepted) {
        setAzureRecovery((current) =>
          current
            ? { ...current, stage: "reauthRequired", actionRequest: "" }
            : null,
        );
      }
    });
  }

  function retryAzureCommand() {
    if (!azureRecovery) {
      return;
    }

    setAzureRecovery({
      ...azureRecovery,
      stage: "retrying",
      actionRequest: azureRecovery.originalRequest,
    });
    void classifyAndQueue(
      azureRecovery.originalRequest,
      azureRecovery.originalCommand,
    ).then((accepted) => {
      if (!accepted) {
        setAzureRecovery((current) =>
          current
            ? { ...current, stage: "retryReady", actionRequest: "" }
            : null,
        );
      }
    });
  }

  function dismissVmLifecycle() {
    setVmLifecycle(null);
  }

  function shellQuote(value: string): string {
    return `'${value.replace(/'/g, "''")}'`;
  }

  function deallocateVm() {
    if (!vmLifecycle) {
      return;
    }
    const { vmName, resourceGroup } = vmLifecycle;
    setVmLifecycle(null);
    void classifyAndQueue(
      `Deallocate the Azure VM ${vmName}`,
      `az vm deallocate --name ${shellQuote(vmName)} --resource-group ${shellQuote(resourceGroup)}`,
    );
  }

  function destroyVmInfrastructure() {
    if (!vmLifecycle) {
      return;
    }
    setVmLifecycle(null);
    void classifyAndQueue(
      "Destroy the Terraform-managed infrastructure",
      "terraform destroy -auto-approve",
    );
  }

  function beginVmActionResolution(action: VmAction, vmName?: string) {
    processedVmActionRunId.current = null;
    setVmActionResolution({ action, vmName, stage: "listing", candidates: [] });
    void classifyAndQueue(AZURE_VM_LIST_REQUEST, AZURE_VM_LIST_COMMAND).then(
      (accepted) => {
        if (!accepted) {
          setVmActionResolution(null);
        }
      },
    );
  }

  function chooseVmForAction(vm: AzureVirtualMachine) {
    if (!vmActionResolution) {
      return;
    }
    const { action } = vmActionResolution;
    setVmActionResolution(null);
    runVmAction(action, vm);
  }

  function handleCommandPaste(event: ClipboardEvent<HTMLInputElement>) {
    const pasted = event.clipboardData.getData("text");
    if (!/[\r\n]/.test(pasted)) {
      return;
    }

    event.preventDefault();
    const commands = parseSequentialCommands(
      pasted,
      workspace?.profile.shell ?? "",
    );
    if (!commands) {
      setNotice({
        level: "blocked",
        text: "The pasted commands could not be separated safely. Check incomplete Bash \\, PowerShell `, or grouped expressions.",
      });
      return;
    }

    const input = event.currentTarget;
    const start = input.selectionStart ?? message.length;
    const end = input.selectionEnd ?? start;
    const firstCommand = `${message.slice(0, start)}${commands[0]}${message.slice(end)}`;
    setNotice(
      commands.length > 1
        ? {
            level: "safe",
            text: `${commands.length} commands queued. TerminalMate will run one command at a time and stop if any command does not succeed.`,
          }
        : null,
    );
    setMessage(firstCommand);
    if (commands.length > 1) {
      setSequentialQueue({
        current: firstCommand,
        remaining: commands.slice(1),
        completed: 0,
        total: commands.length,
        activeRunId: null,
        approvalMode: "pending",
      });
    } else {
      setSequentialQueue(null);
    }
    requestAnimationFrame(() => {
      const caret = start + commands[0].length;
      inputRef.current?.setSelectionRange(caret, caret);
    });
  }

  function cancelPending() {
    setPendingRequest(null);
    setPendingCommand(null);
    setPendingDecision(null);
    setHighRiskConfirmation("");
    setInterpretedNote(null);
    setPhase("idle");
    stopSequentialQueue(
      sequentialQueue
        ? "Command queue cancelled. Remaining commands were not run."
        : undefined,
    );
    setAzureRecovery((current) => {
      if (!current) {
        return null;
      }
      if (current.stage === "setting") {
        return { ...current, stage: "choosing", actionRequest: "" };
      }
      if (current.stage === "signingIn") {
        return { ...current, stage: "reauthRequired", actionRequest: "" };
      }
      if (current.stage === "retrying") {
        return { ...current, stage: "retryReady", actionRequest: "" };
      }
      return current;
    });
  }

  const placeholder = !workspace
    ? "Add a workspace to begin..."
    : !session
      ? "Start a terminal session to run commands..."
      : running
        ? "A command is running..."
        : "Type a command or describe a task...";

  return (
    <footer className="command-area">
      {sequentialQueue ? (
        <section
          className="sequential-command-queue"
          aria-label="Command queue"
        >
          <div className="sequential-command-queue-header">
            <div>
              <strong>Sequential command queue</strong>
              <span>
                {sequentialQueue.completed}/{sequentialQueue.total} completed -
                one command at a time
              </span>
            </div>
            <button
              type="button"
              onClick={() => {
                setSequentialQueue(null);
                setNotice({
                  level: "caution",
                  text: activeRun
                    ? "Remaining commands cleared. The current command is still running."
                    : "Command queue cleared. The current input was kept.",
                });
              }}
            >
              Clear queue
            </button>
          </div>
          <ol>
            {[sequentialQueue.current, ...sequentialQueue.remaining].map(
              (command, index) => (
                <li
                  className={index === 0 ? "current" : undefined}
                  key={`${index}-${command}`}
                >
                  <span>{sequentialQueue.completed + index + 1}</span>
                  <code>{command}</code>
                </li>
              ),
            )}
          </ol>
          {sequentialQueue.approvalMode === "pending" ? (
            <div
              className="sequential-command-approval"
              role="alertdialog"
              aria-label="Choose command queue approval mode"
            >
              <div>
                <strong>How should this batch be approved?</strong>
                <span>
                  Auto-approve applies only to Caution commands in this pasted
                  batch. High-risk commands still require typed confirmation.
                </span>
              </div>
              <div className="sequential-command-approval-actions">
                <button
                  ref={batchReviewRef}
                  type="button"
                  disabled={!sequentialQueue.current.trim()}
                  onClick={() => startSequentialQueue("reviewEach")}
                >
                  Review each
                </button>
                <button
                  type="button"
                  className="auto-approve-batch"
                  disabled={!sequentialQueue.current.trim()}
                  onClick={() => startSequentialQueue("autoApproveCaution")}
                >
                  Auto-approve Caution
                </button>
              </div>
            </div>
          ) : null}
        </section>
      ) : null}
      {notice ? (
        <div className={`command-notice risk-${notice.level}`} role="status">
          <ShieldX size={14} />
          <span>{notice.text}</span>
          {notice.editablePath ? (
            <button
              className="run-open-log"
              type="button"
              onClick={() =>
                onOpenFileEditor(
                  notice.editablePath as string,
                  currentDirectory ?? workspace?.profile.workingDirectory ?? "",
                )
              }
            >
              Open in built-in editor
            </button>
          ) : null}
          {notice.externalTerminalCommand && workspace ? (
            <button
              className="run-open-log"
              type="button"
              onClick={() =>
                onOpenExternalTerminal(
                  currentDirectory ?? workspace.profile.workingDirectory,
                  workspace.profile.shell,
                  notice.externalTerminalCommand as string,
                )
              }
            >
              Open in a new terminal window
            </button>
          ) : null}
        </div>
      ) : null}

      {interpretedNote ? (
        <div className="command-interpreted-note" role="status">
          {interpretedNote}
        </div>
      ) : null}

      {explainOnly ? (
        <section
          className="command-explain-only"
          aria-label="Explain-only command"
        >
          <div className="command-approval-summary">
            <ShieldAlert size={15} />
            <span className="risk-badge">EXPLAIN ONLY</span>
            <strong>Manual action required</strong>
          </div>
          <p>{explainOnly.explanation}</p>
          <code className="command-approval-text">{explainOnly.command}</code>
          <div className="command-approval-actions">
            <button
              type="button"
              className="approval-cancel"
              onClick={() => setExplainOnly(null)}
            >
              Dismiss
            </button>
            <button
              type="button"
              className="explain-copy"
              title="Copy command"
              onClick={() =>
                void navigator.clipboard.writeText(explainOnly.command)
              }
            >
              <Copy size={13} />
              Copy command
            </button>
          </div>
        </section>
      ) : null}

      {activeRun ? (
        <div className="command-run-status running" role="status">
          <span className="run-status-label">
            <span className="run-status-dot" />
            Running
          </span>
          <span className="run-duration">
            <Clock3 size={12} />
            {formatDuration(activeDurationMs)}
          </span>
          <code className="command-approval-text">{activeRun.command}</code>
          <button
            type="button"
            className="run-stop"
            onClick={() => void onStopCommand()}
          >
            <Square size={12} />
            Stop
          </button>
          <form className="session-input-form" onSubmit={submitSessionInput}>
            <label className="sr-only" htmlFor="running-command-response">
              Response to running command
            </label>
            <input
              id="running-command-response"
              ref={sessionInputRef}
              value={sessionInput}
              autoComplete="off"
              spellCheck={false}
              placeholder="Respond to the prompt, for example Y or 1"
              onChange={(event) => setSessionInput(event.target.value)}
            />
            <button
              type="submit"
              title="Send response to the running command"
              aria-label="Send response to the running command"
              disabled={sendingSessionInput}
            >
              <CornerDownLeft size={14} />
            </button>
          </form>
        </div>
      ) : lastFinished ? (
        <div
          className={`command-run-status ${lastFinished.stoppedByUser ? "stopped" : lastFinished.success ? "succeeded" : "failed"}`}
          role="status"
        >
          <span className="run-status-label">
            {lastFinished.stoppedByUser
              ? "Stopped"
              : lastFinished.success
                ? "Succeeded"
                : `Failed (exit ${lastFinished.exitCode})`}
          </span>
          <span className="run-duration">
            <Clock3 size={12} />
            {formatDuration(lastFinished.durationMs)}
          </span>
          <button
            type="button"
            className="run-open-log"
            onClick={() => void onOpenLog(lastFinished.logPath)}
          >
            <FileText size={12} />
            Open log
          </button>
          <button
            type="button"
            className="run-open-log"
            title="Copy the complete command log"
            disabled={copyingLog}
            onClick={() => void copyFinishedLog()}
          >
            <Copy size={12} />
            {copyingLog
              ? "Copying..."
              : copiedLogRunId === lastFinished.id
                ? "Copied"
                : "Copy log"}
          </button>
        </div>
      ) : null}

      {azureRecovery ? (
        <section
          className="command-recovery azure-recovery"
          aria-label="Azure subscription recovery"
        >
          <div className="command-recovery-heading">
            <CircleHelp size={15} />
            <div>
              <span>Guided recovery</span>
              <strong>Restore Azure subscription access</strong>
            </div>
          </div>

          {azureRecovery.stage === "checking" ? (
            <p>
              Checking the subscriptions available to the active Azure
              account...
            </p>
          ) : null}
          {azureRecovery.stage === "choosing" ? (
            <p>
              Choose an accessible subscription. TerminalMate will ask for
              approval before changing the active Azure context.
            </p>
          ) : null}
          {azureRecovery.stage === "setting" ? (
            <p>Review and approve the Azure subscription change below.</p>
          ) : null}
          {azureRecovery.stage === "reauthRequired" ? (
            <p>
              Azure returned only the subscription it just rejected. Refresh the
              Azure sign-in before retrying; selecting the same stale context
              again would create a loop.
            </p>
          ) : null}
          {azureRecovery.stage === "signingIn" ? (
            <p>
              Approve the device-code sign-in below. TerminalMate will check
              subscriptions again after Azure finishes authentication.
            </p>
          ) : null}
          {azureRecovery.stage === "retryReady" ? (
            <p>
              Azure context is ready for another attempt. The original command
              remains High Risk and requires a fresh RUN confirmation.
            </p>
          ) : null}
          {azureRecovery.stage === "retrying" ? (
            <p>Review and confirm the original Azure command below.</p>
          ) : null}
          {azureRecovery.stage === "deadEnd" ? (
            <p>
              Azure still cannot use an accessible subscription. Confirm the
              account and tenant in the Azure portal or ask the subscription
              administrator to restore access before trying again.
            </p>
          ) : null}

          {azureRecovery.subscriptions.length > 0 ? (
            <div className="azure-subscription-list">
              {azureRecovery.subscriptions.map((subscription) => {
                const rejected =
                  subscription.id === azureRecovery.failedSubscriptionId;
                return (
                  <div className="azure-subscription-row" key={subscription.id}>
                    <div>
                      <strong>{subscription.name}</strong>
                      <code>{subscription.id}</code>
                      <small>Tenant {subscription.tenantId}</small>
                    </div>
                    <div className="azure-subscription-actions">
                      {subscription.isDefault ? <span>Default</span> : null}
                      {rejected ? (
                        <span className="rejected">Rejected</span>
                      ) : null}
                      {azureRecovery.stage === "choosing" && !rejected ? (
                        <button
                          type="button"
                          className="command-recovery-action"
                          onClick={() => selectAzureSubscription(subscription)}
                        >
                          Use subscription
                        </button>
                      ) : null}
                    </div>
                  </div>
                );
              })}
            </div>
          ) : null}

          <code className="azure-recovery-original">
            Original: {azureRecovery.originalCommand}
          </code>

          <div className="azure-recovery-actions">
            {azureRecovery.stage === "reauthRequired" ? (
              <button
                type="button"
                className="command-recovery-action"
                onClick={refreshAzureSignIn}
              >
                Refresh Azure sign-in
              </button>
            ) : null}
            {azureRecovery.stage === "retryReady" ? (
              <button
                type="button"
                className="command-recovery-action"
                onClick={retryAzureCommand}
              >
                Review and retry original command
              </button>
            ) : null}
            {azureRecovery.stage === "deadEnd" ? (
              <button
                type="button"
                className="command-recovery-action"
                onClick={() => setAzureRecovery(null)}
              >
                Close recovery
              </button>
            ) : null}
          </div>
        </section>
      ) : diagnosis ? (
        <section className="command-recovery" aria-label="Recovery guidance">
          <div className="command-recovery-heading">
            <CircleHelp size={15} />
            <div>
              <span>Recovery guidance</span>
              <strong>{diagnosis.title}</strong>
            </div>
          </div>
          <p>{diagnosis.explanation}</p>
          <ol>
            {diagnosis.nextSteps.map((step) => (
              <li key={step}>{step}</li>
            ))}
          </ol>
          {diagnosis.suggestedRequest ? (
            <button
              type="button"
              className="command-recovery-action"
              onClick={
                diagnosis.kind === "azureSubscriptionNotFound"
                  ? beginAzureRecovery
                  : () => {
                      setMessage(diagnosis.suggestedRequest ?? "");
                      setDiagnosis(null);
                      requestAnimationFrame(() => inputRef.current?.focus());
                    }
              }
            >
              {diagnosis.kind === "azureSubscriptionNotFound"
                ? "Check subscriptions and continue"
                : diagnosis.kind === "recursiveSearchAccessDenied"
                  ? "Retry and skip inaccessible folders"
                  : `Try: ${diagnosis.suggestedRequest}`}
            </button>
          ) : null}
        </section>
      ) : vmLifecycle ? (
        <section
          className="command-recovery vm-lifecycle"
          aria-label="Guided VM lifecycle decision"
        >
          <div className="command-recovery-heading">
            <CircleHelp size={15} />
            <div>
              <span>Guided decision</span>
              <strong>Decide what happens to {vmLifecycle.vmName}</strong>
            </div>
          </div>
          <p>
            Terraform created {vmLifecycle.vmName} in{" "}
            {vmLifecycle.resourceGroup}. Worth deciding now rather than leaving
            it running by accident.
          </p>
          <div className="azure-recovery-actions">
            <button
              type="button"
              className="command-recovery-action"
              onClick={dismissVmLifecycle}
            >
              Leave it running for now
            </button>
            <button
              type="button"
              className="command-recovery-action"
              onClick={deallocateVm}
            >
              Deallocate (stop billing, keep it)
            </button>
            <button
              type="button"
              className="command-recovery-action recommended"
              onClick={destroyVmInfrastructure}
            >
              Destroy with Terraform (recommended)
            </button>
          </div>
        </section>
      ) : vmActionResolution?.stage === "choosing" ? (
        <section
          className="command-recovery"
          aria-label={`Choose a virtual machine to ${vmActionResolution.action}`}
        >
          <div className="command-recovery-heading">
            <CircleHelp size={15} />
            <div>
              <span>Guided decision</span>
              <strong>
                {vmActionResolution.vmName
                  ? `Which VM did you mean by "${vmActionResolution.vmName}"?`
                  : `Which VM do you want to ${vmActionResolution.action}?`}
              </strong>
            </div>
          </div>
          <p>
            Multiple virtual machines were found. Choose the one to{" "}
            {vmActionResolution.action}.
          </p>
          <div className="azure-subscription-list">
            {vmActionResolution.candidates.map((vm) => (
              <div
                className="azure-subscription-row"
                key={`${vm.resourceGroup}/${vm.name}`}
              >
                <div>
                  <strong>{vm.name}</strong>
                  <small>
                    Resource group {vm.resourceGroup}
                    {vm.powerState ? ` · ${vm.powerState}` : ""}
                  </small>
                </div>
                <div className="azure-subscription-actions">
                  <button
                    type="button"
                    className="command-recovery-action"
                    onClick={() => chooseVmForAction(vm)}
                  >
                    {VM_ACTION_LABEL[vmActionResolution.action]} this VM
                  </button>
                </div>
              </div>
            ))}
          </div>
          <div className="azure-recovery-actions">
            <button
              type="button"
              className="command-recovery-action"
              onClick={() => setVmActionResolution(null)}
            >
              Cancel
            </button>
          </div>
        </section>
      ) : null}

      {pendingDecision && pendingCommand && pendingRequest ? (
        <div
          className={`command-approval risk-${pendingDecision.level}`}
          role="alertdialog"
        >
          <div className="command-approval-summary">
            <ShieldAlert size={15} />
            <span className="risk-badge">
              {RISK_LABEL[pendingDecision.level]}
            </span>
            <span className="command-approval-reason">
              {pendingDecision.reason}
            </span>
          </div>
          <code className="command-approval-text">{pendingCommand}</code>
          {pendingDecision.level === "highRisk" ? (
            <label className="high-risk-confirmation">
              <span>Type RUN to confirm this high-risk command</span>
              <input
                ref={highRiskConfirmationRef}
                value={highRiskConfirmation}
                onChange={(event) =>
                  setHighRiskConfirmation(event.target.value)
                }
                autoCapitalize="characters"
                autoComplete="off"
                spellCheck={false}
              />
            </label>
          ) : null}
          <div className="command-approval-actions">
            <button
              ref={approvalCancelRef}
              type="button"
              className="approval-cancel"
              onClick={cancelPending}
            >
              Cancel
            </button>
            <button
              type="button"
              className="approval-approve"
              disabled={
                pendingDecision.level === "highRisk" &&
                highRiskConfirmation !== "RUN"
              }
              onClick={() =>
                void executeNow(pendingRequest, pendingCommand, pendingDecision)
              }
            >
              {pendingDecision.level === "highRisk"
                ? "Confirm and run"
                : "Approve and run"}
            </button>
          </div>
        </div>
      ) : null}

      <form
        className="command-bar"
        onSubmit={(event) => void handleSubmit(event)}
      >
        <span className="directory-navigation" aria-label="Directory history">
          <button
            type="button"
            onClick={() =>
              backDirectory &&
              void navigateDirectory(
                backDirectory,
                (directoryHistory?.index ?? 0) - 1,
              )
            }
            disabled={!enabled || navigatingDirectory || !backDirectory}
            title={
              backDirectory
                ? `Back to ${backDirectory}`
                : "No previous directory"
            }
            aria-label={
              backDirectory
                ? `Back to ${backDirectory}`
                : "No previous directory"
            }
          >
            <ArrowLeft size={14} />
          </button>
          <button
            type="button"
            onClick={() =>
              forwardDirectory &&
              void navigateDirectory(
                forwardDirectory,
                (directoryHistory?.index ?? -1) + 1,
              )
            }
            disabled={!enabled || navigatingDirectory || !forwardDirectory}
            title={
              forwardDirectory
                ? `Forward to ${forwardDirectory}`
                : "No next directory"
            }
            aria-label={
              forwardDirectory
                ? `Forward to ${forwardDirectory}`
                : "No next directory"
            }
          >
            <ArrowRight size={14} />
          </button>
        </span>
        <span className="terminal-prompt" aria-hidden="true" title={promptPath}>
          <span className="terminal-prompt-runtime">{promptRuntime}</span>
          <span className="terminal-prompt-path"> {promptPath}</span>
          <span className="terminal-prompt-symbol">{promptSymbol}</span>
        </span>
        <input
          ref={inputRef}
          value={pendingCommand ?? message}
          onPaste={handleCommandPaste}
          onChange={(event) => {
            const value = event.target.value;
            setNotice(null);
            setInterpretedNote(null);
            setMessage(value);
            if (sequentialQueue?.approvalMode === "pending") {
              setSequentialQueue((current) =>
                current?.approvalMode === "pending"
                  ? { ...current, current: value }
                  : current,
              );
            }
          }}
          disabled={!enabled}
          placeholder={placeholder}
          aria-label="Command request"
          autoCapitalize="off"
          autoComplete="off"
          spellCheck={false}
          autoFocus
        />
        <span
          className="command-policy"
          title="Every command is risk-classified before it runs"
        >
          {phase === "classifying" ? "CHECKING..." : "PLAN FIRST"}
        </span>
        <button
          className="submit-command"
          type="submit"
          disabled={!enabled || message.trim().length === 0}
          aria-label="Classify and run command"
          title="Classify and run command"
        >
          <CornerDownLeft size={16} />
        </button>
      </form>
    </footer>
  );
}
