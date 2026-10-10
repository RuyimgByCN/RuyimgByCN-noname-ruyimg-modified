# 2026-09-23 第 34 轮：杀海拾遗·古剑子包切片（规则版本 45）

拆解转换杀海拾遗下一子包 **古剑**（`card/gujian`，40 张实体牌），
首切片 **5/40 张**，三种新宿主原语（`HostBehavior` tag 21–23）：

- **舒筋散**（`GujianShujinsanCard`，tag 21，锦囊×2）：目标选任意张
  手牌（至少 1，可取消）弃置并摸等量的牌。
- **面里金针**（`GujianMianliCard`，tag 22，机关×1）：目标摸一张牌
  然后受 1 点伤害（登记差异：上游为失去 1 点体力，原生无失去体力
  原语；上游无 wuxieable，不包无懈窗口）。
- **飞镖**（`GujianFeibiaoCard`，tag 23，机关×2）：目标弃一张黑色
  手牌（可取消，取消则受伤）或受 1 点伤害；无黑色手牌直接受伤
  （登记差异：失去体力简化为伤害；上游毒属性未建模）。

`STANDARD_RULESET_VERSION` **44 → 45**（`CardKind::Shujinsan`/
`Mianlijinzhen`/`Feibiao`，tag 37–39；14 golden 重录）。

| 验证 | 结果及边界 |
|---|---|
| [engine-native 全量测试](native-tests.log) | **318 passed / 0 failed**（第 33 轮 314 + 古剑切片 4 项）。全工作区 4,884 项通过。测试全绿不代表与上游规则等价。 |
| [探针（v45 重跑）](probes.log) | **22 项 FIXED / 0 项 OPEN** 维持。 |

## 原语细节

- `CardKind::Shujinsan`（37）/`Mianlijinzhen`（38）/`Feibiao`（39）；
  archive nums 至 39；`KNOWN_PROMPTS` + `gj_shujinsan_discard`/
  `gj_feibiao_discard`；def_key（`gj:shujinsan`/`gj:mianlijinzhen`/
  `gj:feibiao`）按牌种在场判定。
- 快照派生遵循**推进后 stage** 惯例：两台选牌机器在 `WaitChoice` 前
  已推进到 `Act`——挂起询问派生臂按 `Act` 键控（`SelectCard` 为瞬态，
  不派生）。
- 引擎校验臂：舒筋散 = 恰 1 名存活有手牌目标（可含自己；登记差异：
  上游含装备区，原生仅手牌）；面里金针 = 目标体力 ≥ 使用者；飞镖 =
  恰 1 名其他存活目标（无距离限制）。
- 取消语义：`allow_cancel` 选择的取消 = **空选择**（`accepts([])`），
  机器以 `answer().first()` 判空分支，不得用 `answer_first()`（空选择
  panic）。

## 回归（g1_shahai_gujian 新建 4 项）

- `g1_gujian_vertical_existing_kinds_and_lock`：发现→导入→桥接→建局
  →面里金针结算（摸 1+受伤、无窗口无选择）→低体力目标拒→异集合
  恢复拒绝。
- `shujinsan_discard_any_draws_equal_with_restore`：目标选 2 弃 2 摸 2、
  未选牌保留；**挂起点字节恢复**（`OpenChoice` 一致性过 N04-e 校验）；
  取消分支无弃无摸。
- `feibiao_discard_black_or_damage`：选项仅含黑色手牌；弃黑牌免伤；
  取消受伤；无黑牌直接受伤（无选择挂起）。
- `gujian_rejects_illegal_targets`：舒筋散无手牌目标拒；飞镖自-target
  /双目标拒。

## 边界（如实登记）

- 古剑切片 **5/40**（暂缓 35 张：食物 12[计时状态原语]、机关 8[展示
  比对/邻座扩展/获得装备/增益减益]、锦囊 8[事件取消/响应拒绝/生成
  虚拟牌/护甲]、基本 4[护甲/濒死标记]、装备 2[专属装备技]）。
- 面里金针/飞镖登记差异：失去体力简化为 1 点伤害（原生无失去体力
  原语——绕过闪避/伤害触发/濒死差异均按伤害路径）。
- 舒筋散登记差异：上游弃牌区含装备（he），原生仅手牌。
- 飞镖上游毒（poison）属性未建模。
- 炉石子包仍 7/17；gwent/mtg/swd/xianjian/ow/zhenfa/wuxing play 未
  转换。整包验收仍 **0/37**。
