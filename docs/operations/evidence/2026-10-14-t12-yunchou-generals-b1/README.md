# T12 运筹武将面批 1：戏才 + 狂骨·变体（2026-10-14）

## 任务来源

[T12 整包 0/37](../../../architecture/native-progress.md) 首来源推进切片。来源勘察：
**运筹帷幄=卡牌子包（23 张已转）+ 武将子包（10 名武将 17 技，此前从未转换）**——
武将面缺失即运筹来源无法整包的核心缺口（卡牌切片 23/23 deferred 0 与整包完成是
两回事）。本批起按将推进武将面。

## 上游语义（杀海拾遗/character/yunchou/skill.js）

| 技能 | 上游 | 原生承载 |
|---|---|---|
| 戏才 xicai（diy_caocao） | 受伤且伤害由牌造成（可选触发）→ 获得伤害牌 | `HostBehavior::YunchouXicai`（tag 241）：Damage 帧 target=持有者 ∧ d.card 有值；获得 table 优先、弃堆兜底（登记近似：不校验上游 position 'o' 处理区位） |
| 狂骨·变体 diykuanggu（diy_weiyan） | 你造成伤害后（强制）：目标到你攻击距离 >1 摸 1，否则已受伤回复 1 | `HostBehavior::YunchouDiykuanggu`（tag 242）：filter 含满血距离门；效果内按 `seat_distance` 分支（登记差异：攻击距离按座位基础距离，坐骑修正不参与） |

**暂缓登记**（diy_caocao 另两技，mapping null）：奸雄·变体 diyjianxiong
（mode 限定 identity/guozhan 的死亡前置身份改写——原生身份局原型无运行时改身份
承载）、护驾 hujia（内置已有同名词技能——mapping 机制无「内置引用」面，武将
目录暂列 1 技）。

## 交付

- **engine-native/content/host.rs**：`HostBehavior` +2 变体（tag 241/242、ALL
  242、from_tag/key/trigger_meta[xicai optional 确认门]/filter/step/(target,card)
  提取）——第 9 触点（提取臂）由回归抓漏（见下）。
- **content-native-loader**：`HostBehavior => "host:key"` 反向表 +2。
- **转换包** `shahai-yunchou-upstream-noname/assets/character.json`：批 1 两名
  武将（ycn曹操/ycn魏延）。
- **测试** g1_shahai_yunchou +1：桥接（2 将/2 行为/unmapped=2）、目录元数据
  （护驾奸雄不入目录）、戏才真实出杀通道（可选确认门→处理区回收）、狂骨距离
  分支（距离 2 摸 1/距离 1 受伤回 1）。

## 回归抓漏（第 9 触点）

行为接线遗漏 `start()` 的 (target, card) 提取臂——filter/step 均在而
`self.card=None` 静默跳过效果（**编译绿≠接线通**，新行为必定向回归的既证）。
补臂后转绿。反例实证：两 filter 臂桩化恒 false → 批 1 测试 **FAILED**
（0 passed/1 failed）。

## 实测记录

| 项 | 结果 |
|---|---|
| 定向 | g1_shahai_yunchou **14/14**（13 既有 + 1 新） |
| engine-native 全量 | **728/0/0**（无哈希面变化，golden 不变）——[日志](engine-native-tests.log) |
| 装载/包管理/服务端 | **84/0** |
| Gate 1b | exit 0（拦截探针残留变量，已修）——[日志](gate1b-all-targets.log) |

## 登记边界与下一步

- 运筹武将面进度 **2/10 名、2/17 技**（diy_caocao 1/3 技+2 暂缓、diy_weiyan
  1/1 技完成）。
- 批 2 候选：diy_sp_zhouyu（xiongzi+yaliang）、diy_luxun（shaoying+zonghuo）。
- 内置技能引用面（mapping 引用标准技）为机制缺口，登记待立项——解锁后护驾
  类可入目录。
