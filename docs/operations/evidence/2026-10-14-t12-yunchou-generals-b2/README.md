# T12 运筹武将面批 2：雄姿 + 雅量（diy_sp_zhouyu 整将转正，2026-10-14）

## 任务来源

[T12 运筹武将面](../../../migration/upstream-extension-reconstruction.md)批 1 续批：
diy_sp_zhouyu（周瑜·SP，吴 3 血）两技全部转正——**批 2 达成首个整将完成**。

## 上游语义（杀海拾遗/character/yunchou/skill.js）

| 技能 | 上游 | 原生承载 |
|---|---|---|
| 雄姿 xiongzi（强制） | 摸牌阶段开始摸牌数 +1+⌊装备数/2⌋ | `HostBehavior::YunchouXiongzi`（tag 243）：PhaseBegin(Draw) 触发（固定摸 2 后补摸——阶段技后执行口径）；三档实测 1/1/2（0/1/2 件装备） |
| 雅量 yaliang（可选） | 你造成或受到伤害后，伤害双方各摸 1（双方存活且异人） | `HostBehavior::YunchouYaliang`（tag 244）：Damage 触发 + 双存活异人 filter + optional 确认门；双方经 targets=[source,target] 提取，座位序逐个摸（登记差异：上游 sortBySeat 等价） |

## 交付

- host.rs +2 变体（tag 243/244、ALL 244、from_tag/key/trigger_meta/filter/step/
  (target,card)/targets 提取——**8 触点一次到位**，批 1 第 9 触点教训落实）。
- character.json 追加 diy_sp_zhouyu（累计 3 名武将、4 行为、unmapped 仍 2）。
- 测试 g1_shahai_yunchou +1（批 2）：桥接累计断言、目录元数据（周瑜·SP 2 技
  3 血）、雄姿三档补摸（增量断言：Judge→Draw 差=固定 2+补摸）、雅量真实出杀
  通道双摸。

## 实测记录

| 项 | 结果 |
|---|---|
| 反例实证 | Xiongzi/Yaliang filter 桩化恒 false → 批 2 测试 **FAILED**（0/1） |
| 定向 | g1_shahai_yunchou **15/15**（批 1 13+批 1 新+批 2 新） |
| engine-native 全量 | **729/0/0**（golden 不变）——[日志](engine-native-tests.log) |
| 装载/包管理/服务端 | **84/0** |
| Gate 1b | exit 0——[日志](gate1b-all-targets.log) |

## 登记边界与下一步

- 运筹武将面进度 **3/10 名、4/17 技**（diy_caocao 1/3+2 暂缓、diy_weiyan 1/1、
  diy_sp_zhouyu 2/2 整将完成）。
- 批 3 候选：diy_luxun（shaoying 多步选牌亮牌+zonghuo 伤害 nature 改写——后者
  需勘察 DamageBefore 载荷 nature 写回面）。
