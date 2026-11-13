# L10 实际运行命令与首次失败

工作目录均为 `/Users/ruyimg/Documents/Claude/sanguosha/noname-next`。2026-11-14 本地日期；原始 UTC 时间见 `preflight.json` 和 CDP jsonl。工具链/操作系统、HEAD/dirty、包/源码与二进制 hash 见 `environment-final.json`。没有产品改动、Cargo测试或等价签署；本轮只运行外部 probe 与原始上游浏览器场景。

启动自有服务/浏览器（已清理）：

```bash
QUEUE1_LABEL=l10 node docs/operations/evidence/2026-11-14-queue1-runtime/L10/launch.mjs > docs/operations/evidence/2026-11-14-queue1-runtime/L10/launch-first.raw.txt 2>&1
```

`launch.mjs`复用正式tabletop-comparison-cdp启动方式，来源根为真实noname-liuli/noname，独占随机localhost HTTP/CDP/control端口与profile；所有包含spine的路径在任何磁盘读取前404。实际端口50341/50357/50398。CDP原始request/reply/events/HTTP分别在 `session-l10-1794593358863-*.jsonl`；没有Spine路径请求。

原生首跑：

```bash
L10_REPO=/Users/ruyimg/Documents/Claude/sanguosha/noname-next L10_EVIDENCE=/Users/ruyimg/Documents/Claude/sanguosha/noname-next/docs/operations/evidence/2026-11-14-queue1-runtime/L10/native-round1 cargo run --offline --manifest-path docs/operations/evidence/2026-11-14-queue1-runtime/L10/native-probe/Cargo.toml --target-dir target > docs/operations/evidence/2026-11-14-queue1-runtime/L10/native-first.raw.txt 2>&1
```

首跑失败保留：夹具向原生debug_set_deck_top传上游名guohe，原生只接受dismantle，panic。首版源码为 `native-first-source.rs`，首轮仅产出pack/metadata；不计成功场景。修正夹具牌名，并避免move场把已在手的唯一诸葛再放入牌堆。未改产品。

原生第二轮：

```bash
L10_REPO=/Users/ruyimg/Documents/Claude/sanguosha/noname-next L10_EVIDENCE=/Users/ruyimg/Documents/Claude/sanguosha/noname-next/docs/operations/evidence/2026-11-14-queue1-runtime/L10/native-round2 cargo run --offline --manifest-path docs/operations/evidence/2026-11-14-queue1-runtime/L10/native-probe/Cargo.toml --target-dir target > docs/operations/evidence/2026-11-14-queue1-runtime/L10/native-second.raw.txt 2>&1
```

26case各direct/restored，共52次场景执行、164个命令请求/回包，102份pending NGSA恢复成功。probe有1条未用Suit import warning，未作为产品门禁。该probe未再编辑；冻结二进制及hash见environment-final。使用共享target且未清理，无自建private target。

浏览器首跑：

```bash
python3 -B docs/operations/evidence/2026-11-14-queue1-runtime/L10/run-browser.py browser-round1 > docs/operations/evidence/2026-11-14-queue1-runtime/L10/browser-first.raw.txt 2>&1
```

Python三元式else缺空格导致SyntaxError，首源码 `browser-first-source.py` 和首raw保留，没有执行任何场景。

第二轮小组：

```bash
python3 -B docs/operations/evidence/2026-11-14-queue1-runtime/L10/run-browser.py browser-round2 g_sha_ally g_sha_question_fake r_basic r_move_A k_A > docs/operations/evidence/2026-11-14-queue1-runtime/L10/browser-second.raw.txt 2>&1
```

前四场已观察目标分支；党锢未触发。检查真正game.js:32516的global hook通过game.playerMap查owner，发现首夹具只设置playerid而未注册playerMap。保留首夹具与runner `browser-round2-{setup.js,runner.py}`，不把该失败称UI依赖。仅补各真实玩家的playerMap登记，不改技能/过滤/内容。

第三轮确认真实入口：

```bash
python3 -B docs/operations/evidence/2026-11-14-queue1-runtime/L10/run-browser.py browser-round3 k_A > docs/operations/evidence/2026-11-14-queue1-runtime/L10/browser-third.raw.txt 2>&1
```

真实phaseBefore进入k_danggu step0后抛dui未定义；同场原始EpicFX.initSCSAnim抛dcdAnim未定义。不是stub后的成功。

最终完整cohort：

```bash
python3 -B docs/operations/evidence/2026-11-14-queue1-runtime/L10/run-browser.py browser-round4 > docs/operations/evidence/2026-11-14-queue1-runtime/L10/browser-fourth.raw.txt 2>&1
python3 -B docs/operations/evidence/2026-11-14-queue1-runtime/L10/compare-results.py > docs/operations/evidence/2026-11-14-queue1-runtime/L10/comparison-build.raw.txt 2>&1
```

浏览器26个fresh页中23个观察到目标分支、3个k UI错误；0个late无ID牌。比较脚本自检9限定投影相同/12实际差异/2仅门控/3阻塞，未签等价。

清理自有launcher PID86531前核对父子PID及精确命令，SIGTERM后profile已移除，两个PID均不存在、50341/50357/50398的直连TCP均ECONNREFUSED(61)。初次urllib关闭复查超时也原样保存在cleanup.json，直接socket复查另存cleanup-direct-check.json。保留二进制 `/private/tmp/noname-l10-runtime-preserved-tvv_75wj/native-probe-round2`，未清任何共享缓存或其他浏览器。
