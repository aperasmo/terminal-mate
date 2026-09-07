const MASK = "********";

const SENSITIVE_KEY =
  "(?:password|passwd|pwd|secret|token|api[_-]?key|openai_api_key|database_url|connection_string|aws_access_key_id|aws_secret_access_key)";

const patterns: Array<[RegExp, string]> = [
  [
    new RegExp(`("[^"]*${SENSITIVE_KEY}[^"]*"\\s*:\\s*")([^"]+)(")`, "gi"),
    `$1${MASK}$3`,
  ],
  [
    new RegExp(`('[^']*${SENSITIVE_KEY}[^']*'\\s*:\\s*')([^']+)(')`, "gi"),
    `$1${MASK}$3`,
  ],
  [
    new RegExp(`(\\b${SENSITIVE_KEY}\\b\\s*=\\s*)([^\\s,;]+)`, "gi"),
    `$1${MASK}`,
  ],
  [
    /(--(?:password|token|api-key|secret|access-key)\s+)(?:"[^"]*"|'[^']*'|\S+)/gi,
    `$1${MASK}`,
  ],
  [
    /((?:postgres(?:ql)?|mysql|mongodb(?:\+srv)?|redis):\/\/[^:\s"'@]+:)([^@\s"']+)(@)/gi,
    `$1${MASK}$3`,
  ],
  [/\bsk-(?:proj-)?[A-Za-z0-9_-]{16,}\b/g, MASK],
  [/\bAKIA[A-Z0-9]{16}\b/g, MASK],
  [/(\bBearer\s+)[A-Za-z0-9._~+\/-]{12,}/gi, `$1${MASK}`],
];

export interface MaskedSensitiveOutput {
  text: string;
  masked: boolean;
}

export function maskSensitiveOutput(value: string): MaskedSensitiveOutput {
  let text = value;
  for (const [pattern, replacement] of patterns) {
    text = text.replace(pattern, replacement);
  }
  return { text, masked: text !== value };
}

export function containsSensitiveOutput(value: string): boolean {
  return maskSensitiveOutput(value).masked;
}
