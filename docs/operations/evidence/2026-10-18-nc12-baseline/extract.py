#!/usr/bin/env python3
"""NC12 基线日志汇总：把 RESULT 行还原成 README 用的结果表。

用法：python3 -B extract.py            # 打印全部 markdown 表
      python3 -B extract.py --check    # 一致性检查（RESULT 计数 / 非零退出 / 命令清单）

数据源：同目录 01..11 号日志（run-all.sh 产生）。不执行被测代码、不联网。
分位数由测量工具按 nearest-rank 算好并打印，这里只做汇总，不重算。
"""

import re
import sys
from collections import OrderedDict, defaultdict
from pathlib import Path

HERE = Path(__file__).resolve().parent
RESULT = re.compile(r"^RESULT\t(.*)$")


def parse_log(path):
    rows = []
    if not path.exists():
        return rows
    for line in path.read_text(encoding="utf-8", errors="replace").splitlines():
        m = RESULT.match(line)
        if not m:
            continue
        kv = {}
        for part in m.group(1).split("\t"):
            if "=" in part:
                k, v = part.split("=", 1)
                kv[k] = v
        rows.append(kv)
    return rows


def meta(path):
    """(命令列表, [(命令, 退出码)], [峰值 RSS 字节])。"""
    cmds, exits, maxrss, pending = [], [], [], None
    for line in path.read_text(encoding="utf-8", errors="replace").splitlines():
        if line.startswith("$ "):
            pending = line[2:]
            cmds.append(pending)
        elif line.startswith("exit="):
            exits.append((pending, int(line.split("=", 1)[1])))
        elif "maximum resident set size" in line:
            maxrss.append(int(line.split()[0]))
    return cmds, exits, maxrss


def loads(path):
    out = []
    for line in path.read_text(encoding="utf-8", errors="replace").splitlines():
        if line.startswith("load_before:") or line.startswith("load_after:"):
            m = re.search(r"load averages?: ([\d.]+)", line)
            if m:
                out.append(float(m.group(1)))
    return out


def grouped(rows):
    """(tier, config) → 合并后的 kv（后出现的键覆盖，行内键互补）。"""
    out = OrderedDict()
    for r in rows:
        key = (r.get("tier", ""), r.get("config", ""))
        out.setdefault(key, {}).update(r)
    return out


def metric_of(rows, want):
    """metric → (tier, config) 索引。"""
    idx = {}
    for r in rows:
        if r.get("metric") == want:
            idx[(r.get("tier", ""), r.get("config", ""))] = r
    return idx


def stages(cfg):
    """配置串末尾的 stage/steps（stage 行会重复 steps=，取最后一次）。"""
    st = re.search(r"stage=(\w+)", cfg)
    sp = re.findall(r"steps=(\d+)", cfg)
    return (st.group(1) if st else "-"), (sp[-1] if sp else "-")


def us(ns):
    try:
        return f"{int(ns) / 1000:.1f}"
    except (TypeError, ValueError):
        return "-"


def table_probe(res):
    rows = [r for r in res.get("02-probe.log", []) if r.get("scenario") == "probe" and not r.get("metric")]
    print("### 表 A 三档装载与目录规模（`probe samples=5`，2026-10-18 快照）\n")
    print("| 档位 | 来源目录 | package_id | 全量导入 import_ns | 请求包桥接 bridge_ns | 卡牌 | 技能 | 扩展武将 | ruleset_version | content_lock | 开局归档字节 |")
    print("|---|---|---|---:|---:|---:|---:|---:|---:|---:|---:|")
    for r in rows:
        print(
            "| {t} | {d} | {p} | {i} | {b} | {c} | {s} | {g} | {rv} | {lk} | {ab} |".format(
                t=r.get("tier", ""), d=r.get("tier_dirs", "-") or "（基础目录）",
                p=r.get("package_ids", "-（未加载扩展）"), i=r.get("import_ns", "-"), b=r.get("bridge_ns", "-"),
                c=r.get("cards", "-"), s=r.get("skills", "-"), g=r.get("extension_generals", "-"),
                rv=r.get("ruleset_version", "-"), lk=r.get("content_lock_id", "0"),
                ab=r.get("archive_bytes", "-"),
            )
        )
    print()


def table_build(res):
    rows = res.get("03-build.log", [])
    stats = metric_of(rows, "engine_build_ns")
    cat = [r for r in rows if r.get("metric") == "standard_catalog_ns"]
    info = {r.get("tier", ""): r for r in rows if r.get("metric") is None}
    print("### 表 B 建局耗时（`samples=30`，4 座，种子 42007+i，ns）\n")
    print("| 档位 | 标准目录构建 p50 | 建局 p50 | 建局 p95 | 建局 max | 全量导入 import | 请求包桥接 bridge | 卡牌 | 技能 | 开局归档字节 |")
    print("|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|")
    std = cat[0].get("p50_ns", "-") if cat else "-"
    for (tier, _cfg), r in stats.items():
        i = info.get(tier, {})
        print(
            f"| {tier} | {std} | {r.get('p50_ns','-')} | {r.get('p95_ns','-')} | {r.get('max_ns','-')} | "
            f"{i.get('import_ns','-')} | {i.get('bridge_ns','-')} | {i.get('cards','-')} | {i.get('skills','-')} | {i.get('archive_bytes_at_open','-')} |"
        )
    print()


def table_drive(res):
    print("### 表 C 整局规划驱动：每命令延迟（4 座，3 局/组，种子 42007–42009）\n")
    print("分位单位 ns；`plan`=AI 克隆模拟，`submit`=权威规则结算，`cmd`=plan+submit。降级=该组含规划 panic（catch_panics=1 捕获后以兜底动作续行）。\n")
    print("| 轮次 | 档位 | 预算 | 变体 | 局数 | 步数 | 收官局 | wall/步 | plan p50 | plan p95 | plan max | submit p50 | submit p95 | submit max | cmd p50 | cmd p95 | cmd max | 共识节点/秒 | 规划 panic | 降级 |")
    print("|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|:--:|")
    for name, tag in (("04-drive.log", "干净"), ("04b-drive-degraded-variant1.log", "兜底")):
        rows = [r for r in res.get(name, []) if r.get("scenario") == "drive"]
        agg = {(r.get("tier", ""), r.get("config", "")): r for r in rows if "all_games" in r.get("config", "")}
        stats = defaultdict(dict)
        for r in rows:
            if r.get("metric") in ("plan_ns", "submit_ns", "cmd_ns") and "all_games" in r.get("config", ""):
                stats[(r.get("tier", ""), r.get("config", ""))][r["metric"]] = r
        for (tier, cfg), s in sorted(agg.items(), key=lambda kv: (kv[0][1], kv[0][0])):
            m = re.search(r"budget=(\d+),lookahead=(\d+),variant=(\d+)", cfg)
            if not m:
                continue
            st = stats[(tier, cfg)]
            print(
                "| {tag} | {t} | {b} | {v} | {g} | {steps} | {fin}/{g} | {ws} | {a} | {b2} | {c} | {d} | {e} | {f} | {g2} | {h} | {i} | {nps} | {p} | {deg} |".format(
                    tag=tag, t=tier, b=m.group(1), v=m.group(3), g=s.get("games", "-"),
                    steps=s.get("steps", "-"), fin=s.get("finished_games", "-"),
                    ws=s.get("ns_per_step_wall", "-"),
                    a=st.get("plan_ns", {}).get("p50_ns", "-"), b2=st.get("plan_ns", {}).get("p95_ns", "-"),
                    c=st.get("plan_ns", {}).get("max_ns", "-"),
                    d=st.get("submit_ns", {}).get("p50_ns", "-"), e=st.get("submit_ns", {}).get("p95_ns", "-"),
                    f=st.get("submit_ns", {}).get("max_ns", "-"),
                    g2=st.get("cmd_ns", {}).get("p50_ns", "-"), h=st.get("cmd_ns", {}).get("p95_ns", "-"),
                    i=st.get("cmd_ns", {}).get("max_ns", "-"),
                    nps=s.get("nodes_per_sec_plan", "-"), p=s.get("plan_panics", "0"),
                    deg=s.get("degraded", "false"),
                )
            )
    print()


def table_copy(res):
    rows = [r for r in res.get("05-copy.log", []) if r.get("scenario") == "copy"]
    g = grouped(rows)
    metrics = defaultdict(dict)
    for r in rows:
        if r.get("metric"):
            metrics[(r.get("tier", ""), r.get("config", ""))][r["metric"]] = r
    print("### 表 D 状态复制 / AI 模拟 / 归档编码（`budget=64`，`samples=30`，ns）\n")
    print("| 档位 | 阶段 | 步数 | save p50 | Engine::clone p50 | plan_action p50 | plan_action p95 | encode_archive p50 | 归档字节 | 规划节点 p50/max | 状态形状 |")
    print("|---|---|---:|---:|---:|---:|---:|---:|---:|---|---|")
    for (tier, cfg), r in g.items():
        if "stage=" not in cfg:
            continue
        stage, steps = stages(cfg)
        st = metrics[(tier, cfg)]
        print(
            f"| {tier} | {stage} | {steps} | {st.get('save_ns',{}).get('p50_ns','-')} | "
            f"{st.get('engine_clone_ns',{}).get('p50_ns','-')} | {st.get('plan_action_ns',{}).get('p50_ns','-')} | "
            f"{st.get('plan_action_ns',{}).get('p95_ns','-')} | {st.get('encode_archive_ns',{}).get('p50_ns','-')} | "
            f"{r.get('archive_bytes','-')} | {r.get('plan_nodes_p50','-')}/{r.get('plan_nodes_max','-')} | {r.get('state','-')} |"
        )
    print()
    print("**状态规模随对局推进（growth 采样：帧数/技能数/牌区/日志/归档字节）**\n")
    print("| 档位 | 步 | revision | frames | parks | skills | deck | discard | hand | log | 归档字节 |")
    print("|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|")
    for r in rows:
        if "growth" not in r.get("config", "") or r.get("step") is None:
            continue
        shape = dict(p.split("=", 1) for p in r.get("state", "").split() if "=" in p)
        print(
            f"| {r.get('tier','')} | {r.get('step')} | {shape.get('revision','-')} | {shape.get('frames','-')} | "
            f"{shape.get('parks','-')} | {shape.get('skills','-')} | {shape.get('deck','-')} | {shape.get('discard','-')} | "
            f"{shape.get('hand','-')} | {shape.get('log','-')} | {r.get('archive_bytes','-')} |"
        )
    print()


def table_archive(res):
    rows = [r for r in res.get("06-archive.log", []) if r.get("scenario") == "archive"]
    g = grouped(rows)
    metrics = defaultdict(dict)
    for r in rows:
        if r.get("metric"):
            metrics[(r.get("tier", ""), r.get("config", ""))][r["metric"]] = r
    print("### 表 E 存读档（`samples=30`，`steps=80`，ns）\n")
    print("| 档位 | 阶段 | 步数 | save p50 | encode p50 | decode p50 | restore p50 | restore p95 | restore max | 归档字节 | 往返失败 | 状态哈希不一致 | 结果 |")
    print("|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|:--:|")
    for (tier, cfg), r in g.items():
        if "stage=" not in cfg:
            continue
        stage, steps = stages(cfg)
        st = metrics[(tier, cfg)]
        print(
            f"| {tier} | {stage} | {steps} | {st.get('save_ns',{}).get('p50_ns','-')} | "
            f"{st.get('encode_archive_ns',{}).get('p50_ns','-')} | {st.get('decode_archive_ns',{}).get('p50_ns','-')} | "
            f"{st.get('restore_ns',{}).get('p50_ns','-')} | {st.get('restore_ns',{}).get('p95_ns','-')} | "
            f"{st.get('restore_ns',{}).get('max_ns','-')} | {r.get('archive_bytes','-')} | "
            f"{r.get('roundtrip_failures','-')} | {r.get('state_hash_mismatch','-')} | {r.get('result','-')} |"
        )
    print()
    diag = {}
    for r in rows:
        if r.get("diagnostic"):
            diag[(r.get("tier", ""), r.get("config", ""))] = r
    if diag:
        print("**读档被拒诊断**（同一快照重复 30 次的同一诊断，去重后）\n")
        print("| 档位 | 阶段 | what | card | card_name | 计入牌区 | 未计入特殊区 | 挂起机器 parks |")
        print("|---|---|---|---:|---|---|---|---:|")
        for (tier, cfg), r in diag.items():
            stage, _ = stages(cfg)
            print(
                f"| {tier} | {stage} | {r.get('what','-')} | {r.get('card','-')} | {r.get('card_name','-')} | "
                f"{r.get('counted','-')} | {r.get('uncounted','-')} | {r.get('parks','-')} |"
            )
        print()


def table_ai(res):
    rows = res.get("07-ai.log", []) + res.get("07b-ai-combo-degraded.log", [])
    st = metric_of(rows, "plan_action_ns")
    nd = metric_of(rows, "nodes_per_plan")
    info = {(r.get("tier", ""), r.get("config", "")): r for r in rows if r.get("result") == "OK"}
    cmp_ = {(r.get("tier", ""), r.get("config", "")): r for r in rows if r.get("compare_base")}
    print("### 表 F AI 策略矩阵（同种子 42007，`cmds=200`，ns）\n")
    print("兜底=该组含规划 panic（combo 档 variant1 在 step 5 崩，见 §5）。`与 v0 决策不同步`=同预算下与 lookahead1/variant0 逐步比较的不同决策数。\n")
    print("| 档位 | 预算 | lookahead | variant | 步数 | accepted | plan p50 | plan p95 | plan max | 节点 total | 节点 p50/max | 节点/秒 | 与 v0 不同步 | 收官 | 规划 panic |")
    print("|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---|---:|---:|:--:|---:|")
    for (tier, cfg), i in st.items():
        m = re.search(r"budget=(\d+),lookahead=(\d+),variant=(\d+)", cfg)
        if not m:
            continue
        s = st[(tier, cfg)]
        n = nd.get((tier, cfg), {})
        c = cmp_.get((tier, cfg), {})
        inf = info.get((tier, cfg), {})
        print(
            f"| {tier} | {m.group(1)} | {m.group(2)} | {m.group(3)} | {inf.get('steps','-')} | {inf.get('accepted','-')} | "
            f"{s.get('p50_ns','-')} | {s.get('p95_ns','-')} | {s.get('max_ns','-')} | {inf.get('nodes_total','-')} | "
            f"{n.get('p50','-')}/{n.get('max','-')} | {inf.get('nodes_per_sec','-')} | {c.get('decision_diff_steps','-')} | "
            f"{inf.get('finished','-')} | {inf.get('plan_panics','-')} |"
        )
    print()


def table_long(res):
    rows = [r for r in res.get("08-long.log", []) if r.get("scenario") == "long"]
    _, _, rss = meta(HERE / "08-long.log")
    print("### 表 G 长局内存（`budget=8`，`min_cmds=800`，`NC12_ALLOC=1` 计数分配器）\n")
    print("堆字节为「打开计数后」窗口内的累计量；峰值活字节为 alloc−free 的运行峰值。RSS 由外部 `/usr/bin/time -l` 取 maximum resident set size（沙箱禁止 `ps`/`top`）。\n")
    print("| 档位 | 局 | 步数 | 收官 | 规划 panic | 窗口堆分配 (B) | 窗口堆释放 (B) | 局末活字节 (B) |")
    print("|---|---:|---:|:--:|---:|---:|---:|---:|")
    for r in rows:
        if r.get("game") and r.get("steps"):
            print(
                f"| {r.get('tier','')} | {r.get('game')} | {r.get('steps')} | {r.get('finished','-')} | {r.get('plan_panics','-')} | "
                f"{r.get('heap_allocated_bytes','-')} | {r.get('heap_freed_bytes','-')} | {r.get('heap_live_bytes','-')} |"
            )
    print()
    print("| 档位 | 局数 | 总步数 | 窗口分配 (B) | 窗口释放 (B) | 活字节增量 (B) | 分配次数 | 峰值活字节 (B) | 进程峰值 RSS (B) |")
    print("|---|---:|---:|---:|---:|---:|---:|---:|---:|")
    finals = [r for r in rows if r.get("result") == "OK" and r.get("total_cmds")]
    for idx, r in enumerate(finals):
        peak = rss[idx] if idx < len(rss) else "-"
        games = len({x.get("game") for x in rows if x.get("tier") == r.get("tier") and x.get("game")})
        print(
            f"| {r.get('tier','')} | {games} | {r.get('total_cmds','-')} | {r.get('heap_allocated_bytes','-')} | "
            f"{r.get('heap_freed_bytes','-')} | {r.get('heap_live_delta_bytes','-')} | {r.get('heap_alloc_calls','-')} | "
            f"{r.get('heap_peak_live_bytes','-')} | {peak} |"
        )
    print()


def panics_in(path):
    """日志文本里的 panic 现场：[ (位置, 消息) ]。"""
    out, lines = [], path.read_text(encoding="utf-8", errors="replace").splitlines()
    for i, line in enumerate(lines):
        m = re.search(r"panicked at (.+?):$", line.strip())
        if m:
            msg = lines[i + 1].strip() if i + 1 < len(lines) else "-"
            out.append((m.group(1), msg))
    return out


def panics_in(path):
    """日志文本里的 panic 现场：[ (位置, 消息) ]。"""
    out, lines = [], path.read_text(encoding="utf-8", errors="replace").splitlines()
    for i, line in enumerate(lines):
        m = re.search(r"panicked at (.+?):$", line.strip())
        if m:
            msg = lines[i + 1].strip() if i + 1 < len(lines) else "-"
            out.append((m.group(1), msg))
    return out


def table_failures(res):
    print("### 表 H 失败与边界\n")
    print("**H1a 非零退出清单（未捕获的 panic 直接中止进程；exit=101）**\n")
    print("| 日志 | 档位 | 预算 | lookahead | variant | 种子 | 额外参数 | 退出码 |")
    print("|---|---|---:|---:|---:|---:|---|---:|")
    for name in ("04-drive.log", "04b-drive-degraded-variant1.log", "07-ai.log",
                 "09-first-failure-panic.log", "09b-first-failure-swd-real-trajectory-panic.log",
                 "10-panic-boundary-matrix.log"):
        cmds, exits, _ = meta(HERE / name)
        for cmd, code in exits:
            if code == 0:
                continue
            m = re.search(r"tier=(\w+) seats=4 budget=(\d+) lookahead=(\d+) variant=(\d+).*?seed=(\d+)", cmd or "")
            extra = "catch_panics=1" if "catch_panics" in (cmd or "") else "-"
            if m:
                print(f"| {name.split('-')[0]} | {m.group(1)} | {m.group(2)} | {m.group(3)} | {m.group(4)} | {m.group(5)} | {extra} | {code} |")
            else:
                m2 = re.search(r"tier=(\w+)", cmd or "")
                print(f"| {name.split('-')[0]} | {m2.group(1) if m2 else '-'} | - | - | - | - | {extra} | {code} |")
    print()
    print("**H1b panic 现场（位置与消息，按日志顺序）**\n")
    print("| 日志 | # | panic 位置 | 消息 |")
    print("|---|---:|---|---|")
    for name in ("04-drive.log", "04b-drive-degraded-variant1.log", "09-first-failure-panic.log",
                 "09b-first-failure-swd-real-trajectory-panic.log", "10-panic-boundary-matrix.log"):
        for i, (loc, msg) in enumerate(panics_in(HERE / name)):
            print(f"| {name.split('-')[0]} | {i + 1} | {loc} | {msg} |")
    print()
    print("**H2 读档失败边界（archive：swd 单扩展）**\n")
    print("| 档位 | 步数 | 往返失败/样本 | 结果 | 诊断 |")
    print("|---|---:|---:|:--:|---|")
    rows = res.get("11-restore-failure-matrix.log", [])
    g = grouped([r for r in rows if r.get("scenario") == "archive"])
    diag = {(r.get("tier", ""), r.get("config", "")): r for r in rows if r.get("diagnostic")}
    for (tier, cfg), r in g.items():
        if r.get("result") not in ("OK", "FAIL"):
            continue
        stage, steps = stages(cfg)
        if stage != "played":
            continue
        n = re.search(r"samples=(\d+)", cfg)
        d = diag.get((tier, cfg), {})
        note = (
            f"{d.get('what','')} card={d.get('card','')}({d.get('card_name','')}) {d.get('counted','')}"
            if d
            else "-"
        )
        print(f"| {tier} | {steps} | {r.get('roundtrip_failures','-')}/{n.group(1) if n else '-'} | {r.get('result','-')} | {note} |")
    print()


def check(res):
    print("一致性检查：")
    problems = 0
    for name, rows in sorted(res.items()):
        cmds, exits, _ = meta(HERE / name)
        fails = [c for c, code in exits if code != 0]
        print(f"  {name}: RESULT {len(rows)} 行；命令 {len(cmds)}；非零退出 {len(fails)}")
        for c in fails:
            print(f"    - 退出非零: {c}")
        problems += len(fails)
    print(f"合计非零退出（已登记的首次失败）：{problems}")
    return 0


def main():
    if "--check" in sys.argv:
        return check({p.name: parse_log(p) for p in sorted(HERE.glob("*.log"))})
    res = {p.name: parse_log(p) for p in sorted(HERE.glob("*.log"))}
    print("# NC12 基线结果表（由 `extract.py` 从原始日志生成）\n")
    for path in sorted(HERE.glob("*.log")):
        ls = loads(path)
        if ls:
            print(f"<!-- {path.name}: load_before/after 采样 {len(ls)} 次，范围 {min(ls):.2f}–{max(ls):.2f} -->")
    print()
    table_probe(res)
    table_build(res)
    table_drive(res)
    table_copy(res)
    table_archive(res)
    table_ai(res)
    table_long(res)
    table_failures(res)
    return 0


if __name__ == "__main__":
    sys.exit(main())
