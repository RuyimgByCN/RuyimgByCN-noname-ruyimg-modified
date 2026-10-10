# U05 swd 第二批：流星火雨（规则 v86，2026-10-08）

swd 子包第二批转换：流星火雨（♦3/♥6/♥9 三张，锦囊·火）——目标自选弃
两张 he 牌，否则受 1 点火伤。前置：[swd 首切片](../2026-10-08-u05-swd-slice/README.md)。

## 上游语义核验（源码为准，card/swd/card.js）

`liuxinghuoyu`：单目标锦囊，`cardnature: "fire"`；content——目标
`countCards("he") >= 2` 时 `chooseToDiscard("he", 2)`（可取消），step1
若 `directfalse || !result.bool` → `target.damage("fire")`。

## 改动（分开陈述）

- **代码已实现**：`CardKind::Liuxinghuoyu`（编码 105：archive enc/dec、
  a0 variant/ALL 同步）+ `HostBehavior::SwdLiuxinghuoyuCard`（tag 79、
  `ALL` 78→79、from_tag/key/witness 全登记）+ SelectCard 两段实现——
  he≥2 时目标 SelectCards（min=max=2 可取消，`swd_liuxing_discard`）；
  取消或 he<2 → 直接 1 点火伤（`DamageNature::Fire`，铁索传导照常）；
  无懈窗口包装；**弃牌入 discard 修复**（首跑发现 retain 后缺
  `to_discard` 致 card_missing_from_all_zones）。
- **转换包**：cards.json 增 ♦3/♥6/♥9 三张流星火雨 + source_hash/哈希
  重算。

## 验收口径

- **回归已加入**：`swd_liuxinghuoyu_discard2_or_fire`——目标 3 张 he
  自选弃 2（第三张保留、无伤害）、目标无牌直接火伤（不询问）、存档
  往返。
- **全量通过**：engine-native **401 passed / 0 failed / 1 ignored**；
  依赖包 **63/0**，DEPS_EXIT=0。
- **golden**：版本 85→86 入 canonical_hash——14 场景全 mismatch，重录
  后 replay_scenarios 2/2（纯版本位移）。
- **剩余**：swd 其余 17+ 卡种逐批（草木灰/赤龙牙/归有介等多步 content
  较重，需宿主原语增补）。

## 改动文件

见 [source-hashes.txt](source-hashes.txt)（14 文件，相对 noname-next 根）。
