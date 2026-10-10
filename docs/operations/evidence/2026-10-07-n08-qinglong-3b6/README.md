# P2/N08 批③-b 第六把：青龙偃月刀（规则 v70，2026-10-07）

主稿 P2/N08「批③-b 武器逐把」第六把：青龙偃月刀（♠5，射程 3，杀被
目标出闪闪避后可立即对相同目标再使用一张手牌中的杀）。上游对照
`card/standard.js`（qinglong 条目：attackFrom −2 = 射程 3；特效
qinglong_skill）。前置：[第五把贯石斧](../2026-10-07-n08-guanshi-3b5/README.md)。

## 本批落地

- **命名武器牌种 Qinglong（编码 100）**：equip1 武器、射程 3
  （named_weapon_range 登记 3）。
- **特效接线（ShaResolve 被闪路径）**：目标出闪闪避后（无双需求已满），
  来源装备青龙偃月刀且手牌中有杀 → 询问来源（`qinglong_replay_sha`，
  可选可放弃，min=0 空应答=放弃）→ 弃置选中的杀并**以全新状态重启杀
  结算**（card 替换、防具判定/仁王盾/闪/贯石斧/麒麟弓/青龙状态全部
  复位，Flow::Yield 重开询问）；每次被闪都可再追问，来源手牌杀尽自然
  终止；放弃则按被闪结算（无伤）。
- Sha 快照追加 `asking_qinglong` 字段（穿齐 enc/dec/check/哈希/rebuild/
  ctor/a0 样本）。
- 声明表 120→**121 张**；`STANDARD_RULESET_VERSION` 69→**70**；
  **14 golden 重录**。

## 测试（`qinglong_replays_sha_after_dodge`）

- 目标持两张闪、来源持两张杀+使用的杀：第一杀被闪 → 再杀询问（含两张
  手牌杀）→ 选第一张 → 第二杀被闪 → 再询问 → 放弃 → 目标无伤（4→4）、
  未选的杀留手牌。

## 验收口径（分开陈述）

- **代码已实现**：青龙牌种/声明表/闪后再杀语义（状态全复位重启）。
- **回归已加入**：`qinglong_replays_sha_after_dodge`。
- **golden 已重录**：14 场景全量（v70）。
- **全量通过**：engine-native **378/0/1**（唯一 ignore=写轮眼规模诊断
  探针）；a0 7/7、inventory 2/2、replay 2/2；content-runtime 32/0、
  engine-ai 29/0、loader 1/0；lib `-D warnings` 通过。
- 过程修正：插入再杀询问时曾误删 ShaResolveMachine 结构体收括号（编译
  E0425 定位）；测试侧一处断言方向写反（未选拍应留手牌）。

## 剩余

批③-b 其余三把（丈八→雌雄→方天）；模式对齐；主稿其余待做照旧。

## 改动文件

见 [source-hashes.txt](source-hashes.txt)（12 文件，相对 noname-next 根）。
