# 2026-10-11 T11 掠影子项：可选发动 + "另有其他有牌角色"过滤（规则 v194，存档 v42）

任务来源：[进度主稿 T11](../../../architecture/native-progress.md)（10-10 待办整理核对 §1 附带发现的未登记差异）；开工 HEAD `220273ec`（领取锁）。上游：`archive/upstream/noname/apps/core/extension/杀海拾遗/character/xianjian/skill.js:1910`。

## 上游语义（v154 原生漏登记的两点）

1. **filter 双条件**：目标 he 有牌 **且** `game.hasPlayer(current ≠ player ∧ current ≠ target ∧ current.countCards("he"))` ——缺第三名有牌者则整个技能不可发动（连获得也不触发）。原生原实现只要目标有牌就自动发动，无第三者时跳过受害者段但仍获得。
2. **非强制**（trigger 无 `forced`）：使用者确认后才发动；`usable:1` 计确认次数——放弃不消耗，本回合下一杀可再询问。原生原为自动强制 + 询问前置位 `lueying_used`（放弃也消耗，且无确认门）。

## 改动内容

| 文件 | 改动 |
|---|---|
| `kernel.rs`（shaBegin 钩子） | 补 `others` 过滤进发动条件；改为确认门（`lueying_confirm`，Confirm/空应答，observer=使用者） |
| `kernel.rs`（ShaResolve 恢复臂） | 两相协议：Confirm=置 `lueying_used`（确认后消耗）+ 获得目标 he 随机一张 + 仍有其他有牌者则转入选人相（原 victim 相不变）；空=放弃（置 `lueying_declined`，不消耗，重开闪询问）。相以应答形状区分（确认相只会收到 Confirm/空，选人相只会收到 Player） |
| `kernel.rs` 机器结构 | `ShaResolve` 尾部追加 `lueying_declined: bool`——放弃后阻止 step 重入钩子再问（实测无此标记会无限循环重开确认门）。接线 9 处：结构体/`new()`/工厂字面量/`snapshot()`/`standard.rs` 脚手架/`engine.rs` 测试夹具与哈希编码（`push_machine` 尾部追加字节）/`snap.rs` 变体与解码/archive 编解码 |
| `archive.rs` | **`ARCHIVE_FORMAT_VERSION` 41→42**（Sha 机器编码尾部追加 `lueying_declined: u8`；v41 字节被版本门禁拒绝，无生产迁移数据） |
| `content/standard.rs` | `STANDARD_RULESET_VERSION` 193→**194** |
| `tests/g1_wudian.rs` | 既有回归加确认门步；新增 `muchanglan_lueying_no_third_party_never_fires`、`muchanglan_lueying_decline_keeps_usable_for_next_sha`（放弃后同回合第二杀确认门再现 → Confirm → 获得 → 选人 → 受害者弃牌全链） |
| `tests/replay_scenarios.rs` | 14 golden 重录 + 重录依据注释 |

## 开发过程记录（同做法失败一次后的修正）

第一版复用 `asking_lueying` 做两相（零格式变化），实测死循环：放弃后 step 重入，钩子条件仍成立 → 确认门无限重开（原实现靠问前置位 `lueying_used` 挡重入，而 v194 语义要求放弃不置位）。改为显式 `lueying_declined` 机器标记 + 存档版本递增。

## 验证

| 命令 | 结果 |
|---|---|
| `cargo test --offline -p engine-native --test g1_wudian muchanglan` | **3/3** |
| 双版本回退探针（194/42→193/41 复跑 `replay_scenarios`） | **2/2 全绿**（恢复 194/42 后重录） |
| `cargo test --offline -p engine-native --no-fail-fast` | **529 通过 / 0 失败 / 1 ignore**（527+2，exit 0） |
| `RUSTFLAGS="-D warnings" cargo check --offline -p engine-native --lib`（独立 target 目录，用完删） | **exit 0** |

## 边界

- 掠影其余上游细节已对齐；`feixia`（虚拟用牌）维持暂缓登记。
- 存档 v42 仅追加字节、无迁移数据；旧 v41 存档按版本门禁拒绝（既有口径）。
- T11 其余项（模式变体、事件/随机流/恢复轨迹对照、已登记逐技能差异）仍为长期待做。
