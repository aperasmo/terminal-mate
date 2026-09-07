export const HELP_CATEGORIES = [
  "All",
  "Everyday",
  "AWS",
  "Azure",
  "GitHub",
  "Docker",
  "Git",
  "PowerShell",
  "Security",
  "Google Cloud",
  "Terraform",
  "SSH",
] as const;

export type HelpCategory = (typeof HELP_CATEGORIES)[number];

export interface HelpRequest {
  category: HelpCategory;
  query: string;
  requestedTopic?: string;
  unsupportedTopic?: string;
}

const CATEGORY_ALIASES: Record<string, HelpCategory> = {
  all: "All",
  commands: "All",
  everything: "All",
  general: "Everyday",
  everyday: "Everyday",
  basic: "Everyday",
  basics: "Everyday",
  aws: "AWS",
  "amazon web services": "AWS",
  azure: "Azure",
  az: "Azure",
  github: "GitHub",
  gh: "GitHub",
  "github cli": "GitHub",
  docker: "Docker",
  containers: "Docker",
  git: "Git",
  powershell: "PowerShell",
  pwsh: "PowerShell",
  windows: "PowerShell",
  security: "Security",
  secrets: "Security",
  gitleaks: "Security",
  leaks: "Security",
  gcp: "Google Cloud",
  gcloud: "Google Cloud",
  google: "Google Cloud",
  "google cloud": "Google Cloud",
  terraform: "Terraform",
  tf: "Terraform",
  ssh: "SSH",
  "secure shell": "SSH",
};

function cleanTopic(value: string) {
  return value
    .trim()
    .replace(/^[\s:,-]+|[\s?.!]+$/g, "")
    .replace(/\s+/g, " ");
}

export function parseHelpRequest(value: string): HelpRequest | null {
  const normalized = value.trim();
  const prefixMatch = normalized.match(
    /^(?:help|show\s+help)(?:\s+(?:for|with|on))?(?:\s+(.+))?$/i,
  );
  const suffixMatch = normalized.match(/^(.+?)\s+help$/i);
  const rawTopic = prefixMatch?.[1] ?? suffixMatch?.[1];

  if (!prefixMatch && !suffixMatch) {
    return null;
  }

  if (!rawTopic) {
    return { category: "All", query: "" };
  }

  const topic = cleanTopic(rawTopic);
  const category = CATEGORY_ALIASES[topic.toLowerCase()];
  if (category) {
    return {
      category,
      query: "",
      requestedTopic: topic,
    };
  }

  return {
    category: "All",
    query: topic,
    requestedTopic: topic,
    unsupportedTopic: topic,
  };
}

export function defaultHelpRequest(): HelpRequest {
  return { category: "All", query: "" };
}
