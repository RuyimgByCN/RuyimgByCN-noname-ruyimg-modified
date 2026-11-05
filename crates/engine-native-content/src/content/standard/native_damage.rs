//! Native declarations enter the ordinary source-window competition.
use super::*;
use crate::event::NativeDamageEffect;
pub(super) struct NativeDamageBehavior {
    pub name: &'static str,
    pub effects: &'static [NativeDamageEffect],
}
impl SkillBehavior for NativeDamageBehavior {
    fn native_damage_effect(&self, kind: EventKind) -> Option<NativeDamageEffect> {
        self.effects
            .iter()
            .copied()
            .find(|e| e.window().event_kind() == kind)
    }
    fn filter(&self, _ctx: &QueryCtx<'_>, _owner: PlayerId, _event: &EventData) -> bool {
        false
    }
    fn filter_in_window(
        &self,
        ctx: &QueryCtx<'_>,
        owner: PlayerId,
        event: &EventData,
        kind: EventKind,
    ) -> bool {
        matches!(event, EventData::Damage(d) if self.native_damage_effect(kind).is_some_and(|effect|effect.matches(owner,d,ctx,None,None)))
    }
    fn start(&self, owner: PlayerId, event: &EventData) -> Box<dyn SkillMachine> {
        NoopBehavior.start(owner, event)
    }
    fn start_in_window(
        &self,
        ctx: &QueryCtx<'_>,
        owner: PlayerId,
        _event: &EventData,
        kind: EventKind,
    ) -> Box<dyn SkillMachine> {
        Box::new(crate::kernel::NativeDamageSkill {
            owner,
            def: ctx
                .catalog
                .skill_by_name(self.name)
                .expect("immutable declaration"),
            window: self
                .native_damage_effect(kind)
                .expect("declared window")
                .window(),
        })
    }
}
pub(super) fn configure(skills: &mut Vec<SkillDef>) {
    use NativeDamageEffect as E;
    for skill in skills.iter_mut() {
        let effects: &'static [E] = match skill.name {
            JUE_QING => &[E::Jueqing],
            BOSS_WUXIN => &[E::Wuxin],
            BOSS_ZUIJIU => &[E::Zuijiu],
            BOSS_HUOSHEN => &[E::Huoshen],
            BOSS_NITAI => &[E::NitaiOut, E::NitaiIn],
            BOSS_ZHAOHUO => &[E::ZhaohuoSource, E::Zhaohuo],
            BOSS_BUFO => &[E::Bufo],
            BOSS_LINGQU => &[E::Lingqu, E::LingquCancel],
            BOSS_YINZEI => &[E::Yinzei],
            BOSS_ZHANGWU => &[E::Zhangwu],
            BOSS_ZLFANSHI => &[E::Zlfanshi],
            BOSS_ZHUE => &[E::Zhue],
            BOSS_ABI => &[E::Abi],
            BOSS_YUANCE => &[E::Yuance],
            BOSS_JIDING => &[E::Jiding],
            BOSS_RENXING => &[E::Renxing],
            BOSS_CUIDUI => &[E::Cuidui],
            BOSS_DUQU => &[E::Duqu],
            BOSS_JIDIAN => &[E::Jidian],
            BOSS_MINWAN => &[E::Minwan],
            _ => continue,
        };
        let mut meta = effects[0].meta();
        meta.extra_events = effects[1..]
            .iter()
            .map(|e| e.window().event_kind())
            .collect();
        skill.trigger = Some(meta);
        skill.behavior = Box::new(NativeDamageBehavior {
            name: skill.name,
            effects,
        });
    }

    for effects in [
        &[E::Jidong][..],
        &[E::Chengxin],
        &[E::Chilongya],
        &[E::Xingjunyan],
        &[E::Liangou],
        &[E::MailunOut3],
        &[E::MailunIn3],
        &[E::MailunOut4],
        &[E::MailunIn4],
        &[E::Hutian],
        &[E::Shuchui],
        &[E::Paotai],
        &[E::Xixue],
        &[E::Qinglong],
        &[E::Huanxia],
    ] {
        let effect = effects[0];
        let name = effect.helper_name().unwrap();
        skills.push(SkillDef {
            name,
            trigger: Some(effect.meta()),
            limits: Default::default(),
            behavior: Box::new(NativeDamageBehavior { name, effects }),
            original_source: None,
            presentation_binding: None,
        });
    }
}

pub(super) fn append_invocation_helpers(skills: &mut Vec<SkillDef>) {
    use NativeDamageEffect as E;
    skills.push(SkillDef {
        name: E::JidingRecover.helper_name().unwrap(),
        trigger: Some(E::JidingRecover.meta()),
        limits: Default::default(),
        behavior: Box::new(NativeDamageBehavior {
            name: E::JidingRecover.helper_name().unwrap(),
            effects: &[E::JidingRecover],
        }),
        original_source: None,
        presentation_binding: None,
    });
    skills.push(SkillDef {
        name: "native:liuli/guozhan/jubao",
        trigger: None,
        limits: Default::default(),
        behavior: Box::new(GuozhanJubaoGain),
        original_source: None,
        presentation_binding: None,
    });
}

struct GuozhanJubaoGain;
impl SkillBehavior for GuozhanJubaoGain {
    fn filter(&self, _: &QueryCtx<'_>, _: PlayerId, _: &EventData) -> bool {
        false
    }
    fn start(&self, _: PlayerId, _: &EventData) -> Box<dyn crate::catalog::SkillMachine> {
        Box::new(crate::kernel::NoopMachine)
    }
    fn can_be_gained(
        &self,
        ctx: &QueryCtx<'_>,
        owner: PlayerId,
        card: crate::ids::CardId,
        source: PlayerId,
        target: PlayerId,
    ) -> Option<bool> {
        (owner == target
            && source != target
            && ctx.state.player(target).equips.contains(&card)
            && ctx.catalog.card_slot_of(ctx.state, card)
                == Some(crate::catalog::EquipSlot::Treasure))
        .then_some(false)
    }
}
