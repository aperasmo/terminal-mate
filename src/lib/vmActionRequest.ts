export type VmAction = "start" | "deallocate";

export interface VmActionRequest {
  action: VmAction;
  vmName?: string;
}

/**
 * Recognizes "start az vm [name]" / "deallocate azure vm [name]" with no
 * resource group given — the existing sidecar-matched `azure_vm_start`/
 * `azure_vm_deallocate` intents still require both vm_name and
 * resource_group ("start az vm X in Y") and are left untouched. This is a
 * narrower, frontend-only phrase (mirroring `parseHelpRequest`'s existing
 * precedent of a local match that never reaches the sidecar) for when the
 * caller does not want to look up or type the resource group themselves —
 * the name is optional too ("start az vm" / "deallocate the azure vm"
 * alone is enough); the caller resolves whichever pieces are missing via
 * `az vm list`. Anchored to end-of-string so "... in <resource group>"
 * still falls through unmatched to the normal sidecar-resolved path.
 *
 * The "az"/"azure" marker is required, not optional — same reasoning as
 * every sidecar Azure pattern: it is what keeps "start vm X" free for a
 * future "start aws vm X" to mean something else, instead of this phrase
 * silently assuming Azure.
 */
const VM_ACTION_PATTERN =
  /^(start|deallocate)\s+(?:the\s+|my\s+)?(?:az|azure)\s+(?:vm|virtual\s+machine)(?:\s+(\S+))?\s*$/i;

export function parseVmActionRequest(value: string): VmActionRequest | null {
  const match = value.trim().match(VM_ACTION_PATTERN);
  if (!match) {
    return null;
  }
  return {
    action: match[1].toLowerCase() as VmAction,
    vmName: match[2],
  };
}
