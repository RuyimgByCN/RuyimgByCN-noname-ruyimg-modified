# T13 信息隔离跨模式验收·第二批（2026-10-14）

## 任务来源

[第一批](../2026-10-14-t13-info-isolation/README.md) 的余项收敛：「未建模模式（KOF 连续会话/国战完整版）信息隔离随各自模式批」——KOF 与国战模式入口在引擎已有建模（`new_kof`/国战 mode tag），本批补跨模式隔离断言。

## 交付（t13_info_isolation.rs 追加 2 测）

| 模式 | 断言 |
|---|---|
| KOF 连续会话（`new_kof`） | 2 人局对手手牌 Hidden |
| 国战（Kingdoms mode tag） | 4 人局他人手牌 Hidden（与标准原型同构） |

## 实测记录

| 项 | 结果 |
|---|---|
| 定向 | **9/9** 绿（7 既有+2 本批）——[日志](isolation-targeted.log) |
| engine-native 全量 | **715/0/0**（713→715）——[日志](engine-native-tests.log) |
| Gate 1b | exit 0——[gate1b-all-targets.log](gate1b-all-targets.log) |

## 登记边界

- 已建模模式信息隔离验收至此**全部达成**（原型/身份/对决 2v2/斗地主/Boss/KOF/国战 = 7 形态）；后续新建模模式随各自模式批补隔离断言。
- 客户端 UI 仍需浏览器 harness（归产品面批）。
