# U08 实际运行命令（2026-11-14）

工作目录均为 `/Users/ruyimg/Documents/Claude/sanguosha/noname-next`。规则333／NGSA107／schema5／ABI759／WS8。最初 HEAD `605201f3f49e10cea61c31d52a61bbb66c85599d`，带主 agent 的酒诗恢复变更；收口 HEAD `5c73d1d9158145e33f98f2018013914667d13964`。完整两次 tracked status 分别在 preflight.json 与 freeze-environment.json，不把并行文档改动视为本 agent 改动。

启动节点的实际 argv 由 `ps -ww -p 92560 -o pid,ppid,command` 核对为下列命令，stdout保留在 launch-first.raw.txt。Chrome/CDP/profile/control只属于此节点；5173与51215为主 agent 的只读共享服务。

```bash
node docs/operations/evidence/2026-11-14-queue1-runtime/U08/launch.mjs
```

原生两次实际编译运行如下。首轮语法错误原样保留，第二轮仅修probe的多余括号与未用函数，没有改产品；使用共享 target，不创建或清除任何 Cargo 私有缓存。

```bash
U08_REPO=/Users/ruyimg/Documents/Claude/sanguosha/noname-next U08_EVIDENCE=/Users/ruyimg/Documents/Claude/sanguosha/noname-next/docs/operations/evidence/2026-11-14-queue1-runtime/U08/native-round1 cargo run --offline --manifest-path docs/operations/evidence/2026-11-14-queue1-runtime/U08/native-probe/Cargo.toml --target-dir target > docs/operations/evidence/2026-11-14-queue1-runtime/U08/native-first.raw.txt 2>&1
U08_REPO=/Users/ruyimg/Documents/Claude/sanguosha/noname-next U08_EVIDENCE=/Users/ruyimg/Documents/Claude/sanguosha/noname-next/docs/operations/evidence/2026-11-14-queue1-runtime/U08/native-round2 cargo run --offline --manifest-path docs/operations/evidence/2026-11-14-queue1-runtime/U08/native-probe/Cargo.toml --target-dir target > docs/operations/evidence/2026-11-14-queue1-runtime/U08/native-second.raw.txt 2>&1
```

上游各轮实际命令：

```bash
python3 docs/operations/evidence/2026-11-14-queue1-runtime/U08/browser-runner.py browser-round1 y_target_A g_target_A b_hit b_miss > docs/operations/evidence/2026-11-14-queue1-runtime/U08/browser-first.raw.txt 2>&1
python3 docs/operations/evidence/2026-11-14-queue1-runtime/U08/browser-runner.py browser-round2 y_target_A g_target_A b_hit b_miss > docs/operations/evidence/2026-11-14-queue1-runtime/U08/browser-second.raw.txt 2>&1
python3 docs/operations/evidence/2026-11-14-queue1-runtime/U08/browser-runner.py browser-round3 > docs/operations/evidence/2026-11-14-queue1-runtime/U08/browser-third.raw.txt 2>&1
python3 docs/operations/evidence/2026-11-14-queue1-runtime/U08/browser-runner.py browser-round4 > docs/operations/evidence/2026-11-14-queue1-runtime/U08/browser-fourth.raw.txt 2>&1
```

第一轮4场初态弃牌面的闪电nature断言失败。第二轮4个代表正例跑通。第三轮31场保留医心目标取消后误假设能回到原选牌窗的首个runner失败。第四轮31场使用真实HTMLElement.click控制按钮并如实处理结束选择的行为，30结束、1强制挂起、0runner异常。各轮的脚本快照均保留，最终选择场景没有替换规则函数、卡牌定义或技能content。

最后数据核验命令：

```bash
python3 docs/operations/evidence/2026-11-14-queue1-runtime/U08/compare.py > docs/operations/evidence/2026-11-14-queue1-runtime/U08/comparison.raw.txt 2>&1
python3 docs/operations/evidence/2026-11-14-queue1-runtime/U08/validate-initial.py > docs/operations/evidence/2026-11-14-queue1-runtime/U08/initial-validation.raw.txt 2>&1
python3 docs/operations/evidence/2026-11-14-queue1-runtime/U08/write-report.py
```

另外的读取/收集命令及工具链实际argv、stdout保留在 runtime-inputs.json、browser-environment.json；原始每次CDP请求/回包（包括早期顶层await语法探测失败）没有删除。原生完整请求/回包/日志逐条保留。未运行共享Cargo全量门禁或产品UI验收，本任务只收集这三个技能的运行证据。

归档核验与原始CDP行索引的实际命令：

```bash
python3 docs/operations/evidence/2026-11-14-queue1-runtime/U08/validate-native.py > docs/operations/evidence/2026-11-14-queue1-runtime/U08/native-validation.raw.txt 2>&1
python3 docs/operations/evidence/2026-11-14-queue1-runtime/U08/index-raw.py > docs/operations/evidence/2026-11-14-queue1-runtime/U08/raw-index.raw.txt 2>&1
```

自有Node PID核验后实际发送 `kill -TERM 92560`；其原有退出处理关闭Chrome92575、CDP50185、control50210并删除自己的profile。cleanup.json记录退出前后端口：共享5173/51215仍开启，自有50185/50210已关闭。仅删除自己的临时curl响应（已复制进证据）；独立复核所需probe二进制按 preserved-probe.json 保留，不是Cargo缓存。没有清共享target，也没有动L29/L10保留二进制。
