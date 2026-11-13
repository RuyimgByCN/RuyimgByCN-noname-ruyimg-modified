"""Record actual upstream GameEvents using the native probe's exact visible fixtures."""
from pathlib import Path
import datetime
import json
import os
import time
import urllib.request

BASE = Path(__file__).resolve().parent
meta = json.loads((BASE / "active-session-modern.json").read_text())


def evaluate(expression):
    req = urllib.request.Request(
        "http://127.0.0.1:" + str(meta["controlPort"]),
        data=expression.encode(), method="POST",
    )
    reply = json.loads(urllib.request.urlopen(req, timeout=40).read())
    if reply.get("result", {}).get("exceptionDetails"):
        raise RuntimeError(json.dumps(reply, ensure_ascii=False))
    value = reply["result"]["result"].get("value")
    return json.loads(value) if isinstance(value, str) else value


def wait_for(event):
    evaluate("window.runtimeExpected=" + json.dumps(event) + ";true")
    return evaluate((BASE / "await-choice.js").read_text())


def cancel():
    return evaluate("q1._status.clicked=false;q1.ui.click.cancel();true")


def choose(expression, kind):
    return evaluate("(()=>{q1._status.clicked=false;q1.ui.click." + kind +
                    ".call(" + expression + ");q1._status.clicked=false;" +
                    "q1.ui.click.ok();return true})()")


cases = [json.loads(line) for line in (BASE.parent / "02-native-probe-first.log").read_text().splitlines() if line.startswith('{')]
assert len(cases) == 13
if os.environ.get("U06_ONLY"):
    cases = [c for c in cases if c["case"].startswith(os.environ["U06_ONLY"])]
tag = os.environ.get("U06_RUN_TAG", "first")
for case in cases:
    name = case["case"]
    evaluate("setTimeout(()=>location.reload(),0);true")
    time.sleep(0.15)
    for attempt in range(100):
        try:
            ready = evaluate("(async()=>{const n=await import('/noname.js');return !!(n.game.players.length===8 && n.lib.card.sha && n.game.players.every(p=>!!p.node?.handcards1))})()")
            if ready:
                break
        except Exception:
            pass
        time.sleep(0.1)
    else:
        raise RuntimeError("fresh upstream page failed to initialize")
    evaluate("window.runtimeFixture=" + json.dumps(case, ensure_ascii=False) + ";true")
    setup = evaluate((BASE / "start-case.js").read_text())
    assert setup["playerCount"] == 8 and setup["skillReady"]
    choices = []
    if name in ["monkey-confirm", "monkey-cancel"]:
        choices.append(wait_for("chooseBool"))
        if name.endswith("confirm"):
            evaluate("q1._status.clicked=false;q1.ui.click.ok();true")
        else:
            cancel()
    elif name in ["kuwu-hand", "kuwu-equip"]:
        choices.append(wait_for("chooseToDiscard"))
        choose("q1.game.players[1].getCards('" + ("h" if name.endswith("hand") else "e") + "')[0]", "card")
    elif name.startswith("xuelun") and name != "xuelun-no-skills":
        choices.append(wait_for("chooseTarget"))
        if name == "xuelun-target-cancel":
            cancel()
        else:
            target = 2 if name == "xuelun-target-b" else 1
            choose("q1.game.players[" + str(target) + "]", "target")
            choices.append(wait_for("chooseButton"))
            if name == "xuelun-skill-cancel":
                cancel()
            else:
                selected = case["initial"]["selected_skill"]
                mapped = {"马术": "mashu", "咆哮": "paoxiao", "武圣": "wusheng"}[selected]
                choose("q1.ui.dialog.buttons.find(b=>b.link===" + json.dumps(mapped) + ")", "button")
    result = evaluate((BASE / "finish-case.js").read_text())
    assert result["settled"]
    row = {"at": datetime.datetime.now().astimezone().isoformat(), "case": name,
           "run_tag": tag, "setup": setup, "choices": choices, "result": result}
    with (BASE / "case-results.jsonl").open("a") as out:
        out.write(json.dumps(row, ensure_ascii=False) + "\n")
    print(name, "settled", flush=True)
