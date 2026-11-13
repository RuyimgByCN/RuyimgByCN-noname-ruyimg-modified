"""Replay the native probe's actual card faces through upstream GameEvents."""
import datetime
import json
import os
import pathlib
import urllib.request

BASE = pathlib.Path(__file__).resolve().parent
meta = json.loads((BASE / "active-session-modern.json").read_text())


def evaluate(expression):
    request = urllib.request.Request(
        f"http://127.0.0.1:{meta['controlPort']}", data=expression.encode(), method="POST"
    )
    reply = json.loads(urllib.request.urlopen(request, timeout=55).read())
    if reply.get("result", {}).get("exceptionDetails"):
        raise RuntimeError(json.dumps(reply, ensure_ascii=False))
    value = reply["result"]["result"].get("value")
    return json.loads(value) if isinstance(value, str) else value


def wait_for(name):
    evaluate(f"window.queue1Expected={json.dumps(name)};true")
    return evaluate((BASE / "modern-choice.js").read_text())


native = [json.loads(line) for line in (BASE.parent / "native-comparison-green-first.log").read_text().splitlines() if line.startswith('{')]
cases = [case for case in native if case["case"].startswith(("yuanzhen-", "juxian-", "xuanlun-"))]
assert len(cases) == 8, f"incomplete native results: {len(cases)}"
if os.environ.get("QUEUE1_ONLY"):
    cases = [case for case in cases if case["case"].startswith(os.environ["QUEUE1_ONLY"])]
for case in cases:
    name = case["case"]
    skill = "dddyuanzhen" if name.startswith("yuanzhen") else "dddjuxian" if name.startswith("juxian") else "dddxuanlun"
    evaluate(f"window.queue1Fixture={json.dumps(case['fixtures'])};window.queue1Skill={json.dumps(skill)};true")
    evaluate((BASE / "modern-u01-start.js").read_text())
    if name.startswith("yuanzhen"):
        choice = wait_for("chooseToDiscard")
        if name.endswith("cancel"):
            evaluate("q1._status.clicked=false;q1.ui.click.cancel();true")
        else:
            key = "first" if name.endswith("sha") else "second"
            evaluate("(()=>{q1._status.clicked=false;q1.ui.click.card.call(queue1Cards." + key + ");if(q1.ui.selected.cards.length!==1)throw Error('card count');q1._status.clicked=false;q1.ui.click.ok();return true})()")
    elif name.startswith("juxian"):
        choice = wait_for("gainPlayerCard")
        index = 0 if name.endswith("black") else 1 if name.endswith("red") else 2
        # Hidden hand buttons are selected by slot, never by an AI reading faces.
        evaluate("(()=>{const button=q1.ui.dialog.buttons[" + str(index) + "];if(!button)throw Error('button absent');q1._status.clicked=false;q1.ui.click.button.call(button);if(q1.ui.selected.buttons.length!==1)throw Error('button count');q1._status.clicked=false;q1.ui.click.ok();return true})()")
    else:
        wait_for("chooseToRespond")
        evaluate("q1._status.clicked=false;q1.ui.click.cancel();true")
        choice = wait_for("chooseBool")
        evaluate("q1._status.clicked=false;q1.ui.click." + ("ok" if name.endswith("confirm") else "cancel") + "();true")
    result = evaluate((BASE / "modern-finish.js").read_text())
    assert result["done"] is True
    row = {"at": datetime.datetime.now(datetime.timezone.utc).isoformat(), "case": os.environ.get("QUEUE1_RUN_TAG", "exact") + "-" + name,
           "engine": "modern", "choice": choice, "result": result}
    with (BASE / "case-results.jsonl").open("a") as stream:
        stream.write(json.dumps(row, ensure_ascii=False) + "\n")
    print(name, "done", flush=True)
