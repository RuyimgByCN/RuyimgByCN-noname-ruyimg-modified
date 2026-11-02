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
        matches!(event, EventData::Damage(d) if self.native_damage_effect(kind).is_some_and(|effect|effect.matches(owner,d,ctx,None)))
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
    {
        let effect = E::Jidong;
        skills.push(SkillDef {
            name: effect.helper_name().unwrap(),
            trigger: Some(effect.meta()),
            limits: Default::default(),
            behavior: Box::new(NativeDamageBehavior {
                name: effect.helper_name().unwrap(),
                effects: &[E::Jidong],
            }),
        });
    }
    {
        let effect = E::Chengxin;
        skills.push(SkillDef {
            name: effect.helper_name().unwrap(),
            trigger: Some(effect.meta()),
            limits: Default::default(),
            behavior: Box::new(NativeDamageBehavior {
                name: effect.helper_name().unwrap(),
                effects: &[E::Chengxin],
            }),
        });
    }
    {
        let effect = E::Chilongya;
        skills.push(SkillDef {
            name: effect.helper_name().unwrap(),
            trigger: Some(effect.meta()),
            limits: Default::default(),
            behavior: Box::new(NativeDamageBehavior {
                name: effect.helper_name().unwrap(),
                effects: &[E::Chilongya],
            }),
        });
    }
    {
        let effect = E::Xingjunyan;
        skills.push(SkillDef {
            name: effect.helper_name().unwrap(),
            trigger: Some(effect.meta()),
            limits: Default::default(),
            behavior: Box::new(NativeDamageBehavior {
                name: effect.helper_name().unwrap(),
                effects: &[E::Xingjunyan],
            }),
        });
    }
    {
        let effect = E::Liangou;
        skills.push(SkillDef {
            name: effect.helper_name().unwrap(),
            trigger: Some(effect.meta()),
            limits: Default::default(),
            behavior: Box::new(NativeDamageBehavior {
                name: effect.helper_name().unwrap(),
                effects: &[E::Liangou],
            }),
        });
    }
    {
        let effect = E::MailunOut3;
        skills.push(SkillDef {
            name: effect.helper_name().unwrap(),
            trigger: Some(effect.meta()),
            limits: Default::default(),
            behavior: Box::new(NativeDamageBehavior {
                name: effect.helper_name().unwrap(),
                effects: &[E::MailunOut3],
            }),
        });
    }
    {
        let effect = E::MailunIn3;
        skills.push(SkillDef {
            name: effect.helper_name().unwrap(),
            trigger: Some(effect.meta()),
            limits: Default::default(),
            behavior: Box::new(NativeDamageBehavior {
                name: effect.helper_name().unwrap(),
                effects: &[E::MailunIn3],
            }),
        });
    }
    {
        let effect = E::MailunOut4;
        skills.push(SkillDef {
            name: effect.helper_name().unwrap(),
            trigger: Some(effect.meta()),
            limits: Default::default(),
            behavior: Box::new(NativeDamageBehavior {
                name: effect.helper_name().unwrap(),
                effects: &[E::MailunOut4],
            }),
        });
    }
    {
        let effect = E::MailunIn4;
        skills.push(SkillDef {
            name: effect.helper_name().unwrap(),
            trigger: Some(effect.meta()),
            limits: Default::default(),
            behavior: Box::new(NativeDamageBehavior {
                name: effect.helper_name().unwrap(),
                effects: &[E::MailunIn4],
            }),
        });
    }
    {
        let effect = E::Hutian;
        skills.push(SkillDef {
            name: effect.helper_name().unwrap(),
            trigger: Some(effect.meta()),
            limits: Default::default(),
            behavior: Box::new(NativeDamageBehavior {
                name: effect.helper_name().unwrap(),
                effects: &[E::Hutian],
            }),
        });
    }
    {
        let effect = E::Shuchui;
        skills.push(SkillDef {
            name: effect.helper_name().unwrap(),
            trigger: Some(effect.meta()),
            limits: Default::default(),
            behavior: Box::new(NativeDamageBehavior {
                name: effect.helper_name().unwrap(),
                effects: &[E::Shuchui],
            }),
        });
    }
    {
        let effect = E::Paotai;
        skills.push(SkillDef {
            name: effect.helper_name().unwrap(),
            trigger: Some(effect.meta()),
            limits: Default::default(),
            behavior: Box::new(NativeDamageBehavior {
                name: effect.helper_name().unwrap(),
                effects: &[E::Paotai],
            }),
        });
    }
    {
        let effect = E::Xixue;
        skills.push(SkillDef {
            name: effect.helper_name().unwrap(),
            trigger: Some(effect.meta()),
            limits: Default::default(),
            behavior: Box::new(NativeDamageBehavior {
                name: effect.helper_name().unwrap(),
                effects: &[E::Xixue],
            }),
        });
    }
    {
        let effect = E::Qinglong;
        skills.push(SkillDef {
            name: effect.helper_name().unwrap(),
            trigger: Some(effect.meta()),
            limits: Default::default(),
            behavior: Box::new(NativeDamageBehavior {
                name: effect.helper_name().unwrap(),
                effects: &[E::Qinglong],
            }),
        });
    }
    {
        let effect = E::Huanxia;
        skills.push(SkillDef {
            name: effect.helper_name().unwrap(),
            trigger: Some(effect.meta()),
            limits: Default::default(),
            behavior: Box::new(NativeDamageBehavior {
                name: effect.helper_name().unwrap(),
                effects: &[E::Huanxia],
            }),
        });
    }
}
