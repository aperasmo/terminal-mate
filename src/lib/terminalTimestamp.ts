function padTimestampPart(value: number) {
  return value.toString().padStart(2, "0");
}

export function formatTerminalTimestamp(receivedAtMs: number) {
  const timestamp = new Date(receivedAtMs);
  return [
    `${padTimestampPart(timestamp.getDate())}/${padTimestampPart(timestamp.getMonth() + 1)}/${timestamp.getFullYear()}`,
    `${padTimestampPart(timestamp.getHours())}:${padTimestampPart(timestamp.getMinutes())}:${padTimestampPart(timestamp.getSeconds())}`,
  ].join(" ");
}
