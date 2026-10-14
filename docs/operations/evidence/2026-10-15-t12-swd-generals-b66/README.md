# T12 swd 武将面批㉖：浊山铸/莫耶/夜鸦熙

- 日期：2026-10-15
- 范围：三将转正（一整将+两 partial）——**30/79 名**（上批 27 将+3）；新增 HostBehavior tag 330-332
- 上游回读：`archive/qita/sanguosha/noname/extension/杀海拾遗/character/swd/{character.js,skill.js,translate.js}`（wanjun/senluo/huopu 逐段核验；zhenjiu2/lmazui2 子技语义勘察后 defer 林月）

## 三将语义与接线

| 将 | 技 | 接线 |
|---|---|---|
| swd浊山铸 (wu hp4, 男) | 隳延 suiyan（**复用已实现 def**——荣霜同技同 mapping）+万钧 wanjun | SwdWanjun(tag 330)=marker：engine PlayCard viewAs 链新增臂——装备牌→南蛮入侵（`kind.is_equip() && wanjun holder`）；响应面天然收敛登记 |
| swd莫耶 (wu hp3, 女) | 森罗 senluo（热血/流离 deferred） | SwdSenluo(tag 331)=ActivateSkill usable:1——activate 门=自己手牌数为全场最少（并列可）；效果=其他有手牌角色各弃 2（确定性首两张，登记差异上游自选）+摸 1 |
| swd夜鸦熙 (shu hp3, 女) | 火扑 huopu（热血/神焰 deferred） | SwdHuopu(tag 332)=marker：engine PlayCard viewAs 链新增臂——红桃牌→流星火雨（单目标 trick，机器已有） |

## 结构性修复（非本批技能、本批暴露）

- **AOE 早退臂 viewAs 声明缺失**：`CardKind::SavageAttack|ArcheryAttack|Jingleishan|Chiyuxi` push 臂用 `card_use`（不带 use_as）早退——万钧转化后工厂按实体种（HorsePlus）路由丢空（帧进、机不生、零伤害）。修复=臂内派生 `declared = kind != 实体种 ? Some(kind) : None` 改 `card_use_as`。火扑经尾部 4507 臂自带派生 ✓ 未受影响。

## 登记差异

1. **万钧**：响应面（装备牌当杀打出）天然收敛（native viewAs 链仅覆盖使用向）。
2. **森罗**：弃牌确定性首两张（上游目标自选 chooseToDiscard forced——道法同款登记）。
3. deferred：莫耶热血（回合外出杀面）/流离（subskill 树）；夜鸦熙神焰（limited+storage 延迟伤害面）。

## 验证

- **定向回归** `swd_generals_b66_wanjun_senluo_huopu`：万钧装备牌→南蛮全体无杀各受 1+消耗+手动目标拒反例；火扑红桃→流星火雨 he<2 直接火伤+黑桃杀正常杀结算被闪反例（红桃实体精确布阵）；森罗最少门并列可→p1 弃 2 摸 1+非最少拒反例。
- **全量**：engine-native 778 passed / 0 failed（无哈希面改动——golden 零扰动）。
- **Gate 1b**：exit 0（filler unused/redundant clone 拦截修复后复跑绿）。

## 测试布阵勘误（入记忆）

- **标准桃全红双花色**：桃实体含 Heart+Diamond 两种——火扑门（Heart）须按实体花色精确布阵（debug_place 按 kind 会拿到方块桃）。
- **森罗最少门**：isMinHandcard=并列最少即可——其余玩家须各持 ≥ 自己手牌数的牌（空手玩家会破坏最少性）。
- **viewAs 早退臂**：kind 转化后走早退 push 臂（AOE 系）必须携带 use_as 声明——工厂按帧载荷种路由机器。

## 结论

- **30/79 名**（upstream 79 将 183 技）；整将累计二十四（+1：浊山铸）。
- 四项用户决策维持待用户裁决清单登记（T15③ 排期/十殿阎罗族级 D/龙血玄黄立项/O5-O4 产品）。
