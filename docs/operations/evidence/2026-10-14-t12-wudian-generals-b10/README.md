# T12 玩点论杀武将面续批：屯田·DIY 重铸 + 禁言（三将补入，2026-10-14）

## 任务来源

[T12 玩点论杀武将面](../../migration/upstream-extension-reconstruction.md)勘察续批：
存量 6 将（四整将：费诗/韩龙/刘焉/留赞；刘孚 2/3、田豫 1/2）——本批补入剩余
**三将**（西真夕红/造之仁君/杨仪），玩点论杀武将面全量入册。

## 本批承载

| 技能 | 上游 | 原生承载 |
|---|---|---|
| 屯田·DIY diyjuntun（造之仁君，出牌主动） | 重铸一张装备牌 | `WudianDiyjuntun`（tag 255）：activate（装备存在门）+ SelectCards 自选装备（手/场，装备栏走 revoke）→ 弃之 + **牌堆随机装备入手**（实体发现模式 v187 先例；登记差异：上游 canRecast 换置面不做；deck+discard 无装备则仅弃置） |
| 禁言 jinyan（西真夕红，锁定查询） | 体力 ≤2 时黑色锦囊禁用（四面 mod） | tag 256 目录锚 + **PlayCard 门查询**（往事封禁先例同位：hp≤2 ∧ 锦囊 ∧ 黑 → CardLimit 拒；**登记差异：上游 cardEnabled/Usable/Respondable/Savable 四面，原生仅使用面**） |

## 暂缓登记（+6，全量 8）

- 复仇 fuchou/fuchou2（shaBefore 弃牌标记 + phaseAfter 跨回合存储链式强制杀——跨回合存储面）
- 良策 liangce（viewAs 基本牌→五谷 + wuguRemained 子事件——子事件面）
- 坚壁 jianbi（多目标锦囊目标转移——伊籍先例可循，排后续批）
- 仇渡 choudu（视为用【调兵遣将】——依赖 deferred 卡，同调兵遣将暂缓先例）
- 理断 liduan（他人回合外得装备→令其使用——使用他人牌面）

## 过程要点

- **npack 资产哈希重算**：cards.json 改动需同步 npack 两处——`asset_manifest
  .entries[].source_hash`（blake3 文件字节）与 `manifest.asset_hash`
  （compute_asset_hash 同构，schema/package.rs:1391；translations/asset_paths/
  character 面按序喂入）——Python blake3 重算（流程登记）。
- **HostBehavior tag 值域已满**（u8，256 变体全占用，96 为唯一历史空洞——
  禁言回填 96）：**下一个新变体必须先将 tag/from_tag 扩为 u16**（from_tag
  的 `_` 兜底臂已因穷尽移除；snap 消费侧 u16::from 兼容）——登记边界。
- jinyan 查询键勘正：与 cards.json 声明一致（"jinyan"）。

## 实测记录

| 项 | 结果 |
|---|---|
| 反例实证 | 屯田 activate 桩化（返 None）+禁言查询禁用（恒假）→ 批 10 测试 **FAILED** |
| 定向 | g1_wudian **118/118**（117 既有 + 1 新） |
| engine-native 全量 | **738/0/0**（golden 不变——新逻辑仅扩展包技能持有者可达）——[日志](engine-native-tests.log) |
| 装载/包管理/服务端 | **84/0** |
| Gate 1b | exit 0（拦死 let/冗余 mut，已修）——[日志](gate1b-all-targets.log) |

## 登记边界与下一步

- 玩点论杀武将面：**9/9 名入册、14/22 条目转正、8 暂缓**（六整将：费诗/韩龙/
  刘焉/留赞/造之仁君/杨仪；刘孚 2/3、田豫 1/2、西真夕红 1/3）。
- 批 11 候选：坚壁（伊籍转移先例）、理断（令他人用牌面）。
