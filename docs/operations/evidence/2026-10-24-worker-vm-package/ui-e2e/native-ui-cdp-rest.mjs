const targets = await fetch("http://127.0.0.1:9222/json").then((r) => r.json());
const page = targets.find((t) => t.type === "page" && t.url.includes("127.0.0.1:4173"));
const ws = new WebSocket(page.webSocketDebuggerUrl);
let id = 0; const pending = new Map();
function send(method, params = {}) { const i = ++id; ws.send(JSON.stringify({ id: i, method, params })); return new Promise((r) => pending.set(i, r)); }
ws.onmessage = (e) => {
  const m = JSON.parse(e.data);
  if (m.id && pending.has(m.id)) { pending.get(m.id)(m); pending.delete(m.id); return; }
  if (m.method === "Page.javascriptDialogOpening") void send("Page.handleJavaScriptDialog", { accept: true });
};
await new Promise((res, rej) => { ws.onopen = res; ws.onerror = rej; });
await send("Runtime.enable");
await send("Page.enable");
const clickImport = await send("Runtime.evaluate", { expression: `(() => { const b=[...document.querySelectorAll('button')].find(x=>x.textContent.includes('扫描安装')); if(!b) return 'no-button'; b.click(); return 'clicked'; })()`, returnByValue: true });
console.log("IMPORT", clickImport.result?.result?.value);
await new Promise((r) => setTimeout(r, 1000));
const afterImport = await send("Runtime.evaluate", { expression: "document.body.innerText", returnByValue: true });
console.log("IMPORT_DONE", String(afterImport.result?.result?.value ?? "").includes("安装扫描完成"));
const clickUninstall = await send("Runtime.evaluate", { expression: `(() => { const b=[...document.querySelectorAll('button')].find(x=>x.textContent.includes('卸载')); if(!b) return 'no-button'; b.click(); return 'clicked'; })()`, returnByValue: true });
console.log("UNINSTALL", clickUninstall.result?.result?.value);
await new Promise((r) => setTimeout(r, 1200));
const afterUninstall = await send("Runtime.evaluate", { expression: "document.body.innerText", returnByValue: true });
console.log("UNINSTALL_DONE", String(afterUninstall.result?.result?.value ?? "").includes("uninstall 完成"));
ws.close();
