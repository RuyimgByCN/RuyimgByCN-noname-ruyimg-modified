# U05 ow 纵切批六：炮台（托比昂）（规则 v123，2026-10-08）

ow 纵切批六：**炮台**（paotai，托比昂主技）+ **炮台射击**（paotai2
group 子技显式化）——出牌阶段弃一张【杀】布置/升级炮台（≤3 级，跨回
合持久）；End 阶段开始按 30%×层概率对随机敌人造成 1 点火焰伤害；炮台
持有者受伤 1 点降 1 级。铆钉（maoding）暂缓（零件牌池 hslingjian 不在
原生目录）。前置：[ow 纵切批五](../2026-10-08-ow-duwen-batch/README.md)。

## 上游语义核验（源码为准，…/character/ow/skill.js paotai/paotai2/paotai3）

- `paotai`：enable phaseUse、filterCard 杀、filter 层数<3、content
  `storage++`。
- `paotai2`：trigger phaseEnd forced、filter `100*Math.random() < num`
  （层数×30%）、content 随机敌人 `damage("fire")`。
- `paotai3`：trigger damageEnd forced、filter num>0、content
  `storage -= trigger.num`（下限 0）。

## 改动（分开陈述）

- **代码已实现**：`PlayerState.paotai_lv: u8`（跨回合持久；**存档格式
  v5→v6**）+ `WudianPaotai`（tag 115）主技两段机（弃杀→lv+1 ≤3）+
  `WudianPaotaiFire`（tag 116，TriggerMeta PhaseBegin/Normal——End 阶
  段开始帧）触发臂：概率 30%×层（RuleRng）→ 随机存活敌人 1 点火焰伤
  害 + **commit_damage 炮台降级钩**（受伤 1 点降 1 级，下限 0）。
  mapping 双条目（paotai/paotai2 显式化）。
- **登记差异**：上游 paotai2 的 filter 在入队前掷骰（100*random<num）
  ——原生入队后结算时掷骰（等价概率分布）；mark intro UI 不迁移；
  maoding 暂缓（零件牌池不在原生目录）。
- **转换包**：cards.json 增 托比昂（generals 三技：paotai/paotai2 双映
  射 + maoding 暂缓 reason）+ paotai/paotai2 skills 条目，三哈希重算。

## 验收口径

- **回归已加入**：`tuobiang_paotai_stack_fire_and_downgrade`——①三次
  布置 lv 1→2→3（满级拒绝）②End 阶段 lv=3 概率 100% 必中随机敌掉血
  ③受 2 伤降 2 级（3→1）④等级跨回合持久。
- **全量通过**：engine-native **436 passed / 0 failed / 1 ignored**；
  依赖包 **63/0**（content-runtime 33 + engine-ai 29 +
  content-native-loader 1）。
- **golden**：版本 122→123 入 canonical_hash——14 场景全 mismatch，重录
  后 replay_scenarios 2/2（炮台不在标准场景，纯版本位移）。
- **剩余**：ow 其余 19 武将、xianjian 36 武将逐武将纵切；U07
  duoqi/youzhan 暂缓；zhenfa 五种阵法。

## 改动文件

见 [source-hashes.txt](source-hashes.txt)（16 文件，相对 noname-next 根）。
