# T12 swd 武将面批 81：多鹏/疾鹏（两整将）

- 日期：2026-10-15
- 范围：两整将转正——**69/79 名**（上批 67 将+2）；新增 HostBehavior tag 368–370
- 上游回读：`archive/upstream/noname/apps/core/extension/杀海拾遗/character/swd/{character.js,skill.js,translate.js}`（luanji/guozao/reyingzi 逐段核验）

## 两将语义与接线

| 将 | 技 | 接线 |
|---|---|---|
| swd多鹏 (wu hp3, 男) | 乱击 luanji + 英姿 reyingzi | SwdLuanji(tag 368)=ActivateSkill usable:1——SelectCards 弃两张同花色手牌（消费段花色一致性校验，异花色拒）→ use_virtual_card 万箭齐发（AOE 全体其他存活，座位序）；SwdReyingzi(tag 370)=PhaseBegin(Draw) 额外摸 1 + `state.hand_limit` 基数换体力上限 |
| swd疾鹏 (wu hp3, 男) | 英姿 reyingzi + 聒噪 guozao | SwdGuozao(tag 369)=Damage forced 锁定——距离 1 以内角色（来源非自己）受损后：TargetChoice 距离 2 内有手牌角色（含自己）→ 其手牌全部入弃堆、牌堆顶三张入手（上游 lose._triggered=null + gain "draw" 同口径） |

## 设计要点

- **英姿·刷新面独立承载**：标准目录「英姿」只有摸牌面（ADR 0750 口径），refresh reyingzi 另有 mod maxHandcardBase=maxHp 面——以 `host:reyingzi` 独立行为承载两面（摸牌机 + hand_limit 基数查询），`builtin:英姿` 不适用于该引用。
- **hand_limit 基数替换**：既有修饰（星体改造/无脏/兽九/灵渠/神恩）全部保留叠加，只替换基数 hp→maxHp（上游 mod maxHandcardBase 覆盖语义）。
- **乱击双卡校验在消费段**：SelectCards 选项集=全部手牌（min=max=2），花色一致性在 Answer 消费时 `upstream_same_suit` 校验（不合即 Done 不发牌）——上游 complexCard 逐张校验同口径；activate 前置检查存在同花色对（无非对时拒发动）。
- **聒噪目标含受损者自己**（上游 filterTarget 无 target≠player 排除，只要求 h>0 且距离 ≤2）；「来源为自己」被 filter 排除（上游 `event.source === player → false`）。
- 登记差异：聒噪观看面收敛为单窗（上游为 dialog 二段交互，无玩家决策差异）。

## 验证

- **定向回归** `swd_generals_b81_luanji_guozao_reyingzi`：
  - 乱击正例：同花色 Sha+Tao → 万箭 p1/p2/p3 各受 1；反例：异花色两卡 → ActivateSkill 拒。
  - 聒噪正例：p3 杀 p2（距离 1，来源非持技者）→ 目标窗 observer=1、选 p2 → 手牌替换为 3 张。
  - 英姿：hand_limit_for_debug=4（hp2/maxHp4 基数替换）；p3 摸牌阶段清手后摸 3（2+1）。
- **包装载**：69 将入册（cards.json 67→69）。
- **全量**：engine-native **793 passed / 0 failed / 0 ignored**（golden 零扰动——14 场重放逐字节一致）。
- **Gate 1b**：`RUSTFLAGS="-D warnings" cargo check -p engine-native --all-targets` exit 0。

## 结论

- **69/79 名**（upstream 79 将；两整将 4/4 技 + 英姿复用于双将）。
- 四项用户决策维持待用户裁决清单登记（T15③ 排期/十殿阎罗族级 D/龙血玄黄立项/O5-O4 产品）。
