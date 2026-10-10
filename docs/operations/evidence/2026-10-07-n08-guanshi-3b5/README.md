# P2/N08 批③-b 第五把：贯石斧（规则 v69，2026-10-07）

主稿 P2/N08「批③-b 武器逐把」第五把：贯石斧（♦5，射程 3，杀被闪后
来源可弃两张牌强制伤害）。上游对照 `card/standard.js`（guanshi 条目：
attackFrom −2 = 射程 3；特效 guanshi_skill）。前置：
[第四把诸葛连弩](../2026-10-07-n08-zhuge-3b4/README.md)。

## 本批落地

- **命名武器牌种 Guanshi（编码 99）**：equip1 武器、射程 3
  （named_weapon_range 登记 3）。
- **特效接线（ShaResolve 被闪路径）**：目标出闪闪避后（无双需求已满），
  来源装备贯石斧且可弃池 ≥2（手牌+装备、**不含贯石斧本体**）→ 询问
  来源恰选两张（`guanshi_force_damage`，min 2/max 2，可放弃）→ 恰弃
  两张才强制 1 点伤害；放弃无伤。
  **登记差异：强伤不带裸衣/酒加成，固定 1 点**（上游同为此口径——
  guanshi_skill 为独立伤害）。
- Sha 快照追加 `asking_guanshi`/`guanshi_cards` 字段（穿齐 enc/dec/
  check/哈希/rebuild/a0 样本/构造点）。
- 声明表 119→**120 张**；`STANDARD_RULESET_VERSION` 68→**69**；
  **14 golden 重录**。

## 测试（`guanshi_forces_damage_after_dodge`）

1. 弃二强伤路径：目标出闪 → 贯石斧询问（池含两张手牌、不含贯石斧
   本体）→ 恰选两张 → 强制伤害（被闪仍命中）。
2. 放弃路径：目标出闪（第二张闪保证真实触发被闪）→ 放弃（空应答）→
   不强制伤害（hp 不变）。

## 验收口径（分开陈述）

- **代码已实现**：贯石斧牌种/声明表/闪后弃二强伤语义（可放弃）。
- **回归已加入**：`guanshi_forces_damage_after_dodge`。
- **golden 已重录**：14 场景全量（v69）。
- **全量通过**：engine-native **377/0/1**（唯一 ignore=写轮眼规模诊断
  探针）；a0 7/7、inventory 2/2、replay 2/2；content-runtime 32/0、
  engine-ai 29/0、loader 1/0；lib `-D warnings` 通过。
- 过程修正：首版挂点误插于伤害分支（字符串替换匹配错位）导致闪避结算
  结构被破坏——按原双分支结构重建，贯石斧询问移入被闪路径（无双判断
  之后），快照接线（enc/dec/check/哈希/rebuild/构造点五处）补齐。

## 剩余

批③-b 其余四把（青龙→丈八→雌雄→方天）；模式对齐；主稿其余待做照旧。

## 改动文件

见 [source-hashes.txt](source-hashes.txt)（13 文件，相对 noname-next 根）。
