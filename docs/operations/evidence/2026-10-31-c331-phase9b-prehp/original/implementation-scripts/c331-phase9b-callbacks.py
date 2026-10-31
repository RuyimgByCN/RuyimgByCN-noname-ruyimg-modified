from pathlib import Path
r=Path(Path('/tmp/c331-phase7-root').read_text().strip());w=r/'worktree';p=w/'crates/engine-native-kernel/src/kernel/damage.rs';s=p.read_text()
def cut(start,end):
 global s
 a=s.index(start);b=s.index(end,a);out=s[a:b];s=s[:a]+s[b:];return out
huanxia=cut('    // 还侠跟踪','    // 鼠槌');shuchui=cut('    // 鼠槌','    let mut hp_loss');jidian=cut('    // 极点','    // 反噬');minwan=cut('    // 冥顽锁定集','\n}\n\n/// 禳星')
gates=cut('    // 延妒门控','    // 催毒')+cut('    // 邪酬门控','\n}\n\n/// 禳星')
a=s.index('    // 隐贼');b=s.index('    // 章武',a);yin=s[a:b];activ_a=yin.index('            let p = ctx.core.player(holder);');activ_b=yin.index('            // 伤害后弃牌');activation=yin[activ_a:activ_b];s=s[:a]+'''    // 隐贼 damageEnd discard; its changeHp activation already completed before TargetDamage.
    if let Some(ydef)=ctx.catalog.skill_by_name(crate::vocabulary::names::BOSS_YINZEI) {
        let holder=d.target;
        if ctx.core.has_skill(holder,ydef) && ctx.core.player(holder).hand.is_empty() && ctx.core.player(holder).hp>0 {
            if let Some(src)=d.source {
                if src!=holder && ctx.core.player(src).alive {
                    if let Some(c)=ctx.core.player(src).hand.first().copied(){ctx.discard_from_hand(src,c);}
                }
            }
        }
    }
'''+s[b:]
# End now contains only damageEnd callbacks; After and changeHp have their own transitions.
s=s.replace('''pub(super) fn post_damage(ctx: &mut Ctx<'_>, d: crate::event::DamageData) {
''','''pub(super) fn post_damage(ctx: &mut Ctx<'_>, d: crate::event::DamageData) {
'''+shuchui,1)
s=s.replace('''    hlss_light_refill(ctx, d.target);''','''    hlss_light_refill(ctx, d.target);
    damage_change_hp::commit_damage_change_hp(ctx,&d);''',1)
# Shanguang/matching gates run on HP changes even when notrigger suppresses damage windows.
new='''//! HP-change gates complete before the ordinary TargetDamage window.
use super::*;
pub(super) fn commit_damage_change_hp(ctx:&mut Ctx<'_>, d:&crate::event::DamageData) {
    if let Some(ydef)=ctx.catalog.skill_by_name(crate::vocabulary::names::BOSS_YINZEI) {
        let holder=d.target;
        if ctx.core.has_skill_any_state(holder,ydef) {
'''+activation+'''        }
    }
'''+gates+'''    shangshix_draw(ctx,d.target);
}
''';(p.parent/'damage_change_hp.rs').write_text(new)
after='''//! Source-qualified damageAfter callbacks, after the entire Dying/Source/End subtrees.
use super::*;
pub(super) fn post_damage_after(ctx:&mut Ctx<'_>,d:crate::event::DamageData) {
'''+huanxia+jidian+minwan+'\n}\n';(p.parent/'damage_after.rs').write_text(after);p.write_text(s)
p=w/'crates/engine-native-kernel/src/kernel.rs';s=p.read_text().replace('mod damage_windows;','mod damage_windows;\nmod damage_change_hp;\nmod damage_after;',1);s=s.replace('''{post_damage(ctx,d);}''','''{post_damage(ctx,d.clone());}
            if window==crate::event::DamageWindowKind::After && d.amount>0 && !d.prevented && !d.settlement.notrigger && d.settlement.outcome==crate::event::DamageOutcome::Running {damage_after::post_damage_after(ctx,d);}''',1);p.write_text(s)
p=w/'crates/engine-native-kernel/src/resolution/trigger_order.rs';s=p.read_text().replace('''            if event.kind() != frame.data.kind()''','''            let data_kind=if matches!(frame.data,EventData::DamageWindow{..}) {EventKind::Damage}else{frame.data.kind()};
            if event.kind() != data_kind''',1);p.write_text(s)
p=w/'crates/engine-native-content/src/content/standard/boss_skills.rs';s=p.read_text()
for skill,event in [('BOSS_XIANGDE','DamageBegin3'),('BOSS_ZHAOHUO','DamageBegin1')]:
 a=s.index('            name: '+skill+',');b=s.index('            limits:',a);part=s[a:b].replace('EventKind::DamageBefore','EventKind::'+event);s=s[:a]+part+s[b:]
p.write_text(s)
# Preserve semantic fixtures rather than reverting the corrected target-before-Dying ordering.
p=w/'crates/engine-native/tests/g1_wudian/yuanshi.rs';s=p.read_text().replace('assert_eq!(e.debug_hp(2), 3, "传导伤害不加成（4→3）");','assert_eq!(e.debug_hp(2), 2, "传导携带主事件 num2；notLink 防止再次施加飞刃，不能降回基础 num1");');p.write_text(s)
p=w/'crates/engine-native/tests/g1_shahai_swd/xielv.rs';s=p.read_text().replace('''    // 反例 ②b：取消 → 1 伤
''','''    // 反例 ②b 是独立的 linked-only 条件；前例 fire/changeHp 已解除横置。
    e.state_core_mut().player_mut(1).linked=true;
    // 反例 ②b：取消 → 1 伤
''',1);p.write_text(s)
p=w/'crates/engine-native/tests/native_kernel/combat.rs';s=p.read_text().replace('''    // 火杀先在 1 处结算 → 移伤确认 → 转移给 3
''','''    // changeHp 以最终承受者为准；3 必须真的横置才有传播资格。
    e.state_core_mut().player_mut(3).linked=true;
    // 火杀先在 1 处结算 → 移伤确认 → 转移给 3
''',1);p.write_text(s)
p=w/'crates/engine-native/tests/native_kernel/boss_chujiangwang.rs';s=p.read_text().replace('''    for _ in 0..3 {
        match e.open_choice() {''','''    for _ in 0..8 {
        match e.open_choice() {''',1).replace('''            Some(c) if c.spec.prompt == "dying_save" => {''','''            Some(c) if matches!(c.spec.prompt,"dying_save"|"trigger_confirm") => {''',1);p.write_text(s)
p=w/'crates/engine-native/tests/native_kernel/dying.rs';s=p.read_text().replace('''    // 无人持桃：全部跳过询问，直接死亡
    assert!(!e.debug_alive(0));''','''    // 散财的真实 targetDamage admission 先于 Dying，拒绝后无人持桃才死亡。
    if prompt_of(&e)=="trigger_confirm" {answer(&mut e,ChoiceOption::Pass);}
    assert!(!e.debug_alive(0));''',1);p.write_text(s)
p=w/'crates/engine-native/tests/native_kernel/boss_xuanwu.rs';s=p.read_text().replace('''    assert_eq!(e.state_core().player(1).lingqu_count, 1, "0 点伤害仍计数");''','''    assert_eq!(e.state_core().player(1).lingqu_count, 0, "Zero 终止普通 damageEnd，不计数");''').replace('''assert_eq!(e.debug_hand(1).len(), h1 + 1, "0 点伤害仍摸 1");''','''assert_eq!(e.debug_hand(1).len(), h1, "Zero 不派普通 damageEnd 摸牌");''').replace('''assert_eq!(e.state_core().player(1).lingqu_count, 2);''','''assert_eq!(e.state_core().player(1).lingqu_count, 1);''').replace('''assert_eq!(e.debug_hand(1).len(), h1 + 2);''','''assert_eq!(e.debug_hand(1).len(), h1 + 1);''').replace('''assert_eq!(e.state_core().player(1).lingqu_count, 3);''','''assert_eq!(e.state_core().player(1).lingqu_count, 1);''').replace('''assert_eq!(lim, 6, "手牌上限 = hp 3 + 灵渠计数 3");''','''assert_eq!(lim, 4, "手牌上限 = hp3 + 一次正数 damageEnd");''');p.write_text(s)
