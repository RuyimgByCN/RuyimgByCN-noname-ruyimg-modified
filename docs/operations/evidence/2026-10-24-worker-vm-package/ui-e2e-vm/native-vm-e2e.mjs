const targets = await fetch("http://127.0.0.1:9222/json").then((r) => r.json());
const page = targets.find((t) => t.type === "page" && t.url.includes("127.0.0.1:4173"));
const ws = new WebSocket(page.webSocketDebuggerUrl);
let id = 0; const pending = new Map();
function send(method, params = {}) { const i = ++id; ws.send(JSON.stringify({ id: i, method, params })); return new Promise((r) => pending.set(i, r)); }
ws.onmessage = (e) => { const m = JSON.parse(e.data); if (m.id && pending.has(m.id)) { pending.get(m.id)(m); pending.delete(m.id); return; } if (m.method === "Page.javascriptDialogOpening") void send("Page.handleJavaScriptDialog", { accept: true }); };
await new Promise((res, rej) => { ws.onopen = res; ws.onerror = rej; });
await send("Runtime.enable"); await send("Page.enable");
const body = async () => String((await send("Runtime.evaluate", { expression: "document.body.innerText", returnByValue: true })).result?.result?.value ?? "");
for (let i = 0; i < 40; i++) { if ((await body()).includes("vm-fixture")) break; await new Promise((r) => setTimeout(r, 200)); }
console.log("HAS_PACKAGE", (await body()).includes("vm-fixture"));
const click = async (label) => (await send("Runtime.evaluate", { expression: `(() => { const b=[...document.querySelectorAll('button')].find(x=>x.textContent.includes(${JSON.stringify(label)})); if(!b) return 'no-button'; b.click(); return 'clicked'; })()`, returnByValue: true })).result?.result?.value;
for (const [label, marker] of [["停用","disable 完成"],["激活","activate 完成"],["扫描安装","安装扫描完成"]]) {
  console.log(label, await click(label));
  await new Promise((r) => setTimeout(r, 800));
  console.log(marker, (await body()).includes(marker));
}
await send("Emulation.setDeviceMetricsOverride", { width: 1280, height: 1800, deviceScaleFactor: 1, mobile: false });
await send("Runtime.evaluate", { expression: `(() => { const el=[...document.querySelectorAll('legend')].find(x=>x.textContent.includes('原生扩展包')); if(el) el.scrollIntoView({block:'start'}); return 'ok'; })()`, returnByValue: true });
await new Promise((r) => setTimeout(r, 300));
const shot = await send("Page.captureScreenshot", { format: "png", captureBeyondViewport: false });
const { writeFileSync } = await import("node:fs");
writeFileSync("/tmp/native-real-settings.png", Buffer.from(shot.result.data, "base64"));
console.log("UNINSTALL", await click("卸载"));
await new Promise((r) => setTimeout(r, 1000));
console.log("UNINSTALL_DONE", (await body()).includes("uninstall 完成"));
ws.close();
