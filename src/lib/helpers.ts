// Extract the ID part from a SurrealDB RecordId.
// 3.x format: { table: "user", key: { String: "abc" } }
// 2.x format: { tb:    "user", id:  { String: "abc" } }  (kept for compat)
export function sid(thing: any): string {
  if (!thing) return '';
  if (typeof thing === 'string') {
    const i = thing.indexOf(':');
    const id = i >= 0 ? thing.slice(i + 1) : thing;
    return id.replace(/[⟨⟩]/g, '');
  }
  // 3.x: key field (may be nested variant or plain string)
  const key = thing?.key ?? thing?.id;
  if (typeof key === 'string') return key.replace(/[⟨⟩]/g, '');
  if (key?.String)                    return key.String;
  if (key?.Uuid)                      return key.Uuid;
  if (key?.Number !== undefined)      return String(key.Number);
  return JSON.stringify(thing);
}

// Return canonical "table:id" string for equality checks
export function full(thing: any): string {
  if (typeof thing === 'string') return thing;
  const table = thing?.table ?? thing?.tb ?? '';
  return `${table}:${sid(thing)}`;
}

export function fmt(ts: string): string {
  return new Date(ts).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
}

export async function cmd<T>(name: string, args?: Record<string, unknown>): Promise<T> {
  const { invoke } = await import('@tauri-apps/api/core');
  return invoke<T>(name, args);
}
