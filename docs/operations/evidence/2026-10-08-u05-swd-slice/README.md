# U05 swd 首切片：护符 hufu + 转换包生命周期（规则 v85，2026-10-08）

U05 杀海拾遗 swd 子包（轩辕剑跨界卡 30+ 卡种）的**首个转换切片**：
护符（hufu）——上游 savable 守卫限定仅濒死自救的桃形基本牌。
前置：[wuxing 第六例](../2026-10-08-u05-wuxing-lifecycle/README.md)。

## 上游语义核验（源码为准，card/swd/card.js）

`hufu: { type: "basic", savable(card, player, dying) { return dying ===
player; } }`——基本牌，濒死结算中**只能对濒死者本人使用**（普通桃无此
限制）。swd list 中 ♠1/♣1 两张。

## 改动（分开陈述）

- **代码已实现**：`CardKind::Hufu`（编码 104：archive enc/dec、a0
  variant 表、`ALL` 104→105 同步）；`is_tao_like` 双镜像（kernel+snap）
  增 `dying: PlayerId` 参数——Hufu 仅 `responder == dying`（普通桃/酒
  不受影响）；PlayCard 门控 Hufu 需自身濒死（非濒死 `IllegalTargets`）。
- **转换包**：`packages/extensions/shahai-swd-upstream-noname`（护符
  ♠1/♣1 两张；npack 按 huanle 模板，cards.json source_hash/asset_hash
  重算；学习项：非武器卡不得声明 `range` 字段）。
- **生命周期验收**：六步同模板（安装+激活→建局→对局中卸载→锁定副本
  续玩→重装 Revoked→import 被拒）。

## 验收口径

- **回归已加入**：`swd_hufu_self_dying_only`——自救成功（回 1 点体力）
  + 他人濒死负例（座位序 1→2→0 询问中持符者 0 号即使被问、选项也不含
  护符，濒死者无援死亡）；`swd_real_host_full_lifecycle`（六步）。
- **全量通过**：engine-native **400 passed / 0 failed / 1 ignored**；
  依赖包 **63/0**，DEPS_EXIT=0。明细见 [test-logs.txt](test-logs.txt)。
- **golden**：版本 84→85 入 canonical_hash——14 场景全 mismatch，重录
  后 replay_scenarios 2/2（标准场景无护符，纯版本位移）。
- **剩余**：swd 其余 20+ 卡种（流星火雨/草木灰/赤龙牙等，含火/雷/
  poison 属性与多步 content）逐批转换；ow/xianjian/mtg/zhenfa 同前置。

## 改动文件

见 [source-hashes.txt](source-hashes.txt)（16 文件，相对 noname-next 根）。
