# MOBILE 确定性移植

- `xinjiaojin`：受到男性角色造成的伤害前，可弃置一张有效装备牌防止整段伤害；作为跨包规范定义供 TW 孙鲁班复用。
- `re_liubiao`：复用 YIJIANG「自守」与 REFRESH「界宗室」闭合，不复制跨包技能实现。
- `rejiuchi`：黑桃手牌可按既有视为牌协议作为【酒】；酒加成的【杀】实际造成伤害后，原技能
  实例「崩坏」失效至回合结束。与 SHENHUA「肉林、崩坏、暴虐」闭合手杀界董卓。
- `zhaohuo`：其他角色进入濒死时，若体力上限大于一，将体力上限减至一并摸原体力上限减一张牌。
- `rejiangchi`：出牌阶段开始时可选择摸一张并在本阶段禁用【杀】，或弃一张手牌/装备牌并获得 YIJIANG 已重写的 `jiangchi2` 强化。
- `rejiangchi3`：摸牌分支的临时效果；本出牌阶段不能主动使用【杀】，阶段结束时移除。
- `reanxu`：出牌阶段限一次，第二名目标从第一名目标的手牌区或装备区获得一张牌；若来自手牌区，拥有者摸一张，随后可令手牌较少者再摸一张。
- `xin_caozhang`：由 `rejiangchi` 完整闭合进入武将索引；分支原语与 STANDARD「将驰」共享 ABI 50，但触发时点和禁用范围遵循手杀旧 JavaScript。
- `refuli`：自己濒死时可回复至存活势力数；回复结算后仅在当前体力全场唯一最高时翻面，并消耗限定技。
- `redangxian`：回合开始时仅从弃牌堆获得一张【杀】，再执行一个先于准备阶段的独立额外出牌阶段；与已重写的 `refuli` 一起闭合 `xin_liaohua`。
- `huaizi`：手牌上限以体力上限为基数。
- `juliao`：其他角色计算与拥有者的距离时，增加存活角色势力数减一。
- `shixin`：锁定防止拥有者受到的火属性伤害。
- `yingjian`：准备阶段可选择合法目标，按完整用牌流程使用一张无距离限制的无实体【杀】。
- `rw_zhuge_skill`：重铸诸葛连弩装备技能生效时，【杀】的使用次数上限为无限。
- `cz_liulongcanjia_skill`：统计装备区或判定区至少有一张 K 的存活角色数，动态等量缩短拥有者至他人的距离并增加他人至拥有者的距离。
- `mbranshang`：按火焰伤害点数积累“燃”，结束阶段按标记失去体力，并在标记大于一时减 2 点体力上限、摸两张牌。
- `mbhanyong`：受伤且使用牌指定多个目标时，可选择摸两张牌、令当前牌基础伤害 +1，或背水执行前两项并获得一个“燃”；与 `mbranshang` 闭合手杀兀突骨。
- `tiansuan2_damage`：上签授予的子技能；受到伤害后可摸等同伤害点数的牌。
- `tiansuan2_0`：完整重写上上签；防止拥有者受到的所有非失去体力伤害。
- `tiansuan2_1`：完整重写上签；在藤甲等前段修正后把最终伤害值封顶为 1，并保留旧分组子技能的可选等量摸牌。
- `tiansuan2_2`：完整重写中签；先把受到的伤害改为火属性，再在藤甲等加成后把最终伤害封顶为 1。
- `tiansuan2_fire`：中签旧分组使用的独立改火属性子技能。
- `tiansuan2_3`：完整重写下签；受到的非失去体力伤害在普通修正阶段加一。
- `tiansuan2_4`：完整重写下下签；受到的伤害加一，且不能使用或以【桃】/【酒】自救。
- `tiansuan2_ai`：旧客户端伤害评估提示，无权威规则行为；保留技能定义但不生成规则索引。
- `xinfu_pdgyingshi2`：〖隐世〗分组效果；跳过拥有者的判定阶段。
- `dujin`：摸牌阶段可选择额外摸“装备数的一半（向上取整）+1”张牌。
- `fenyin`：自己的回合内，使用牌的颜色与上一张不同时，可以摸一张牌。
- `mbyilie3`：〖义烈〗的结束阶段子技能；有“烈”时摸一张牌，失去等量体力并移去所有“烈”。
- `scsniqu`：出牌阶段限一次，对一名角色造成 1 点火焰伤害；使用通用主动技与属性伤害契约。郭胜是旧包中 `isUnseen`、0 体力的十常侍组件，不加入普通可选武将索引。
- `xinjintao`：使用【杀】无距离限制且次数上限 +1；出牌阶段第一张【杀】不可被响应，第二张【杀】伤害 +1。`xin_wuban` 与该技能一起闭环进武将索引。
- `friendgongli`：旧客户端阵营查询辅助函数；新运行时直接使用权威关系查询，因此只保留技能定义，不生成规则索引。
- `liechi`：伤害令拥有者进入濒死时，存活的伤害来源强制弃置一张手牌或装备牌。
- `rw_minguangkai_link`：耀光铠的横置子技能；防具有效时，未横置的拥有者不能被横置，已经横置时仍可正常解除横置。
- `rw_tengjia1`：防具有效时，【南蛮入侵】与【万箭齐发】不能指定拥有者。
- `rw_tengjia2`：防具有效时，拥有者受到的火焰伤害 +1。
- `rw_tengjia3`：防具有效时，未横置的拥有者不能被横置；已经横置时仍可正常解除横置。
- `rw_tengjia4`：防具有效时，普通【杀】不能指定拥有者。
- `rw_renwang_skill`：防具有效时，黑色【杀】和红桃【杀】不能指定拥有者。
- `xiaoxi_hansui`：黑色手牌或装备牌可在出牌阶段当【杀】使用，也可在响应窗口当【杀】打出。
- `mobilejingce`：结束阶段统计本回合所有角色因使用或打出而进入弃牌堆的实体牌；数量不小于当前体力时可摸两张牌。`mobilejingce_count` 的旧客户端存储职责已并入权威回合计数，`xin_guohuai` 同批闭环进武将索引。
- `scszimou`：锁定技；当前出牌阶段主动使用第 2/4/6 张牌时，分别从牌堆或弃牌堆取得一张【酒】/【杀】/【决斗】。计数按每次出牌阶段独立重置，响应牌不计入。孙璋是旧包中 `isUnseen`、0 体力的十常侍组件，不加入普通可选武将索引。
- `scsxiaolu`：出牌阶段限一次，先摸两张并等待嵌套触发完整结束，再选择把两张当前手牌交给一名其他角色或弃置两张当前手牌；不足两张时弃置所有当前可弃手牌。韩悝是旧包中 `isUnseen`、0 体力的十常侍组件，只迁移技能，不加入普通可选武将索引。
- `scsyaozhuo`：出牌阶段限一次，与一名其他角色拼点；严格获胜令其跳过下一个摸牌阶段，失败或平局则弃置两张手牌或装备牌，不足时全部弃置。夏恽是旧包中 `isUnseen`、0 体力的十常侍组件，只迁移技能，不加入普通可选武将索引。
- `xinduodao`：受到伤害结算后，若伤害来源装备区仍有武器，可选择一次性获得其中所有武器；结算时重新核对来源装备区及装备栏位。
- `xinanjian`：使用【杀】指定攻击范围不包含自己的目标后，强制选择令该目标不能响应，或只令此【杀】对该目标的伤害 +1。单目标和多目标续执行均按目标保存分支；与 `xinduodao` 一起闭合 `xin_panzhangmazhong` 进入武将索引。
- `rejieming`：每受到一点伤害均可独立选择一名存活角色摸两张牌；等待该次摸牌及其嵌套触发结束后，若其最终手牌数仍小于体力上限，拥有者摸一张牌。SHENHUA `quhu` 完成后，`re_xunyu` 的跨包技能闭包已一并发布。
- `ly_piliche`：霹雳车装备技能；对其他角色造成伤害后，若其手牌或装备区有当前可由拥有者弃置的牌，可以弃置其装备区内所有当前可弃牌。确认后重新检查存活、区域归属与弃置保护；保留旧 JS 在“只有可弃手牌而无装备”时仍可确认但最终空操作的行为。
- `jujun`：任一具体阶段结束时，若拥有者在该阶段成为过用牌目标且实际体力发生过变化，锁定视为使用一张【万箭齐发】；阶段历史为定长玩家位图，不保存旧 JS 事件对象。
- `bixian`：任一具体阶段结束时，若拥有者在该阶段成为过用牌目标且实际体力未变化，锁定视为使用一张【决斗】并强制选择合法目标；与 `jujun` 共用 ABI 98 的阶段历史和无实体用牌原语。
- `o_chenlan`、`o_meicheng`：分别由 `jujun`、`bixian` 闭合进入 MOBILE 武将索引。
- `xin_gongsunzan`：武将闭包由本包登记，技能复用 `official/refresh@0.1.0` 中已重写的 `xinyicong` 与 `qiaomeng`；安装器会按精确依赖验证并加载 REFRESH。
- `re_sp_zhugeliang`：武将闭包由本包登记，复用 SHENHUA 的 `bazhen` 与 REFRESH 的 `rehuoji`、`rekanpo`；现有 MOBILE → REFRESH → SHENHUA 精确依赖链即可闭合。
- `re_xunyu`：武将闭包由本包登记，复用 SHENHUA 的 `quhu` 与本包 `rejieming`；沿 MOBILE → REFRESH → SHENHUA 精确依赖链加载。
- `xinpingkou`：结束阶段可按本回合跳过阶段数选择其他角色依次造成 1 点伤害；全部伤害结算后只从共享牌堆获得最靠上的装备牌，不回退查询弃牌堆。与 YIJIANG 已重写的 `fenli` 闭合 `xin_zhuhuan`。
- `mbkuili`：受到伤害后，若存活的伤害来源有被废除的武器栏，锁定令其恢复一层武器栏；恢复事件保留技能拥有者作为来源。
- `re_sunjian`：复用 SHENHUA 已重写的「英魂」与 REFRESH「破虏」形成完整武将闭包；
  MOBILE 现有精确依赖已覆盖两包，不复制跨包技能实现。
- `re_dianwei`：复用 REFRESH 已重写的「强袭」形成完整武将闭包；同一目标按每次出牌阶段
  独立限一次，不复制跨包技能实现。
- `re_caiwenji`：复用 REFRESH「界悲歌」与 SHENHUA「断肠」形成完整武将闭包；保持群势力、
  三点体力和 `[rebeige, duanchang]` 技能顺序，MOBILE 不复制跨包技能实现。
- `re_bulianshi`：复用本包「界安恤」与 YIJIANG「追忆」形成完整武将闭包；保持吴势力、
  三点体力和 `[reanxu, zhuiyi]` 技能顺序，MOBILE 不复制跨包技能实现。
- `xinqingxi`：每回合限一次，造成伤害前令另一名目标弃置
  `max(1, 4-来源至目标距离)` 张手牌；不足或取消则本次伤害与连环基数加一。与
  YIJIANG「千驹」闭合 `xin_caoxiu`，复用既有倾袭弃牌/增伤选择链。
- `xinqieting`：其他角色回合结束且未对其他角色造成伤害时，可取消、摸一张、私下查看其
  至多两张手牌并获得其中一张，或移动其一张可装备牌。复用既有窃听装备移动续接，与
  YIJIANG「献州」闭合 `xin_caifuren`。
- `beizhu`：出牌阶段限一次，私下观看一名有手牌的其他角色。无【杀】时弃置其一张牌并
  可令其获得牌堆中的一张【杀】；有【杀】时随机依次令其无距离、无次数限制地对自己使用
  快照中的有效【杀】，每点直接伤害摸一张牌。闭合 `dingyuan`。
- `chengzhao`：任一角色进入结束阶段时，若本回合获得至少两张牌且双方有手牌，可与一名
  角色拼点；严格胜利后对其使用无距离、无视防具的虚拟【杀】，失败或平局无后续。闭合
  `dongcheng`。
- `mbshanjia`：出牌阶段限一次摸三张，再弃置“3-本局从装备区失去牌数”张牌。未弃基本牌
  可使用一张不计次数的虚拟【杀】，未弃锦囊牌则本出牌阶段使用牌无距离限制；两项奖励独立
  判断。装备离区复用现有逐牌触发累计，不保存旧 JS 历史对象；闭合 `mb_caochun`。
- `daoji`：出牌阶段限一次弃置一张非基本牌，选择一名有可获得装备的其他角色，获得其一件
  装备并在当前合法时强制使用；实体武器无论是否成功装备，均在该次尝试及其嵌套触发后对
  原角色造成一点伤害。闭合 `hucheer`。
- `jiaohua`：出牌阶段限两次，依次选择尚未记录的基本、锦囊或装备类别及任一存活角色；
  只从摸牌堆获得最靠上的对应类别牌。即使牌堆无牌也记录类别，三类齐全后清空并开始新一轮。
  记录跨阶段和回合保留；闭合 `liwei`。
- `laishou`：受到不小于当前体力的伤害时，若体力上限小于九，先增加等同伤害值的体力上限
  再防止该伤害；准备阶段体力上限达到九后无来源死亡。
- `luanqun`：出牌阶段限一次，所有有手牌的存活角色依座次完成私密选牌后统一亮牌；身份模式
  可获得至多四张同色牌，其他模式至多两张。异色角色下个出牌阶段的首张【杀】只能指定乱群
  来源，且其本回合对这些来源使用的【杀】不可响应；两个隐藏助手合并保存多个乱群来源，并在
  该角色回合结束时兜底清理。与 `laishou` 闭合 `laimin`。
- `refuhai`：出牌阶段限一次，其他角色从下家起依座次私密选择潮起/潮落，全部完成后统一
  公开；从下家起连续相同结果超过一名时，摸等同连续人数的牌。闭合 `re_weiwenzhugezhi`。
- `mbanda`：每轮限一次，仅响应由存活伤害来源造成的濒死；来源可交给濒死者恰好两张有效
  颜色不同的手牌或装备牌，取消、无牌或没有合法异色牌对时改为回复 1 点体力。
- `mbzhuguo`：出牌阶段限一次，将一名角色手牌调整至体力上限且至多五张；未摸牌则回复
  1 点体力，实际摸牌后若并列全场手牌最多，吴珂可指定另一角色，由被调整者选择实体或合法
  视为【杀】并至少包含该目标，本次用牌无距离和次数限制。与 `mbanda` 闭合 `wuke`。
- `mbanxian`：每回合第一次弃置手牌的事件若包含仍在弃牌堆的实体【杀】，可获得其中一张；
  获得及嵌套移动触发完成后，有合法目标时强制按统一流程使用，忽略通常距离与次数限制。
- `mbjinzu`：出牌阶段限一次，声明式选择一张手牌和一名其他角色；对方手牌超过两张时私密
  选择两张后统一亮牌。拥有者牌为最值时弃置全部展示牌并在嵌套移动触发后刷新技能；不为
  唯一最值时按目标累计到本回合下一张【杀】，只令实际对应目标不可响应并分别增加伤害。
  与 `mbanxian` 闭合 `mb_caoxing`。
- `hannan`：出牌阶段限一次，复用公共拼点流水线；仅严格胜者对另一方造成一点伤害，平局
  不造成伤害。
- `jiebing`：受到伤害后，强制选择伤害来源之外的一名其他角色，按权威随机数获得其一张
  当前可获得的手牌或装备牌；若为装备牌则按普通用牌流程立即使用。与既有 `hannan` 闭合
  `yangfu`。
- `mobiledanshou`：其他角色结束阶段按其本回合使用牌指定过拥有者的次数结算；零次自动
  摸一张，否则可弃置等量手牌或装备牌并对其造成一点伤害。闭合 `xin_zhuran`。
- `scskuiji`（试探）：出牌阶段限一次，选择一名有手牌的其他角色，从你与其的手牌中共选至多四张
  花色互异的牌并弃置。以新增 choice 续体原语 `choose_scskuiji_discard`（ABI 348）表达：续体在
  resolve 时校验花色互异并按归属分别弃置；旧 JavaScript 不进入运行时。

```bash
cargo run -q -p content-cli -- pack \
  content/original/ported/mobile/zhaohuo.skill \
  content/original/ported/mobile/rejiangchi.skill \
  content/original/ported/mobile/rejiangchi3.skill \
  content/original/ported/mobile/refuli.skill \
  content/original/ported/mobile/redangxian.skill \
  content/original/ported/mobile/huaizi.skill \
  content/original/ported/mobile/reqianxin3.skill \
  content/original/ported/mobile/juliao.skill \
  content/original/ported/mobile/shixin.skill \
  content/original/ported/mobile/yingjian.skill \
  content/original/ported/mobile/rw_zhuge_skill.skill \
  content/original/ported/mobile/cz_liulongcanjia_skill.skill \
  content/original/ported/mobile/mbranshang.skill \
  content/original/ported/mobile/mbhanyong.skill \
  content/original/ported/mobile/tiansuan2_damage.skill \
  content/original/ported/mobile/tiansuan2_0.skill \
  content/original/ported/mobile/tiansuan2_1.skill \
  content/original/ported/mobile/tiansuan2_2.skill \
  content/original/ported/mobile/tiansuan2_fire.skill \
  content/original/ported/mobile/tiansuan2_3.skill \
  content/original/ported/mobile/tiansuan2_4.skill \
  content/original/ported/mobile/tiansuan2_ai.skill \
  content/original/ported/mobile/xinfu_pdgyingshi2.skill \
  content/original/ported/mobile/dujin.skill \
  content/original/ported/mobile/fenyin.skill \
  content/original/ported/mobile/mbyilie3.skill \
  content/original/ported/mobile/scsniqu.skill \
  content/original/ported/mobile/xinjiaojin.skill \
  content/original/ported/mobile/xinjintao.skill \
  content/original/ported/mobile/friendgongli.skill \
  content/original/ported/mobile/liechi.skill \
  content/original/ported/mobile/rw_minguangkai_link.skill \
  content/original/ported/mobile/rw_tengjia1.skill \
  content/original/ported/mobile/rw_tengjia2.skill \
  content/original/ported/mobile/rw_tengjia3.skill \
  content/original/ported/mobile/rw_tengjia4.skill \
  content/original/ported/mobile/rw_renwang_skill.skill \
  content/original/ported/mobile/xiaoxi_hansui.skill \
  content/original/ported/mobile/mobilejingce.skill \
  content/original/ported/mobile/scszimou.skill \
  content/original/ported/mobile/scsxiaolu.skill \
  content/original/ported/mobile/scsyaozhuo.skill \
  content/original/ported/mobile/xinduodao.skill \
  content/original/ported/mobile/xinanjian.skill \
  content/original/ported/mobile/xinpingkou.skill \
  content/original/ported/mobile/rejieming.skill \
  content/original/ported/mobile/ly_piliche.skill \
  content/original/ported/mobile/jujun.skill \
  content/original/ported/mobile/bixian.skill \
  content/original/ported/mobile/mbkuili.skill \
  content/original/ported/mobile/rejiuchi.skill \
  content/original/ported/mobile/reanxu.skill \
  content/original/ported/mobile/xinqingxi.skill \
  content/original/ported/mobile/xinqieting.skill \
  content/original/ported/mobile/beizhu.skill \
  content/original/ported/mobile/chengzhao.skill \
  content/original/ported/mobile/mbshanjia.skill \
  content/original/ported/mobile/daoji.skill \
  content/original/ported/mobile/jiaohua.skill \
  content/original/ported/mobile/laishou.skill \
  content/original/ported/mobile/luanqun.skill \
  content/original/ported/mobile/luanqun_effect.skill \
  content/original/ported/mobile/luanqun_directHit.skill \
  content/original/ported/mobile/mbanda.skill \
  content/original/ported/mobile/mbzhuguo.skill \
  content/original/ported/mobile/mbanxian.skill \
  content/original/ported/mobile/mbjinzu.skill \
  content/original/ported/mobile/mbjinzu_effect.skill \
  content/original/ported/mobile/hannan.skill \
  --characters content/original/ported/mobile/characters.json \
  --metadata content/original/ported/mobile/metadata.json \
  --out content/original/ported/mobile/mobile.npack.json \
  --package-id mobile
```
