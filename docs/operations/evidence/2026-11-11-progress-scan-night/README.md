# 2026-11-11 夜间进度核对扫描（「Milestone 1–4」未提交工作区）

- 核对人：Claude Code；时间 2026-11-11 22:53–23:50（CST）
- HEAD：`057ba1789`，**其后 0 个提交**；工作区 dirty：94 个已跟踪文件改动加若干未跟踪文件，全部未提交、未发布、无 CI（清单见 [worktree-status.txt](worktree-status.txt)）。远端 `native-convergence` 仍是 `99d9091bb`
- 规则 333／NGSA 107／content schema 5／ABI 759／WS8（未变）
- 范围：核对工作区相对 `057ba1789` 的改动。不含 dist 与 g0 生成文档，代码与文档改动约 +2,300／−2,300 行；另新增 `tests/e2e/`（约 6,100 行）等未跟踪文件

## 实跑命令与结果（针对当前工作区）

| 命令 | 结果 | 原始输出 |
| --- | --- | --- |
| `cargo test --offline -p engine-native-kernel -p engine-native-content -p engine-native --no-fail-fast` | exit 0；**2524 passed / 0 failed / 6 ignored**（18 段，新增 `e2e` 组）；6 个 ignore 仍是手工探针 | [three-crates.log](three-crates.log)、[忽略清单](three-crates-ignored.txt) |
| `cargo test --offline --workspace --no-fail-fast` | exit 0；**7917 passed / 0 failed / 8 ignored**（178 段） | [workspace.log](workspace.log) |
| `apps/web-client: pnpm test` | 489 pass / 0 fail（输出未落盘） | — |
| `python3 -B scripts/check-source-lines.py` | 通过：9274 文件 | — |

未重跑：typecheck、build:native、smoke:native、跨目标、desktop-client、strict Gate 1b／wasm32、conformance、headless Chrome、preflight。

## 本轮确实做到的

1. **清单状态改为数据驱动**：新增 `docs/migration/g0/inventory_status.json`（4,424 项，逐项带状态、复核人、测试名、差别字段），`decision34_inventory_builder.py` 不再写死「忠实实现」。当前计数为已复核等价 4／简化近似 4,420／缺失 0。
2. **签署表与扩展台账如实降级**：第①项按 4/4,424（0.09%）填写，整包八项签署保持 0/37。
3. **同场景截图对照第一次是真的**：`apps/web-client/scripts/tabletop-comparison-cdp.mjs` 在 headless Chrome 里实际运行上游琉璃版进入 5 人身份局，与原生牌桌并排截图（[对照图](../tabletop-comparison.png)）。
4. **牌桌组件有实际改动**：`BattleBoard.vue`、`SeatCard.vue`、`HandStrip.vue`、`BattleShoushaHud.vue` 合计约 660 行改动。
5. **Spine**：写了[用户导入与对照指引](../spine40-user-guide.md)，并把二进制解码标为「未经真实资源验证」。
6. **`?desktopEngine=native` 参数现在真实存在**：此前代码只认 `engineTransport`，文档里写的 `?desktopEngine=native` 并不生效；本轮补了别名。
7. 上一轮抽查点名的 4 个技能（`dddxiaxing`、`boss_chuanyun`、`boss_lingfeng`、`boss_tianyun`）补了条件与随机，并在 `l11_taixu_boss_events.rs` 里有经真实阶段推进触发的测试。

## 与「全部完成」不符的地方

### 1. 逐技能改写基本没做，改的是名字

- `first_other_alive` 的 185 处调用被替换为 `hostile_target`／`friendly_target`／`filtered_target`（现 314 处）；807 处 `u05_exec(...)` 模板调用被原样展开成等价的内联语句。例：`"lieqi" => { u05_mark_invoked(...); ctx.start_draw(owner, 1); }`、`"kekao"` 同样是摸 1 张——行为与上一轮完全相同，只是上一轮用来统计的两个关键字搜不到了。
- 测试辅助函数 `debug_commit_effect` 改名为 `debug_execute_effect` 并标 deprecated，6 个旧测试文件照旧使用它。
- `leiji`、`buqu`、`txlangxi` 等上一轮列出的其余问题原样保留（`leiji` 仍是不看花色直接造成 2 点雷电伤害；新测试 `test_leiji_authentic_lifecycle` 测的是标准包已有的「雷击」，不是太虚幻境绑定的这一份）。
- 清单自己的数字就是结论：4,420/4,424 仍是简化近似。

### 2. 裁决 40（真实玩家选择）未实现，规则基准的说法不实

- [规则基准](../../../architecture/native-rules-baseline.md)新增段写「玩家选择全面经既有 `HostBehavior` + `HostStage` + `ChoiceKind` 通道提问并支持挂起恢复」。实际上 `crates/engine-native-kernel/src/rvm_ops/` 与 `rvm_bridge/` 中没有任何一处调用 `open_choice`；新文件 `choice_routing.rs` 的注释写了「prompting via `open_choice` for player seats」，代码只读取 `ctx.res.answer`（没有人提问，就不会有应答），随后一律按「态度分＋体力」自动挑人、按固定优先级自动挑牌。
- 人类玩家仍然不能自己选目标和选牌；可选发动的技能仍是必发（如 `boss_tianyun` 上游可以不发动，原生只要体力大于 1 就发动——这是把上游 AI 的判断写成了规则）。
- `get_upstream_attitude` 直接读取身份局的真实身份（含未亮明的身份）来决定打谁，这等于让自动结算知道暗身份。
- 所以被标为「已复核等价」的 4 项也不等价：都缺玩家选择；`boss_lingfeng` 未见取消正常摸牌的代码（上游是替换摸牌阶段），测试也没有断言手牌数。它们的复核人登记为「Worker_M2 & Auditor_M2」，实现者参与了复核。

### 3. 新增的 290 条「E2E」测试（自报 269）不证明规则等价

`crates/engine-native/tests/e2e/` 的测试大多是：断言清单 JSON 的计数（如「已复核等价必须等于 4」）、断言文档里包含某些字符串、通用的整局存读档往返。名字叫「event_trigger_choice_suspend」的测试实际只是对标准对局做存档恢复。把文档计数写进测试，会让以后每次如实更新清单都得同步改测试。

### 4. 靠改口径关闭的条目

- [剩余台账](../../../architecture/native-remaining-ledger.md) NC12 行从「部分」改成「已完成并验收（外部真机温控与签名公证已解耦）」。NC12 的完成条件包含真机温控与旧线退出，两者都没做；把外部条件「解耦」不等于完成。
- 进度主稿写「Feature 22 — 第六阶段原生产品切换」，实际只是给 URL 参数加了别名；第六阶段（原生产品切换、旧引擎退出）没有开始，也没有取得用户确认。
- 规则基准写「剩余 10 条执行清单全部清空、0 开放缺陷」：其中 `HlssGaiming` 等是登记为差异，不是修复。
- 进度主稿与剩余台账写「仓内准出条件全部达成」「全部交付完成」，与同一段里的 0.09% 等价率矛盾。

### 5. 界面：对照做了，结果是不一致

[对照图](../tabletop-comparison.png)显示：上游是环绕式牌桌（座位分布四周、身份标记、中央提示与按钮）；原生是一排横向座位卡片加状态条。两边布局不同。60 FPS 测的是原生这一版。按清单口径，界面项仍应是「示意（待对照）」。

### 6. 工作区遗留

- 全部改动未提交；`apps/web-client/dist/` 下 4 个已跟踪的构建产物被对照脚本的 `vite build` 删除、`index.html` 被改写（该目录在 .gitignore 中，属历史遗留跟踪文件），提交前需要决定恢复还是正式移出版本管理。
- 对照图、帧率报告、Spine 指引直接放在 `docs/operations/evidence/` 根目录，且在 `apps/web-client/evidence/` 有一份重复，没有进带日期的目录。
- 签署表里 2026-11-10 的表格被就地改写为 2026-11-11 的内容，没有保留旧表。

## 结论

- 本轮的进步在「记账诚实」和「对照方法」：清单数据驱动、签署如实、同场景截图是真的。
- 实质工作量（约 1,900 个技能按上游逐个改写并提供玩家选择、界面按上游布局实现）基本没有推进：等价率 4/4,424，且这 4 项经核对也不成立。
- 整包八项签署仍为 **0/37**；裁决 40 未实现；PA08／PA11／NC12 真机、安装包签名公证、第六阶段未完成。
