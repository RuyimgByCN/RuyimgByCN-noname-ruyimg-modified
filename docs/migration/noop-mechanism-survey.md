# noop 机制族普查（2026-08-15，规划型任务产出）

> 历史参考（2026-09-16 标注）：本文描述旧线及所注明日期的状态，文中的“当前”、阶段验收和统计不代表新原生线完成度。最新入口见 [旧实现参考](../architecture/legacy-reference.md)。

> **快照声明（2026-08-21）**：本普查为一次性产出，数字不回写、不再随清单滚动。
> 当前活数字：noop 2094（`docs/content-fidelity-ledger.md`）；普查中经济性
> 排名第 1 的 expansion 族已落地为 ADR 0641（扩张系统原语，accepted）。
> 其余各族排序结论仍可供后续批次规划参考。

> 对全仓 INTENTIONAL_EMPTY_NOOP 技能按**机制**（而非按技能）聚簇，为「1 原语 + 3-5
> 技能」批次提供经济性排序。方法：提取各 noop 债务注释中的机制短语，按机制词表归类
> （一技可属多族）。明细数据：[`noop-mechanism-clusters.json`](noop-mechanism-clusters.json)。
> 普查时活清单为 2181 条 noop（见 [`../content-fidelity-ledger.md`](../content-fidelity-ledger.md)）；
> 下表 2184 / 1319 / 865 是本普查当时的机制口径，不回写清单。

## 总量修正（重要）

| 分类 | 数量 |
|---|---|
| noop 文件总数 | 2184 |
| 其中 **audio-only 终态**（上游无规则面，已逐项核实，无可重写内容） | **1319** |
| **真实债务**（有上游规则体、待原语解锁） | **865** |

此前以 2184 作为"待重写"口径**高估了 60%**——ledger 的 noop 轴混合了两种性质。

## 机制族排序（解锁技能数 ≥3）

| # | 机制族 | 解锁数 | 代表技能 | 备注 |
|---|---|---|---|---|
| 1 | **expansion 武将牌上置牌/取牌** | **165** | sp2/biaozhao2, sp/tongji, sp/fuman, sp/olbihun | 旗舰族；SkillInstance.state_blob/content_card_tags 有底座；成本最高解锁最多，与 clan README 原判断一致 |
| 2 | 判定改写/判定交互 | 127 | sp/tongji, sp/fuchao | 需判定结果任意改写（flip_judgment 现仅 Neutral） |
| 3 | 全局历史/本局游戏统计 | 91 | sp2/dccanyun, jsrg/jsrgfenjian | 需 per-game/global 历史查询面 |
| 4 | 花色/颜色匹配 | 58 | sp2/biaozhao2, sp/fenxin | 多为过滤器族（字符串、零 ABI） |
| 5 | 获得他人区域牌 | 57 | sp2/spshicai, sp/pxjuancheng | gain_target_hand_card 已有（ABI 647），扩展到装备/判定区 |
| 6 | chooseButton 多选/双选 | 46 | sp2/dcmoukui, jsrg/jsrgfenjian | 通用多选原语 |
| 7 | 牌堆顶/牌堆操作 | 39 | sp2/spshicai, sp2/mubing | 观顶+取顶族 |
| 8 | 手牌展示/明置 | 29 | sp2/mubing, sp/fenxin | gaintag 展示追踪 |
| 9 | 交给/交出牌 | 29 | sp2/mubing, clan/clansankuang | give 原语族 |
| 10 | 多目标/依次结算 | 25 | xianding/xinyingbing, ddd/sunliang | |
| 11-33 | 点数/弃牌堆/攻击范围/免疫/观顶/体力上限/装备区/交换/手牌上限/随机/横置/跨技能/属性转换/转换技/拼点/阶段跳过/防具/回合外/重铸/宗族 | 3-24 | | 单击清单见 [`noop-mechanism-clusters.json`](noop-mechanism-clusters.json) |

低于门槛（<3）：refreshSkill(2)、虚拟普通锦囊(2)、额外结算(1——已由 ABI 649 覆盖)。

## 下一步批次建议（按解锁/成本比）

1. **花色/颜色匹配过滤器族（58 技，零 ABI 字符串过滤器为主）**——性价比最高：多为
   trigger filter 表达（本会话已验证 5 处注册模式 ×N 个过滤器），无需引擎状态。
2. **获得他人区域牌扩展（57 技）**——gain_target_hand_card 管线已通，扩展 hej 区。
3. **expansion 系统（165 技）**——旗舰但成本最高，建议独立 ADR 立项后专项会话实施。
4. 判定改写（127）/全局历史（91）次之，均需引擎查询面扩展。

## 与会话终局状态的关系

本普查不改变已完成事实（迁移 100%、当日 10 技+2 原语+3 过滤器、3923 测试全绿），
只修正"剩余量"口径（2184→865 真债务）并给出后续批次的经济性排序。
