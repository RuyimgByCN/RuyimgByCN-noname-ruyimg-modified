# 2026-10-29 NC07 字段批：10 个「入档未入摘要」字段全部入摘要（规则 v327）

范围：[裁决登记](../2026-10-28-user-decisions/README.md)第 11 项——对[字段审计](../2026-10-28-nc07-lianyaohu-hash/archive-vs-hash-field-audit.log)所列 10 个字段另立一批、全部共用一个版本号；逐字段写反例；证明不影响结算的字段列入排除名单并写明理由；加结构守卫测试。**规则 v326→v327（本批独占）**；v326 及更早旧档按版本门禁拒绝。存档 **v105**、全局原生 ABI **753**、协议不变。开始 [start-time.txt](start-time.txt)，结束 [end-time.txt](end-time.txt)，源码基准 HEAD `95f51867`（[source-head.txt](source-head.txt)，B8/v326/GT-014 批同工作树之上）。

## 1. 字段清单与排除名单

| 字段 | 归属 | 规则路径读取（语义） | 处置 |
|---|---|---|---|
| `jiguanyuan_cards` | PlayerState | swd 机关鸢待获得牌（play_card 目标合法性、ai 候选） | 入摘要 |
| `jileng_active` | PlayerState | swd 金极冷冻：杀不可被闪避（play_card/play_sha）、回合清零 | 入摘要 |
| `chaofeng_active` | PlayerState | swd 嘲讽旗标：play_sha 目标强制、判定阶段开始清除 | 入摘要 |
| `xingtigaizao_count` | PlayerState | swd 形体改造层数：手牌上限削减（state/core_ops.rs:831）、阶段清零 | 入摘要 |
| `yinmi_active` | PlayerState | swd 隐秘立场：目标合法性（play_card 两处） | 入摘要 |
| `xuehuang_used` | PlayerState | swd 雪煌 limited 已发动闸 | 入摘要 |
| `xuejibingbao` | PlayerState | swd 血祭冰瀑：摸牌阶段 +1 张、逐次递减（phase/draw） | 入摘要 |
| `nigong_lv` | PlayerState | swd 拟攻等级 0–4（装备命中累计、strikes 读取） | 入摘要 |
| `card_mods` | PlayerState | ADR 0780 卡牌实例改造表：目标合法性与攻距（swd_card_effects） | 入摘要 |
| `minwan_targets` | TurnState | 悯婉目标集：伤害结算收集、事件分派判读、回合清空 | 入摘要 |

**排除名单为空**：十个字段全部在规则路径被读取（审计末尾引用计数 + 上表出处），无一属纯展示缓存。守卫测试的排除名单机制保留，供未来真正的非语义字段登记并写明理由。

## 2. 反例（先红，[red-counterexamples.log](red-counterexamples.log)，v326 源码）

`nc07_archive_hash_fields`：同一引擎的同一快照，**仅在恢复前把该字段改为非默认值**——两份存档都合法、可恢复、其余逐字相同；修复前 10/10 全红（`canonical_hash` 给出相同摘要＝GT-012 缺陷），修复后 10/10 全绿（摘要不同）。反例取值均为合法值（如 `nigong_lv=2`、`minwan_targets=[2]`、`card_mods` 插入牌堆顶实体）。

## 3. 改动

| 位置 | 改动 |
|---|---|
| `engine/hash.rs` | 玩家段（壶仓块后）按 `state_encode.rs` 编码序写入 9 个玩家字段；回合段（`bte_used_play_phase` 后）写入 `minwan_targets` |
| `content/standard.rs` | `STANDARD_RULESET_VERSION` 326→**327** + 版本史注释 |
| `tests/nc07_archive_hash_fields.rs`（新增） | 10 个逐字段反例 + v326 夹具拒绝（[native-v326.ngsa](native-v326.ngsa)，版本号临时回退采集后恢复，一次性测试已删） |
| `tests/nc07_archive_hash_guard.rs`（新增） | 结构守卫：`state_encode.rs` 写档的 PlayerState（`p.`）/TurnState（`t.`）字段必须出现在 `engine/hash.rs`（入摘要）**或**排除名单（含理由）；名单成员若已入摘要则测试失败（防名单过期）。口径与审计脚本一致 |
| 版本断言同步 | `nc04_vm_resume.rs`、`nc07_lianyaohu_hash.rs`、`pa12_tag656_version.rs`、`nc04_cards_given_event.rs` 共 7 处 `catalog`／`^` 期望 326→327 |

替代方案：只入部分字段（不取——审计清单内无一可排除，且裁决要求共用一个版本号）；按结构体批量派生哈希（不取——保持现有显式逐字段口径与 PA03 缓冲方案兼容）。

## 4. golden：三步探针归因（全部 14 场）

1. **修复 + v327**：14 场全位移（重录值 [golden-hex-v327.txt](golden-hex-v327.txt)）。
2. **回退 v326 保留修复**：仍 14 场全位移（[step2](golden-probe-step2-fix-v326.log)）——新增哈希写入**无条件**（默认值也进入摘要域，与 v325 壶仓空长度字节同理）。
3. **回退 hash.rs 修复（版本 326）**：14 场全绿（[step3](golden-probe-step3-revert-fix.log)）——金面只受 hash.rs 与版本号影响，无行为变化；行为面由全量 1639/0/73 佐证。

重录后 3/3（[golden-after-rerecord.log](golden-after-rerecord.log)）；`replay_scenarios.rs` 注记 v327 重录原因。

## 5. 回归与影响面（2026-10-29 01:05–01:25，HEAD `95f51867` + 本批工作树，规则 v327／存档 v105／ABI 753）

| 命令 | 结果 | 日志 |
|---|---|---|
| 定向（字段/守卫/版本同步六套件） | **10+1+7+9+2+1 / 0** | [green-targeted.log](green-targeted.log) |
| `cargo test -p engine-native --no-fail-fast` | **1639/0/73**（336 二进制；1627+12） | [engine-native.log](engine-native.log) |
| `cargo test -p engine-native-wasm --no-fail-fast` | **71/0/0** | [engine-native-wasm.log](engine-native-wasm.log) |
| `cargo test -p content-native-loader --no-fail-fast` | **32/0/0** | [content-native-loader.log](content-native-loader.log) |
| `cargo test -p rule-vm --no-fail-fast` | **21/0/0** | [rule-vm.log](rule-vm.log) |
| `cargo test -p game-server --no-fail-fast` | **362/0/0** | [game-server.log](game-server.log) |
| `cargo check --workspace --all-targets` | exit 0，0 错 | [workspace-check.log](workspace-check.log) |
| Gate 1b（`-D warnings`） | exit 0 | [gate1b.log](gate1b.log) |
| `build:native` + `smoke:native` | 重建 exit 0；**81/0** | [build-native.log](build-native.log)、[smoke-native.log](smoke-native.log) |
| `pnpm --dir apps/web-client test`／`typecheck` | **127/0**／0 错 | [web-test.log](web-test.log)、[web-typecheck.log](web-typecheck.log) |
| 行数门禁 | **8660** 文件 ≤2000 行 | [source-lines.log](source-lines.log) |

## 6. 兼容性与未覆盖项

- 规则 **v327**（裁决第 11 项，全部字段共用）；存档 v105（归档编码不变，只是摘要域扩张）；ABI 753、协议、Worker 不变。v326 及更早旧档一律版本拒绝（v326 夹具回归；v322–v325 夹具随既有断言继续拒绝）。
- 十个字段的**归档往返**本就存在（审计以入档为前提），本批不改变编码；语义行为不变（全量绿）。
- 未覆盖：`GameStateCore` 顶层与 `vm_aux` 等其他结构体的同类审计（守卫目前覆盖 PlayerState/TurnState 两个最大面，与审计口径一致）；十个字段的整局技能级 E2E（属各技能族批）。
