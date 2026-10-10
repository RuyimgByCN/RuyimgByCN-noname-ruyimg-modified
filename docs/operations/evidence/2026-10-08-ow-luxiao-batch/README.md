# U05 ow 纵切批九：卢西奥三技（规则 v126，2026-10-08）

ow 纵切批九：**卢西奥**（ow_luxiao，wu 3 男）三技全接线（无 deferred）
——出牌阶段主动弃牌强化（扩音/呼唤）+ 出牌阶段结束多目标摸/回（乐动）
的回合级存储三角。上游 25 将已闭环第 8 将。前置：
[ow 纵切批八](../2026-10-08-ow-mode-effects/README.md)。

## 上游语义核验（源码为准，…/character/ow/skill.js
yuedong/kuoyin/huhuan/kuoyin2）

- `kuoyin2`（kuoyin group）：silent phaseBegin 重置 storage
  （recover=false/num=0/eff=0）——原生 TurnState 换回合清零等价（存量
  仅自己出牌阶段可积攒/消费，跨回合无观察面）。
- `kuoyin`：phaseUse 主动，position he（手牌+装备）；selectCard 按存
  量：eff 已置→恰 1 张、num 已置→恰 2 张、否则 [1,2]；content：1 张→
  yuedong_num+=2、2 张→yuedong_eff++（各自限一次，双满 filter false）。
- `huhuan`：phaseUse 主动，恰 2 张；content：yuedong_recover=true
  （filter !recover 限一次）。
- `yuedong`：player phaseUseEnd direct（无确认询问）——chooseTarget
  [1, 1+num]（recover 模式 filterTarget hp<maxHp），摸 1+eff 各
  （asyncDraw）/recover 模式按座位序各回复 1+eff；可整体放弃。

## 改动（分开陈述）

- **代码已实现**：`TurnState.yuedong_num/yuedong_eff/yuedong_recover`
  （回合级；canonical_hash 三字段；存档 v6→**v7**）+
  `WudianYuedong`（tag 119，PhaseEnd 触发机：Act 建选项→TargetChoice
  应答臂按当时存量摸/回；recover 选项收窄未满体力者、空选项直跳过）+
  `WudianKuoyin`（tag 120，activate 门控（双满/he 空→拒绝）+
  SelectCards min/max 按存量 1–2/1/2→弃牌+存量更新）+
  `WudianHuhuan`（tag 121，恰 2 张→recover=true）；snap.active_ask_host
  三派生臂；a0 守卫表 121。
- **转换包**：cards.json 增 卢西奥（三技 mapping 通道）+ npack 三哈希
  重算（新增 `content-schema` example `rehash_cards_asset` 重算 Data 资
  产 entry source_hash=blake3(文件)，再 `rehash_npack` 重算
  logic_hash/asset_hash）。
- **登记差异**：乐动摸牌逐目标顺序 draw（上游 game.asyncDraw 同时摸，
  实体结果等价）；上游 storage 跨回合残留不可观察（kuoyin2 每回合重
  置）→ 原生换回合清零等价。

## 验收口径

- **回归已加入**：g1_wudian 4 例——
  `luxiao_kuoyin_num_extends_yuedong_draw`（弃 1→数+2；3 目标各摸 1）、
  `luxiao_kuoyin_eff_and_double_full_gate`（弃 2→量+1；第二次恰 1 张；
  双满 activate 拒绝 InvalidChoice；1 目标摸 2）、
  `luxiao_huhuan_switches_yuedong_to_recover`（回复模式；目标选项仅未
  满体力者；呼唤限一次拒绝；回复不摸牌）、
  `luxiao_yuedong_cancel_and_pending_ask_restores`（空选择=取消；挂起
  乐动询问跨存档恢复（v7）后应答生效、存量随存档保留）。
- **全量通过**：engine-native **442 passed / 0 failed / 1 ignored**
  （24 测试二进制逐项实测；v125 基线 438 +4 新增）；依赖 **63/0**
  （content-runtime 33 + engine-ai 29 + content-native-loader 1）。
- **golden**：版本 125→126 + TurnState 三零值字段入 canonical_hash
  ——14 场景全 mismatch（卢西奥不在标准场景，纯版本/零值字段位移），
  重录后 replay_scenarios 2/2。
- **测试坑（登记）**：SelectPlayers/SelectCards 的取消=空 selection
  （`ChoiceSpec::accepts` 只认 options 成员，`Pass` 非选项 → 
  InvalidChoice）；存档恢复断言按增量（中途他玩家固定摸牌使基数变化）。

## 剩余

- ow 其余 17 武将（美 bingqiang/jidong/baoxue、猎空/死神/法老之鹰/
  源氏/D.Va/安娜/麦克雷/狂鼠/半藏/莱因哈特/路霸/温斯顿/查莉娅/黑影/
  奥丽莎）逐武将纵切；xianjian 36 武将；U07 duoqi/youzhan 暂缓；
  zhenfa 五种阵法。

## 改动文件

见 [source-hashes.txt](source-hashes.txt)（17 文件，相对 noname-next 根）。
