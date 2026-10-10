# E03 欢乐包独立规则载荷验收（2026-10-07，逐张对照上游）

主稿 E03 行「独立规则载荷验收」切片：欢乐包 7 张实体牌与上游
`archive/upstream/noname/apps/core/extension/欢乐卡牌/card/`（list.js /
card.js / skill.js，commit 5c02a745）逐张对照并修正偏差。前置批次：
[帧生命周期核验](../2026-10-07-e03-frame-lifecycle/README.md)。

## 对照结果与修正

| 牌 | 上游（list.js/subtype） | 原生（修正前） | 处置 |
|---|---|---|---|
| 苦无 kuwu | ♠6 equip1 武器 | ♠6 武器（range 2） | 一致（上游 range 1+attackFrom−1 等效 2，机制差异登记） |
| 手里剑 shoulijian ×2 | ♦4/♠4 基础牌 | ♦4/♠4 基础牌（ShouLiJian） | 一致 |
| 猴子 monkey | ♦5 equip5 宝物 | **♥8** | **修正 ♦5** |
| 写轮眼 xuelunyang | ♥2 equip5 宝物 | ♥2 宝物 | 一致 |
| 面具 mianju | ♣3 **equip2 防具** | **♠1、宝物栏** | **修正 ♣3 + 防具栏** |
| 酒尾 jiuwei | ♥9 equip5 宝物 | **牌缺失**（技能已声明映射） | **补齐实体牌** |

## 补齐酒尾（第 7 张实体牌）

- `CardKind::Jiuwei`：枚举/`ALL`（90）/键 `jiuwei`/宝物栏/存档编码 **89**
  （表尾追加，A0 键双射与编码往返测试同步）。
- 技能零新代码：包声明 `huanle_jiuwei` 映射既有
  `host:phase_end_recover_or_draw`（结束阶段可选：受伤回 1 否则摸 1——
  与上游 `phaseEnd` frequent 语义一致）。
- 包数据 `assets/cards.json` 追加酒尾卡 + `package.npack.json` 资产
  source_hash 更新（blake3）+ `rehash_npack` 重算逻辑/资产哈希。
- 行为测试 `huanle_jiuwei_recovers_or_draws_at_turn_end`：真实装备 →
  受伤确认回血 / 满血确认摸牌 / 放弃无事 三路径（OptionalGate 承担
  确认门）；既有 `huanle_jiuwei_optional_confirm_and_restore`（授予驱动
  + 恢复）保持全绿。

## 载荷清单守护

`huanle_pack_payload_matches_upstream_list`：7 张牌名称序/花色/点数/牌种
逐一对照上游 list.js + 装备栏断言（武器/防具/宝物）+ 真实建局装备酒尾
授予 `huanle_jiuwei` 实例。

## 语义差异登记（规则基准「欢乐包独立规则载荷验收」节）

- 苦无：上游目标自选弃牌（he 区、`card.name=="sha"` 精确、notLink）；
  原生确定性弃目标首张手牌（仅手牌、无牌静默）、`is_sha_like()` 含火/雷
  杀（超集）、不查连环。
- 写轮眼：原生无 charlotte 区分，全部技能实例可窃。
- 苦无射程：上游 range 1 + `attackFrom:-1`；原生直接 range 2（等效）。

## 验收口径（分开陈述）

- **代码已实现**：CardKind::Jiuwei 全注册点 + 包数据三处修正。
- **回归已加入**：载荷清单守护 + 酒尾三路径行为测试。
- **定向通过**：g1_huanle huanle 组 9/9（含既有 7 测试不回归）。
- **全量通过**：engine-native **360/0/1**（唯一 ignore=写轮眼规模诊断
  探针）、golden（replay_scenarios）全绿——扩展牌种不入标准 golden；
  a0_tag_registry 7/7；content-runtime 32/0；engine-ai 29/0；
  lib `RUSTFLAGS=-D warnings` 通过。
- **规则/格式版本不变**（v61/v4）：Jiuwei=89 为表尾追加，合法 v4 字节
  不受影响（golden 场景无扩展牌）。

## 未做（登记台账 U06）

跨牌组合矩阵（猴子×面具、写轮眼×据守等组合语义）；独立安装/卸载全
生命周期在真实宿主的验收（引擎级已由 C3 卸载场景覆盖）。其余扩展包
照此「逐张对照上游 list + 行为差异登记」模板分批验收。

## 改动文件

见 [source-hashes.txt](source-hashes.txt)（11 文件，相对 noname-next 根）。
