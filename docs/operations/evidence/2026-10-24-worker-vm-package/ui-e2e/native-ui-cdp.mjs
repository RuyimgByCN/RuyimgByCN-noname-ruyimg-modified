const targets = await fetch("http://127.0.0.1:9222/json").then((r) => r.json());
const page = targets.find((t) => t.type === "page" && t.url.includes("127.0.0.1:4173"));
if (!page) throw new Error("page target not found: " + JSON.stringify(targets.map((t) => [t.type, t.url])));
const ws = new WebSocket(page.webSocketDebuggerUrl);
let id = 0;
const pending = new Map();
ws.onmessage = (event) => {
  const msg = JSON.parse(event.data);
  if (msg.id && pending.has(msg.id)) {
    pending.get(msg.id)(msg);
    pending.delete(msg.id);
  }
};
await new Promise((resolve, reject) => { ws.onopen = resolve; ws.onerror = reject; });
function send(method, params = {}) {
  const callId = ++id;
  ws.send(JSON.stringify({ id: callId, method, params }));
  return new Promise((resolve) => pending.set(callId, resolve));
}
await send("Runtime.enable");
for (let i = 0; i < 30; i++) {
  const text = await send("Runtime.evaluate", { expression: "document.body.innerText", returnByValue: true });
  if (String(text.result?.result?.value ?? "").includes("browser-probe")) break;
  await new Promise((r) => setTimeout(r, 200));
}
const before = await send("Runtime.evaluate", { expression: "document.body.innerText", returnByValue: true });
console.log("BEFORE_HAS_PACKAGE", String(before.result?.result?.value ?? "").includes("browser-probe"));
const click = await send("Runtime.evaluate", {
  expression: `(() => { const b=[...document.querySelectorAll('button')].find(x=>x.textContent.includes('停用')); if(!b) return 'no-button'; b.click(); return 'clicked'; })()`,
  returnByValue: true,
});
console.log("CLICK", click.result?.result?.value);
await new Promise((r) => setTimeout(r, 1200));
const after = await send("Runtime.evaluate", { expression: "document.body.innerText", returnByValue: true });
const text = String(after.result?.result?.value ?? "");
console.log("AFTER_HAS_DONE", text.includes("disable 完成"));
ws.close();
