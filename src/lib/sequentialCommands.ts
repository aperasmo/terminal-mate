export type SequentialApprovalMode =
  "pending" | "reviewEach" | "autoApproveCaution";

export function shouldAutoApproveSequentialCommand(
  mode: SequentialApprovalMode,
  riskLevel: string,
) {
  return mode === "autoApproveCaution" && riskLevel === "caution";
}

function updatePowerShellGroupStack(line: string, stack: string[]) {
  let quote: "'" | '"' | null = null;

  for (let index = 0; index < line.length; index += 1) {
    const character = line[index];
    if (quote) {
      if (character === "`" && quote === '"') {
        index += 1;
      } else if (character === quote) {
        if (quote === "'" && line[index + 1] === "'") {
          index += 1;
        } else {
          quote = null;
        }
      }
      continue;
    }

    if (character === "'" || character === '"') {
      quote = character;
      continue;
    }
    if ("([{".includes(character)) {
      stack.push(character);
      continue;
    }
    if (")]}".includes(character)) {
      const expected = character === ")" ? "(" : character === "]" ? "[" : "{";
      if (stack.pop() !== expected) {
        return false;
      }
    }
  }

  return quote === null;
}

function isPowerShellGroupedIterationScript(value: string) {
  const lines = value
    .trim()
    .split(/\r?\n/)
    .map((line) => line.trim())
    .filter(Boolean);
  const stack: string[] = [];
  let foundIteration = false;
  let openedIterationGroup = false;
  let completedIteration = false;
  let awaitingPipelineCommand = false;

  for (const line of lines) {
    const depthBefore = stack.length;
    if (completedIteration) {
      return false;
    }

    if (!foundIteration && depthBefore === 0) {
      if (
        /^foreach\s*\(/i.test(line) ||
        /(?:^|\|)\s*ForEach-Object\b[^{}]*\{/i.test(line)
      ) {
        foundIteration = true;
        awaitingPipelineCommand = false;
      } else if (!/^\$[A-Za-z_][\w:]*\s*=/.test(line)) {
        if (endsWithPowerShellPipeline(line)) {
          awaitingPipelineCommand = true;
        } else if (
          !awaitingPipelineCommand ||
          !/^ForEach-Object\b[^{}]*\{/i.test(line)
        ) {
          return false;
        } else {
          foundIteration = true;
          awaitingPipelineCommand = false;
        }
      }
    }

    if (!updatePowerShellGroupStack(line, stack)) {
      return false;
    }
    if (foundIteration && stack.length > 0) {
      openedIterationGroup = true;
    }
    if (foundIteration && openedIterationGroup && stack.length === 0) {
      completedIteration = true;
    }
  }

  return foundIteration && completedIteration;
}

function isPowerShellScopedEnvironmentScript(value: string) {
  const lines = value
    .trim()
    .split(/\r?\n/)
    .map((line) => line.trim())
    .filter(Boolean);

  if (lines.length < 2 || !/^\$env:[A-Za-z_][\w]*\s*=/.test(lines[0])) {
    return false;
  }

  return lines.slice(1).some(
    (line) => !/^\$env:[A-Za-z_][\w]*\s*=/.test(line),
  );
}

function isPowerShellVariableBackedScript(value: string) {
  const lines = value
    .trim()
    .split(/\r?\n/)
    .map((line) => line.trim())
    .filter(Boolean);

  // Loop scripts use the stricter block-aware parser, which rejects any
  // unrelated command appended after the completed loop.
  if (
    lines.some(
      (line) =>
        /^foreach\s*\(/i.test(line) || /\|\s*ForEach-Object\s*\{/i.test(line),
    )
  ) {
    return false;
  }

  const assignedVariables = new Map<string, number>();

  for (const [index, line] of lines.entries()) {
    const assignment = line.match(/^\$([A-Za-z_][\w]*)\s*=/);
    if (assignment) {
      assignedVariables.set(assignment[1].toLowerCase(), index);
    }
  }

  if (assignedVariables.size === 0) {
    return false;
  }

  return [...assignedVariables].some(([name, assignmentIndex]) => {
    const reference = new RegExp(`\\$${name}\\b`, "i");
    return lines
      .slice(assignmentIndex + 1)
      .some((line) => reference.test(line));
  });
}

function endsWithPowerShellPipeline(line: string) {
  const trimmed = line.trimEnd();
  if (!trimmed.endsWith("|") || trimmed.endsWith("||")) {
    return false;
  }

  let backtickCount = 0;
  for (let index = trimmed.length - 2; index >= 0; index -= 1) {
    if (trimmed[index] !== "`") {
      break;
    }
    backtickCount += 1;
  }
  return backtickCount % 2 === 0;
}

function normalizeContinuedPaste(value: string, shell: string) {
  const powershell = ["powershell", "pwsh"].includes(shell.toLowerCase());
  const groupedValue =
    powershell &&
    (isPowerShellGroupedIterationScript(value) ||
      isPowerShellScopedEnvironmentScript(value) ||
      isPowerShellVariableBackedScript(value))
      ? `& {\n${value.trim()}\n}`
      : value.trim();
  const lines = groupedValue.split(/\r?\n/);
  const groupStack: string[] = [];
  let normalized = "";

  for (let index = 0; index < lines.length; index += 1) {
    const line = lines[index].trim();
    const markerContinued = line.endsWith("\\") || line.endsWith("`");
    const pipelineContinued = powershell && endsWithPowerShellPipeline(line);
    const continued = markerContinued || pipelineContinued;
    const content = markerContinued ? line.slice(0, -1).trimEnd() : line;

    if (powershell && !updatePowerShellGroupStack(content, groupStack)) {
      return null;
    }
    normalized += content;

    if (index < lines.length - 1) {
      if (!continued && (!powershell || groupStack.length === 0)) {
        return null;
      }
      normalized += continued || !powershell ? " " : "\n";
    }
  }

  if (powershell && groupStack.length > 0) {
    return null;
  }
  return normalized;
}

export function parseSequentialCommands(
  value: string,
  shell: string,
): string[] | null {
  const normalizedWhole = normalizeContinuedPaste(value, shell);
  if (normalizedWhole) {
    return [normalizedWhole];
  }

  const powershell = ["powershell", "pwsh"].includes(shell.toLowerCase());
  const commands: string[] = [];
  const groupStack: string[] = [];
  let currentLines: string[] = [];

  const flushCurrent = () => {
    if (currentLines.length === 0) {
      return true;
    }
    const normalized = normalizeContinuedPaste(currentLines.join("\n"), shell);
    if (!normalized) {
      return false;
    }
    commands.push(normalized);
    currentLines = [];
    return true;
  };

  for (const rawLine of value.split(/\r?\n/)) {
    const line = rawLine.trim();
    if (!line) {
      if (groupStack.length === 0 && !flushCurrent()) {
        return null;
      }
      continue;
    }

    currentLines.push(line);
    const markerContinued = line.endsWith("\\") || line.endsWith("`");
    const pipelineContinued = powershell && endsWithPowerShellPipeline(line);
    const continued = markerContinued || pipelineContinued;
    const content = markerContinued ? line.slice(0, -1).trimEnd() : line;
    if (powershell && !updatePowerShellGroupStack(content, groupStack)) {
      return null;
    }

    if (!continued && (!powershell || groupStack.length === 0)) {
      if (!flushCurrent()) {
        return null;
      }
    }
  }

  if (groupStack.length > 0 || !flushCurrent()) {
    return null;
  }
  return commands.length > 0 ? commands : null;
}
