# T12 运筹武将面批 4：蛊惑·变体 + 狡黠 + 红莲（2026-10-14）

## 任务来源

[T12 运筹武将面](../../migration/upstream-extension-reconstruction.md)批 3 续批。
同批完成**全部十名武将的技能归属勘察**（character.js 映射表）：

| 武将 | 上游技能 | 状态 |
|---|---|---|
| diy_caocao | xicai/diyjianxiong/hujia | 批 1（1/3，2 暂缓） |
| diy_weiyan | diykuanggu | 批 1 整将 |
| diy_sp_zhouyu | xiongzi/yaliang | 批 2 整将 |
| diy_luxun | shaoying/zonghuo | 批 3 整将 |
| **diy_yuji** | diyguhuo/diychanyuan | **本批（1/2，蝉缘暂缓）** |
| **diy_zhouyu** | jieyan/honglian | **本批（1/2，隙焰暂缓）** |
| **diy_zhenji** | diy_jiaoxia/yiesheng | **本批（1/2，野升暂缓）** |
| diy_caiwenji | beige/guihan | 待批（guihan 濒死限定技需主动技面） |
| diy_lukang | luweiyan(viewAs)/qianxun | 待批（viewAs 转化+内核名引用） |
| old_majun | xinfu_jingxie/xinfu_qiaosi | 待批（骰子/多步给牌面） |

## 本批承载

| 技能 | 上游 | 原生承载 |
|---|---|---|
| 蛊惑·变体 diyguhuo（于吉，强制） | 回合开始摸 2 然后弃自己区域 2 张 | tag 247：Judge-begin 触发 + 摸 2 弃 2 确定性序（判定区→装备[revoke]→手牌首张；登记差异：上游 AI 择弃） |
| 狡黠 diy_jiaoxia（甄姬，frequent） | 成为红色牌目标摸 1 | tag 248：CardUsed+targets 含持有者 ∧ 卡红 suit（frequent=自动无确认门） |
| 红莲 honglian（周瑜·DIY，可选） | 受伤后来源弃其全部红牌 | tag 249：Damage 触发 + 来源有红牌 filter + 确认门 + 手/装备红牌全弃（装备走 revoke_equip_skills 同款） |

**暂缓新增**：蝉缘 diychanyuan（死亡时来源 loseMaxHp——内核无 max_hp 减面/
MaxHpChanged 事件，登记待原语）、隙焰 jieyan（红牌 AOE 火伤主动技——multiline
多目标主动技面）、野升 yiesheng（弃任意张黑手牌摸等量——主动技 selectCard
1..∞ 面）。主动技面三技归同一机制缺口。

## 实测记录

| 项 | 结果 |
|---|---|
| 反例实证 | 三 filter 桩化恒 false → 批 4 测试 **FAILED**（0/1） |
| 定向 | g1_shahai_yunchou **17/17** |
| engine-native 全量 | **731/0/0**（golden 不变）——[日志](engine-native-tests.log) |
| 装载/包管理/服务端 | **84/0** |
| Gate 1b | exit 0——[日志](gate1b-all-targets.log) |

测试驱动修正两处：蛊惑断言点移至 Draw（Judge-begin 触发在 advance_to 停点
之后）；累计断言全面松弛化（character.json 累进式追加）。

## 登记边界与下一步

- 运筹武将面进度 **7/10 名、7/17 技**（三整将；暂缓 5：奸雄·变体/护驾/蝉缘/
  隙焰/野升）。
- 批 5 候选：主动技面三技（隙焰/野升+侍练）立项或 diy_caiwenji（悲歌判定面/
  归汉濒死限定技）勘察。
