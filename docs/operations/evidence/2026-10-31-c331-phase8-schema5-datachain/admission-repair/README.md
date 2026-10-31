# C8实际入场修复：限定私有接纳（2026-10-31）

两P1和一P2已真实修复并经[源码复审](source-review.raw.txt)关闭；父独立核验原始结果和源码绑定后，仅接纳C8私有声明／入场基础。**不合Main、不表示Core消费者或整C331完成**；[原C8 BLOCK](../source-review-v1.raw.txt)保留原字节与当时结论。

[实现报告](implementation.raw.txt)及[完整绑定](results-bound.json)记录真实入口：SDK先传原始JSON文本给既有深校验；Rust完整包复用原文typed reader，数组用serde RawValue；无modules的声明载荷拒绝而普通资产包保持；trigger／active共用executor互斥校验。未加JSON框架、wire请求或新依赖版本，未改正式规则／restore／hash。

- [最终7清单](final-freeze/source-manifest.json) SHA `969a7cee6849c7880b853d94e09f166bf4842b68f93f26d59989518e1ba72345`；[完整170清单](full-source/source-manifest.json) SHA `f5829f4023ee465e5e97fe6ad8f998b1480999cd727c348b310686dcdf72876e`。
- [父独立解析](parent-independent-results.json)核170当前源、7增量、19份raw计数、checkpoint→final仅独立smoke变动、入场11坏输入及旧3/4正控。另[父artifact核验](parent-artifact-index-fresh.json)核91证据／报告及私有匹配node-WASM6项；没有重跑测试。
- 四内容包609/0/0、原生三包1773/0/75、WASM相关13/0、前端186/type、独立目标Gate1b `-Dwarnings`、行数9021／自测2／分层0通过；重叠范围不相加。旧3/4原字节、hash、实际导入／装载／恢复正控保持。
- 当前私有nodejs release `--no-opt` WASM SHA `db64d471c60afec3e003d61dcb1539b801054ea1f0ba029cfbab2d27bdc92fdd`，用于真实SDK local-store／直接validate／new／restore的11变异。每条坏输入拒绝后原store、handle、view、hash和archive保持；它不是Main生产web bundle或GUI证明，匹配产品仍私有按[身份清单](wasm-identity.json)保留。

追加正控曾误断言合法restore前后archive相等。真实decoder显示唯一typed变化auth_generation 0→1，其他完整状态相等；字节差仅该字段和32字节payload校验和，view与state hash保持。原失败保留在actual-sdk-wasm2.log，最终按恢复后基线验证拒绝原子性。第三次smoke的同版本内容冲突也是harness输入问题，保留原红后用独立空store正控，未改产品冲突规则。

C9实际Damage结算在后续隔离阶段继续；provider／expiry／death／reveal／rescue／cleanup／SkillInvocation及R05工作仍按[裁决29](../../2026-10-31-c331-runtime-contract/README.md)完成。来源262/290、剩28及全37范围不变，当前状态统一见[进度主稿](../../../../architecture/native-progress.md)。
