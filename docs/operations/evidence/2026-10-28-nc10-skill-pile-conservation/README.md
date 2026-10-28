# NC10：模拟局中可见／本人技能牌区的实体守恒

- 实际执行：2026-10-28 09:18:59 +0800 → 2026-10-28 09:30:41 +0800（[开始](start-time.txt)、[结束](end-time.txt)）；源码基准 [`edba710802d7277fea75db8675e49efecac59592`](source-head.txt)。规则 v323、存档格式 v105、全局原生 ABI 753 不变。

## 反例与范围

静止态的 AI 规划副本从目录生成对手手牌与牌库时，原代码未排除内容自有实体牌区，能把其中一张牌再次放进手牌或牌库。干净 HEAD 独立导出中，真实刘馥筑城后从三个观察者视角遮蔽，均使牌 84 重复；真实技能测试 **4／1／0**（[红测](red-final.log)）。合成状态中，筑城、壶仓、护天三个座位，以及观察者自己的陷阱和 VM 牌堆，共 **11 个**组合均复现重复归属；对应测试 **1／2／0**。原权威对局不受副本修改影响。

本批在静止态填充时保留上游公开的筑城、炼妖壶仓和护天实体，以及观察者自己拥有的陷阱、VM 牌堆；同时保留本方私有牌区时对手手牌／牌库互换前后的合成结果和规划动作一致。上述测试按每次仅含一类受保护牌区的合法状态运行，真实筑城覆盖三个视角；多类受保护牌区同时存在由性能探针额外检查恢复。首次计时夹具混入敌方陷阱时，修改前后均无法恢复（[夹具](redaction-timing-mixed.rs)、[前](timing-mixed-before.log)、[后](timing-mixed-after.log)），因此改用仅包含本批支持牌区的同一负载测量。只修改 AI 模拟副本，不改变权威存档格式、结算规则或摘要。AI 动作可以因不再模拟重复实体牌而变化。

敌方陷阱、敌方 VM 牌堆、其他引用字段与非静止态仍未完成安全重分配；存在这些未处理牌区时，模拟副本仍可能实体重复或读取真实暗牌。特别是上游 `archive/upstream/noname-liuli/noname/character/swd.js` 的 `sxianjing.intro` 只向持有者显示陷阱身份，向他人显示张数；目前原生视图并未完整表达这些牌区。NC10 仍为部分，不能把本批记作完整信息隔离或 AI 质量验收。

## 隐私反例与收窄

开始时尝试直接保留全部五类牌区。虽然实体守恒回归通过（[定向](unsafe-targeted.log)、[全量](unsafe-full-tests.log)），但另取同一可见视图、不同敌方陷阱实体的 96 个真实技能样本，发现模拟填充池随陷阱身份变化；其中 **2 个样本原先两种输入动作相同、尝试方案变得不同**（[前](trap-before-cases.log)、[尝试后](trap-after-cases.log)、[比较](trap-comparison.txt)）。该方案未提交，最终方案改为只保留观察者自己已知的陷阱；对手 VM 牌堆也不假定公开。

同一 96 个样本在最终方案前后，逐样本的两个动作 **96／96 相同**（[基线](trap-before-final.log)、[最终](trap-final.log)、[比较](trap-final-comparison.txt)、[探针](hidden-trap-probe.rs)）；但两种陷阱输入本身在基线和最终各有 **88／96** 组产生不同动作，说明原有信息泄漏仍在，不能以“没有新增”冒充修复。新增常规回归也检查敌方陷阱及 VM 牌身份互换不会新改变合成手牌和牌库。

## 验证

| 命令／范围 | 完整结果 |
|---|---|
| `cargo test --offline -p engine-native --test nc10_skill_pile_conservation --test nc06_zhucheng_restore --test nc10_ai_information_boundary --test nc10_pending_duel_privacy --test nc10_duel_nullify_privacy --no-fail-fast` | **18／0／2**，exit 0；两项忽略为既有手工计时测试（[日志](targeted.log)）。 |
| `CARGO_INCREMENTAL=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_TARGET_DIR=<同工作树独立 target> cargo test --offline -p engine-native -p engine-native-wasm -p content-native-loader -p game-server --no-fail-fast` | engine-native **1600／0／73**、engine-native-wasm **62／0／0**、content-native-loader **32／0／0**、game-server **360／0／0**；共 **2054／0／73**、380 个测试目标，exit 0（[完整日志](full-tests.log)、[统计](test-counts.txt)、[统计脚本](count-tests.py)、[目标映射](test-artifacts.jsonl)、[target](target.txt)）。 |
| 同机同负载红／绿重分配耗时 | 同一干净导出和 target、debug、预热 200 次后各采 2000 次：标准局 p50/p95/p99 **11958／12459／15416 ns → 11916／14458／15250 ns**；五个受保护牌区同时存在 **12083／12542／15541 ns → 14833／15583／18667 ns**。后者修改前副本无法恢复、修改后可恢复，两者行为不等价；这是单次微基准而非端到端或跨设备结论（[环境](environment.log)、[探针](redaction-timing.rs)、[前](timing-before.log)、[后](timing-after.log)）。 |
| `python3 -B -m unittest discover -s scripts -p 'test_check_source_lines.py'`；`python3 -B scripts/check-source-lines.py`；`rustfmt --edition 2021 --check <本批两个测试>` | **1／0**、**8639** 文件合格、格式通过，均 exit 0（[自检](check-unit.log)、[行数](check-source.log)、[格式](check-fmt.log)）。 |
| `python3 -B scripts/check_local_links.py --all`；`python3 -B scripts/check-adr-index.py`；`git diff --check` | **1845** 文件、0 坏链，ADR 索引通过，diff 无空白错误，均 exit 0（[链接](check-links.log)、[ADR](check-adr.log)、[diff](check-diff.log)）。 |

本批前已有的八份用户文档改动保留；进度与台账只暂存本批追加行。
