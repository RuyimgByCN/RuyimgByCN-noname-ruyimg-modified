# 联动包确定性移植

| 武将 | 初始体力 | 技能 | 状态 |
|---|---:|---|---|
| `ol_chouniu`（丑牛） | 1/5 | `olchouniu`（丑牛） | 完整单技能武将；最低体力判断与结束阶段回复有 Golden |
| `ol_sishe`（巳蛇） | 3/3 | `olsishe`（巳蛇） | 完整单技能武将；可选等量反伤与伤害来源校验有 Golden |
| `ol_shenhou`（申猴） | 3/3 | `olshenhou`（申猴） | 完整单技能武将；红色判定排除当前【杀】目标与多目标隔离有 Golden |
| `ol_maotu`（卯兔） | 3/3 | `olmaotu`（卯兔） | 完整单技能武将；脱离濒死后的体力比较目标保护及下回合清理有 Golden |
| `ol_chenlong`（辰龙） | 4/4 | `olchenlong`（辰龙） | 完整单技能武将；数值选择、自损濒死减上限与濒死后伤害续队列有 Golden |
| `ol_weiyang`（未羊） | 3/3 | `olweiyang`（未羊） | 完整单技能武将；不同类别弃牌、等量受伤目标与逐目标回复有 Golden |
| `ol_yinhu`（寅虎） | 4/4 | `olyinhu`（寅虎） | 完整单技能武将；出牌阶段类别去重、伤害濒死与回合内禁用有 Golden |
| `ol_wuma`（午马） | 4/4 | `olwuma`（午马） | 完整单技能武将；他人锦囊目标摸牌、禁止翻至背面及四阶段跳过恢复有 Golden |
| `ol_haizhu`（亥猪） | 5/5 | `olhaizhu`（亥猪） | 完整单技能武将；他人弃置黑牌精确取得及准备阶段最多手牌失去体力有 Golden |
| `ol_zishu`（子鼠） | 3/3 | `olzishu`（子鼠） | 完整单技能武将；强制首次取手牌、可选重复及并列最多终止有 Golden |
| `nizhuanzhishen`（逆转之神） | 4/5 | `dcfanzhuan`、`dcniyun` | 完整双技能武将；每轮体力反转与摸牌有 Golden |
| `dc_caocao`（经典曹操） | 4/4 | `dcjianxiong`（奸雄） | 完整单技能武将；取得伤害牌、递增摸牌与五张上限有 Golden |
| `sp_mushun`（SP穆顺） | 4/4 | `moukui`（谋溃，依赖 `sp` 包） | 完整单技能武将；摸牌/弃牌/取消与精确【杀】-目标反弃有 Golden |

- `olshenhou`：成为其他角色使用【杀】的目标时可判定，结果为红色则仅令此【杀】对拥有者无效。
- `olmaotu`：任意角色脱离濒死后，直到拥有者下个回合开始，体力值不低于拥有者的其他角色不能
  对其使用牌；自身用牌和体力值更低的其他角色不受限制。
- `olchenlong`：出牌阶段限一次，选择另一名角色和 1/2 点数值；拥有者先失去等量体力，若这次
  自损首次进入濒死则减 1 点体力上限，完整结算濒死后再对目标造成等量伤害。
- `olweiyang`：出牌阶段限一次，弃置一至三张类别互不相同的手牌或装备牌，并选择等量已受伤
  角色，各回复一点体力；目标可以包含技能拥有者。
- `olyinhu`：出牌阶段内每种基本、锦囊、装备类别限一次，弃置一张手牌或装备牌并对一名其他
  角色造成一点普通伤害；若这次伤害令角色进入濒死，本回合禁用此技能。额外出牌阶段会重置
  类别记录，但不会提前解除同回合的技能禁用。
- `olwuma`：成为其他角色使用的锦囊牌目标后摸一张牌；正面状态不能翻至背面，但已有背面
  状态仍可正常翻回正面；判定、摸牌、出牌、弃牌阶段被跳过或取消时保留跳过审计事件并恢复
  执行该阶段。
- `olhaizhu`：其他角色因弃置进入弃牌堆的黑色实体牌由拥有者取得；同批红牌、自身弃牌和
  非弃置移动均不包含在内。准备阶段若拥有者手牌数并列全场最多，其失去一点体力。
- `olzishu`：出牌阶段限一次，先从手牌数更多的一名其他角色处取得一张手牌；每次取得完成后
  重新比较手牌数，可以继续选择手牌数更多的角色，直到拥有者并列全场最多或主动停止。
- `dcfanzhuan`：每轮限一次，任意回合开始时可将体力值调整为已损失体力值，并摸变化值张牌。
- `dcniyun`：攻击范围和【杀】使用次数均增加拥有者当前已损失体力值。
- `clbshuiyong`：锁定防止拥有者受到的火属性伤害。
- `clbjisu`（济溯）：判定阶段开始时可选跳过判定阶段和摸牌阶段，视为对攻击范围内一名其他角色使用一张【杀】。（ABI 450）
- `spduyi2`：临时效果存在时，不能使用或打出手牌；非手牌响应不受影响。
- `spchizhong`：手牌上限基数固定为体力上限，其他角色死亡后增加 1 点体力上限。
- `yiran`：锁定令拥有者受到的火焰伤害增加一点；普通、雷电与失去体力不受影响。
- `olyouji`：摸牌阶段额外摸当前轮数张牌，轮数贡献至多为五。
- `dcxurui`：拥有者的回合结束且本回合未造成伤害时，摸攻击范围张牌，至多摸五张。
- `olxugou`：红色【杀】对拥有者无效；拥有者使用红色【杀】无距离限制且伤害 +1。
- `mbjunkui`：游戏开始时将所有进攻坐骑和防御坐骑移出游戏；拥有者使用【杀】的次数上限 +1。
- `mbjiguan`：游戏开始时将所有进攻坐骑和防御坐骑移出游戏；拥有者的手牌上限 +2。
- `bachiqionggouyu_skill`：出牌阶段结束时，若拥有者已受伤则回复 1 点体力；满体力时摸牌阶段额外摸两张牌。
- `clbshuijian`：摸牌阶段可选择额外摸“装备数的一半（向下取整）+1”张牌。
- `dcqiusuo`：造成或受到伤害后，可从牌堆、再从弃牌堆顶获得第一张【铁索连环】。
- `fuzhao`：一名角色进入濒死时可令其判定；有效花色为红桃则由技能拥有者令其回复一点体力。
- `dcjianxiong`：受到伤害后可获得仍在处理区的伤害牌，摸“奸雄”标记数加一张牌，并将后续摸牌数递增至最多五张。
- `dclonggong`：每回合限一次，受到有存活来源的伤害时可防止之，并令来源从牌堆获得一张装备牌。

重新生成包：

`dcnutao`（怒涛，桃神）：使用目标数大于一的锦囊牌指定目标后，对其中一名其他角色造成 1 点雷属性伤害（目标选择由拥有者执行，近似旧 JS 随机）。

```bash
cargo run -q -p content-cli -- pack \
  content/original/ported/collab/bachiqionggouyu.skill \
  content/original/ported/collab/clbjisu.skill \
  content/original/ported/collab/clbshuijian.skill \
  content/original/ported/collab/clbshuiyong.skill \
  content/original/ported/collab/dcfanzhuan.skill \
  content/original/ported/collab/dcjianxiong.skill \
  content/original/ported/collab/dclonggong.skill \
  content/original/ported/collab/dcniyun.skill \
  content/original/ported/collab/dcnutao.skill \
  content/original/ported/collab/dcqiusuo.skill \
  content/original/ported/collab/dczhiheng.skill \
  content/original/ported/collab/spfengyin.skill \
  content/original/ported/collab/dcxurui.skill \
  content/original/ported/collab/fuzhao.skill \
  content/original/ported/collab/mbjiguan.skill \
  content/original/ported/collab/mbjunkui.skill \
  content/original/ported/collab/olchenlong.skill \
  content/original/ported/collab/olchouniu.skill \
  content/original/ported/collab/olhaizhu.skill \
  content/original/ported/collab/olmaotu.skill \
  content/original/ported/collab/olshenhou.skill \
  content/original/ported/collab/olsishe.skill \
  content/original/ported/collab/olweiyang.skill \
  content/original/ported/collab/olwuma.skill \
  content/original/ported/collab/olxugou.skill \
  content/original/ported/collab/olyinhu.skill \
  content/original/ported/collab/olyouji.skill \
  content/original/ported/collab/olzishu.skill \
  content/original/ported/collab/spchizhong.skill \
  content/original/ported/collab/spduyi2.skill \
  content/original/ported/collab/yiran.skill \
  --characters content/original/ported/collab/characters.json \
  --metadata content/original/ported/collab/metadata.json \
  --out content/original/ported/collab/collab.npack.json \
  --package-id collab
```



完整武将 `yeshiwen`（叶诗文）已发布（ADR 0304）：`clbshuiyong` 以 `damage_taken_cancel(fire)`
重写旧 JS 的火属性伤害防止；`clbjisu` 以 ABI 450 的 `skip_judge_and_draw_phases` +
`choose_use_virtual_sha` 重写旧 JS 的判定阶段取消与虚拟【杀】；旧 JS 不进入运行时。

完整武将 `tw_dm_zhouyu`（TW魔周瑜）已发布：四技能全部由既有已移植技能跨包解析
（`twjuyan`/`twlihuo` 来自 TW 包，`yingjian` 来自 MOBILE 包，`yiran` 本包），
本包仅补齐选将池条目（ADR 0326）；旧 JS 不进入运行时。

完整武将 `dc_sunquan`（经典孙权）已发布（ADR 0338）：`dczhiheng`（制衡，ABI 467
`resolve_dczhiheng` 弃任意张手牌/装备牌→摸等量牌，弃尽手牌额外摸一张）单技能闭包；
collab 特化的伤害加发动次数机制文档化延后；旧 JS 不进入运行时。

完整武将 `sp_fuwan`（SP伏完）已发布（ADR 0342）：`spchizhong`（持重，既有）+
`spfengyin`（封印，ABI 471 `resolve_spfengyin` + `skip_next_discard_phase`：他人准备
阶段且体力不低于你时，可交给其一张【杀】令其跳过出牌阶段与弃牌阶段）组合闭包；
旧 JS 不进入运行时。
