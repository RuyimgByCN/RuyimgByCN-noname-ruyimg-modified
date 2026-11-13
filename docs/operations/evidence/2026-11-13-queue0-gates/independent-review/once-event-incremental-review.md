# 第 0 项 once 与伤害臂脚本增量独立复核（2026-11-13）

- 复核人：`queue0_independent_review`；HEAD `3a6e398485db6ef0729abe3013c3c4b7705a5d6d` + tracked dirty；规则 333／NGSA 107／content schema 5／ABI 759／WS8。
- 仅检查实际脚本 diff、原生 `host.rs`／`filtering/common.rs` 和原始运行结果；未读主 agent 实现说明。只在本独立证据目录追加文件，未改产品、测试、登记表或 inventory，未占 Cargo target。

## 伤害臂 88→89 的独立因果

实际 [filtering/common.rs](../../../../../crates/engine-native-content/src/content/host/filtering/common.rs) 的 `MbcmJiushi` 在 429 行开始，430 行匹配 Damage，过滤未被防止、伤害量>0、目标为持有者、持有者背面朝上，并排除既有酒诗强化相关条件。`git blame`／提交 diff 明确来自既有 `14b2b43f7979e5b20afbf1f51ba75d4eb44a34b1`，不是本次第 0 项增加的生产规则；本次读取的 common.rs 与 HEAD 字节一致。来源原文见 [event-arms-jiushi-source-origin.txt](event-arms-jiushi-source-origin.txt)。

独立运行现有 `arms()` 得到 89 行，`(MbcmJiushi, Damage)` 恰出现一次。随后只在 Path.read_text 的内存返回值中移除酒诗完整 arm，不写源码，枚举恢复为 88 行；忽略行号偏移后其余行为／事件／文件多重集合完全相同，无其他删除／增加。证据见 [once-event-source-comparison.json](once-event-source-comparison.json)、[当前 89 行](event-arms-current.json)、[内存反例 88 行](event-arms-without-jiushi-probe.json)。

因此 [test_report_event_arms.py](../../../../../scripts/test_report_event_arms.py) 将旧断言 88 更新为 89 并明确要求包含酒诗 Damage，是对既有源码漂移的有效回归。枚举器 `report_event_arms.py` 未改；这只核对它既有的 filtering 文件／HostBehavior anchor／600 字符窗扫描结果，不把 89 行称为全部伤害技能或规则等价签署。其 `--check` 自测仍明确要求 REVIEW 行存在时 exit 1，没有把旧未核项改成通过。

## once 来源与首轮退回

真实 [host.rs](../../../../../crates/engine-native-content/src/content/host.rs) 的 `host_skill_limits` 在 291 行直接返回 match，已经没有旧脚本查找的 `let limits = match behavior` 临时变量。函数内直接 `SkillLimits { once_per_turn: true }` 的三个组含 65 个行为，SwdJiefen 在其中；其他 U01／L20 等按名字委托限制，不能把没有出现在直接允许表解读为不限次。该原生文件也与 HEAD 字节一致。

首版修补 [audit_upstream_skill_flags.py](../../../../../scripts/audit_upstream_skill_flags.py) 已锁定实际函数和直接 true 表达式，能区分 `|` 别名、直接 false、named 委托、相邻函数，并跳过 `//`。新增纯解析自测覆盖上述基础输入，首轮 5 项通过。但独立补充的三个块注释反例未通过，已要求主 agent 修复，不能用 5／0 声称全部注释边界正确：

| 独立输入 | 首版实际错误 |
| --- | --- |
| 函数内 `/* HostBehavior::Commented => SkillLimits { once_per_turn: true } */` | 把 Commented 加入允许表。 |
| 直接 false 臂内 `/* once_per_turn: true */` | 把 Unlimited 加入允许表。 |
| 真函数前有整个同名函数的块注释 | 先匹配注释假函数，仅返回 CommentedFunction，漏掉 Real。 |

原始输入／结果见 [once-block-comment-initial-probe.json](once-block-comment-initial-probe.json)、[once-commented-function-initial-probe.json](once-commented-function-initial-probe.json)。因此去注释应发生在函数定位之前；Rust 嵌套注释也须有明确处理或保留限制。初始三文件 diff 与输入哈希见 [once-event-initial-reviewed.diff](once-event-initial-reviewed.diff)、[once-event-initial-source-state.json](once-event-initial-source-state.json)。本段保留发现时快照，最终修补与再次核验另记。

## 实际独立运行

| 仓库根目录实际命令 | 结果与范围 | 证据 |
| --- | --- | --- |
| `python3 -B -m unittest discover -s scripts -p 'test_*skill_flags.py' -v` | 首版 5 passed／0 failed／0 skipped，含真实 jiefen CLI。未覆盖上列块注释反例，故未据此签署新注释边界。 | [完整输出](once-allowlist-tests.raw.txt)、[命令／输入哈希](once-allowlist-tests-command.json) |
| `python3 -B -m unittest discover -s scripts -p test_report_event_arms.py -v` | 15 passed／0 failed／0 skipped；保留 REVIEW 的负向 CLI 断言。 | [完整输出](event-arms-tests.raw.txt)、[命令／输入哈希](event-arms-tests-command.json) |
| `python3 -B -m unittest discover -s scripts -p test_audit_upstream_skill_flags.py -v` | 修后最终 6 passed／0 failed／0 skipped，包含嵌套块注释边界；与首轮 5 项分开保存。 | [最终完整输出](once-allowlist-final-tests.raw.txt)、[最终命令／输入哈希](once-allowlist-final-tests-command.json) |

这些都是轻量定向脚本测试，不替代完整脚本发现、preflight 或产品门禁；主 agent 的全量首败及最终补跑另记。本轮没有改变来源认证、版本或上游规则判定，也没有执行上游浏览器样本。

## 2026-11-13 22:12 最终修补与结论

主 agent 已修复首轮退回的注释边界：`once_allowlist()` 复用既有 [rust_effective_lines.py](../../../../../scripts/rust_effective_lines.py) 的 `strip_comments`，在函数定位前剔除行／嵌套块注释。独立读该 helper 确认其深度计数处理嵌套 `/* */`、保留换行，并避免把字符串中的注释符号当注释；helper 文件本身与 HEAD 字节相同。新纯解析测试同时覆盖注释假同名函数、嵌套注释假臂、false 臂内 true 注释，原有别名／false／委托／行注释／邻函数边界测试保留。

本 agent 独立运行修后 6 项全部通过，又逐个重放首轮三个原始 probe，结果分别只含 Real、空集、只含 Real，与预期全部相同。实际源代码的直接允许表仍为 65 个行为，未因去注释变多／变少。完整重放、最终七个输入哈希及范围说明见 [once-event-final-source-state.json](once-event-final-source-state.json)，最终三文件实际 diff 见 [once-event-final-reviewed.diff](once-event-final-reviewed.diff)。

最终结论：此次脚本修复及 89 行期望同步没有剩余阻断。`host.rs`、`common.rs`、`report_event_arms.py` 及其测试与独立 15 项运行时哈希相同，因此未重复该组。新解析仍是针对既有 Rust 排版的直接声明文本扫描，不能替代任意 Rust 语法分析，也不能据不在表中推断按名委托、条件表达式或其他路径不限次。此处 6／0 和 15／0 仅为已执行的定向脚本结果；完整 preflight、发布与 CI 状态由主 agent 新冻结后的完整输出说明。
