# P2/N08 逐技能语义对照批五：自选类遗留收尾（规则 v79，2026-10-08）

主稿 P2/N08「逐技能语义对照」第五批：三卡自选类登记差异消除，主线
（标准包 + 杀海拾遗单点差异）登记清账。前置：
[逐技对照批四](../2026-10-08-n08-skill-parity-4/README.md)。

## 上游语义核验（源码为准，杀海拾遗包）

- **暴雪药水** `gw_baoxueyaoshui`：`target.chooseToDiscard("h", 2, true)`
  ——目标**自选**两张手牌（forced，不足则全选）弃置后 `target.draw()`。
- **冰魄通** `bingpotong`：step0 双方须有手牌（否则该目标 finish）→
  使用者 `chooseCard("请展示一张手牌", true)` → 目标 `chooseCard` →
  同色：使用者弃自己展示牌 + `target.loseHp()`（v77 原语已落）；异色
  无事；**展示不移动牌**。
- **龙须钩** `longxugou`：`player.choosePlayerCard("e", target, true)`
  ——**使用者**自选目标装备区一张获得（装备区明牌；原登记「目标自选」
  系措辞不准，本批更正）。

## 改动（分开陈述）

- **代码已实现**：暴雪（SelectCards 问目标，min=max=min(2,手牌数)，
  无手牌跳过弃牌仍摸 1）；冰魄通（复用拼点双段 stage
  ComparePlayerPick/CompareTargetPick，逐目标队列，使用者展示牌暂存
  `cards` 入快照可恢复，同色结算走 v77 LoseHp）；龙须钩（SelectCards
  问使用者选目标装备）。三询问入 active_ask 派生表；创建 stage 按
  behavior 特例（共享创建点不动其他卡牌）。
- **登记更正**：龙须钩差异方向更正（使用者选非目标选）；
  暴雪/冰魄通 v78「更正归类（自选类）」行注记 v79 消除。

## 验收口径

- **回归已加入/更新**：冰魄通测试改双段应答（含「使用者无手牌后续
  目标跳过」上游分支覆盖 + 展示牌保留断言）；龙须钩加选牌询问断言；
  暴雪测试验证自选语义（选 s2/s3、s1 保留）。
- **定向通过**：`g1_shahai_gujian` 13/0、`g1_shahai_gwent` 6/0。
- **全量通过**：engine-native **382 passed / 0 failed / 1 ignored**
  （唯一 ignore=写轮眼规模诊断探针），单线程
  `cargo test --offline -p engine-native -- --test-threads=1`；
  依赖包 content-runtime + engine-ai + content-native-loader
  **62 passed / 0 failed**，DEPS_EXIT=0。明细见 [test-logs.txt](test-logs.txt)。
- **golden**：版本 78→79 入 canonical_hash——14 场景全 mismatch，重录
  后 replay_scenarios 2/2（无场景触及三卡，纯版本位移）。
- **上游等价边界**：本批消除三卡自选类差异；随机类保留登记七项
  （弹射之刃阵营链/六骰格效果池/舌战/草船/金莲烛/卤玉羹等结构差异）
  与玉衡升级机制仍登记；不构成 G1 整包等价证明。

## 改动文件

见 [source-hashes.txt](source-hashes.txt)（9 文件，相对 noname-next 根）。
