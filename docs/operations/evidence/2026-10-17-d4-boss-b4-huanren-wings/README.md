# 2026-10-17 D4 Boss 关卡流程 · 后援 / 还人 / 青赤白翼

**核验日期**：2026-10-17　**待做 ID**：进度主稿 T11 余项「ADR B4（bianshen4/还人/翼）」　**ADR**：[0783](../../../adr/0783-boss-stage-flow-subsystem.md)　**前置**：[B3](../2026-10-16-d4-boss-b3-sixiang3/README.md)

行为编号按 ADR 0783：后援变身是 ADR 的 B2，还人是 B3，翼轮换是 B4。进度主稿把这三块写成「B4（bianshen4/还人/翼）」。四象 3/3x 已在规则 v294 落地，本批不重做。

## 结论

从 `Engine::new_boss_stage` 起步的定向测试覆盖：

- 后援3：牛头/马面死亡后换成白无常或黑无常，同一种子结果相同。
- 后援4：黑无常/白无常死亡后换成夜叉或罗刹，同一种子结果相同。
- 还人：第一次阶段结束写入十殿、禁将表，并把被禁的非 Boss 座换成目录中第一名刘备。种子 1..80 同时出现孟婆乱入与不乱入。乱入后击杀，应答 `huanren_pick` 的首项，换成楚江王，等级 1，转关技能仍在，已乱入的孟婆被移出且不再投放。
- 等级 2 且孟婆已死、阶段数 ≤50：换成地藏王，弃区后手牌 7（摸 4 再摸 3）。孟婆未死：换成转轮王。候选里没有映姬。
- 青翼第 3/5/7 轮、赤翼第 3 轮授予赤仪2 后真实伤害 +1、白翼第 5 轮弃 2、第 7 轮前雷伤被防止、第 3 轮前敌方摸牌基础 2 变为 1。

规则 **294→295**。本批 Boss 行为没有新的存档字段（`huanren_pick` 写入已知提示词表）。同工作树的 T8 `VmAux` 随后把存档格式升到 **v80**；升档原因见 [T8 余项](../2026-10-17-t8-named-filters/README.md)。

## 技能与 tag

| 技能名 | HostBehavior | tag |
|---|---|---|
| 后援3 / 后援3·转阶 | `BossBianshen3` / `BossBianshen3x` | 427 / 428 |
| 后援4 / 后援4·转阶 | `BossBianshen4` / `BossBianshen4x` | 429 / 430 |
| 还人 / 还人·转关 | `BossHuanren` / `BossHuanrenDie` | 431 / 432 |
| 青翼 | `BossQingyi` | 433 |
| 赤翼 / 赤仪2 | `BossChiyi` / `BossChiyiMark` | 434 / 435 |
| 白翼 / 白翼·雷 | `BossBaiyi` / `BossBaiyiThunder` | 436 / 437 |

装配：牛头、马面挂后援3；白无常、黑无常挂后援4；秦广王挂还人；太昊挂青翼；炎帝挂赤翼；少昊挂白翼与白翼·雷。换将后重新授予还人与还人·转关。

## 登记差异

这些不是上游等价：

1. 没有八座换座。换将保留原座位，清区后摸 4，不把轮次打回 1。
2. 上游禁将九名里，目录能对上的只有陆逊、张春华。孙策替换池不在目录，命中后改挂标准武将表里的第一名刘备。
3. 映姬（映种）不进入还人候选。ADR 0783 保持排除，本批不建外部武将库。
4. 白翼第 5 轮弃牌是手牌然后装备的前两张，不是玩家自选。时点用轮次变为 5 后的第一次判定阶段开始近似上游 `roundStart`，并用 bit 保证只发生一次。
5. 青翼回复、失去体力，以及赤翼第 5/7 轮的火伤，是直接改体力，不逐目标发事件。赤翼第 5/7 轮和青翼第 5 轮不会把体力打到 0 以下。赤仪2 与白翼雷伤仍走真实的伤害前置窗。
6. 忘魂在 Boss 模式的任意发动都会把 `mengpo_died` 置上，不另核对该死亡角色是孟婆。
7. 等级 0 的孟婆不保留 `sixiang_add_fellow` 摸到的 4 张牌，牌按相反顺序退回牌堆。

## 版本与 golden

| 项 | 值 |
|---|---|
| `STANDARD_RULESET_VERSION` | 294→**295** |
| `ARCHIVE_FORMAT_VERSION` | 本批仍是 **79**；同工作树 T8 `VmAux` 随后升到 **80** |
| HostBehavior 数量 | **434**（最高 tag **437**） |
| 版本回退探针 | 新代码 + 规则回退 295→294 + 旧 golden → `replay_scenarios_match_golden` 通过（[probe-294-newcode.log](probe-294-newcode.log)）。位移唯一来源是规则版本字段。14 个标准场景都是 Prototype，到不了这些 Boss。 |
| 14 golden | 探针之后按 v295 实值重录（[golden-v295-actuals.txt](golden-v295-actuals.txt)）。重录后 `replay_scenarios_match_golden` 通过（[replay-golden-v295.log](replay-golden-v295.log)）。 |

## 测试

| 口径 | 结果 |
|---|---|
| 定向 `d4_boss_b4_*` | **9/9**，0 失败（[directed-b4.log](directed-b4.log)） |
| `host_behavior_keys_bijective`、`standard_skill_ids_append_only` | 通过，同 [replay-golden-v295.log](replay-golden-v295.log) |
| engine-native 全量 | 本目录未记。收口全量门禁另记。 |

命令：

```
cargo test --offline -p engine-native --test native_kernel d4_boss_b4 -- --test-threads=8
cargo test --offline -p engine-native --test replay_scenarios replay_scenarios_match_golden
```

探针把 `STANDARD_RULESET_VERSION` 临时改回 294 后复跑同一条 replay 命令，随即改回 295。
