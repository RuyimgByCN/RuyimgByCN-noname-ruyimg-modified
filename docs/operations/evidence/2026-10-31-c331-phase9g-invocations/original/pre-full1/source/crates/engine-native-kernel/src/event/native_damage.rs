//! Finite identities of actual native damage source units in the locked catalog.
use super::*;
use crate::catalog::{CardKind, QueryCtx, TriggerMeta};
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeDamageEffect {
    Jueqing,
    Wuxin,
    Jidong,
    Chengxin,
    Chilongya,
    Xingjunyan,
    Liangou,
    Feiren,
    Ciqiu,
    Duwen,
    MailunOut3,
    MailunIn3,
    MailunOut4,
    MailunIn4,
    Hutian,
    Zuijiu,
    Huoshen,
    NitaiOut,
    Zhaohuo,
    ZhaohuoSource,
    NitaiIn,
    Bufo,
    LingquCancel,
    Shuchui,
    Paotai,
    Xixue,
    Qinglong,
    Yinzei,
    Zhangwu,
    Zlfanshi,
    Zhue,
    Abi,
    Yuance,
    Lingqu,
    Jiding,
    Renxing,
    Cuidui,
    Duqu,
    Huanxia,
    Jidian,
    Minwan,
    JidingRecover,
}
impl NativeDamageEffect {
    pub fn window(self) -> DamageWindowKind {
        match self {
            Self::Jueqing | Self::Wuxin | Self::Jidong | Self::Chengxin => DamageWindowKind::Before,
            Self::Chilongya
            | Self::Xingjunyan
            | Self::Liangou
            | Self::Feiren
            | Self::Duwen
            | Self::MailunOut3
            | Self::MailunIn3
            | Self::MailunOut4
            | Self::MailunIn4
            | Self::Hutian => DamageWindowKind::Begin,
            Self::Ciqiu | Self::Zuijiu | Self::Huoshen | Self::NitaiOut | Self::ZhaohuoSource => {
                DamageWindowKind::Begin1
            }
            Self::Zhaohuo | Self::NitaiIn | Self::Bufo | Self::LingquCancel => {
                DamageWindowKind::Begin4
            }
            Self::Shuchui
            | Self::Paotai
            | Self::Xixue
            | Self::Qinglong
            | Self::Yinzei
            | Self::Zhangwu
            | Self::Zlfanshi
            | Self::Zhue
            | Self::Abi
            | Self::Yuance
            | Self::Lingqu
            | Self::Jiding
            | Self::JidingRecover
            | Self::Renxing
            | Self::Cuidui
            | Self::Duqu => DamageWindowKind::End,
            Self::Huanxia | Self::Jidian | Self::Minwan => DamageWindowKind::After,
        }
    }
    pub fn helper_name(self) -> Option<&'static str> {
        match self {
            Self::JidingRecover=>Some("native:liuli/boss/jiding_recover"),
            Self::Jidong => Some("native:liuli/ow/jidong2"),
            Self::Chengxin => Some("native:liuli/xianjian/chengxin2"),
            Self::Chilongya => Some("native:liuli/card-swd/chilongya"),
            Self::Xingjunyan => Some("native:liuli/card-swd/xingjunyan"),
            Self::Liangou => Some("native:liuli/ow/liangou2"),
            Self::MailunOut3 => Some("native:liuli/swd/mailun31"),
            Self::MailunIn3 => Some("native:liuli/swd/mailun32"),
            Self::MailunOut4 => Some("native:liuli/swd/mailun41"),
            Self::MailunIn4 => Some("native:liuli/swd/mailun42"),
            Self::Hutian => Some("native:liuli/swd/hutian2"),
            Self::Shuchui => Some("native:liuli/card-swd/shuchui2"),
            Self::Paotai => Some("native:liuli/ow/paotai3"),
            Self::Xixue => Some("native:liuli/card-swd/xixue"),
            Self::Qinglong => Some("native:liuli/card-swd/qinglonglingzhu"),
            Self::Huanxia => Some("native:liuli/swd/huanxia_expire"),
            _ => None,
        }
    }
    pub fn meta(self) -> TriggerMeta {
        TriggerMeta {
            event: self.window().event_kind(),
            timing: Timing::Normal,
            priority: match self {
                Self::Wuxin => 10,
                Self::Hutian => -55,
                Self::LingquCancel => -11,
                _ => 0,
            },
            order_selectable: !matches!(
                self,
                Self::Liangou | Self::Shuchui | Self::Huanxia | Self::Paotai
            ),
            optional: false,
            extra_events: vec![],
        }
    }
    pub fn from_host(host: crate::vocabulary::host::HostBehavior) -> Option<Self> {
        use crate::vocabulary::host::HostBehavior as H;
        match host {
            H::WudianFeiren => Some(Self::Feiren),
            H::WudianCiqiu => Some(Self::Ciqiu),
            H::WudianDuwen => Some(Self::Duwen),
            _ => None,
        }
    }
    pub fn matches(
        self,
        owner: PlayerId,
        d: &DamageData,
        ctx: &QueryCtx<'_>,
        parent: Option<&Frame>,
        card_kind: Option<CardKind>,
    ) -> bool {
        let core = ctx.state;
        let catalog = ctx.catalog;
        let p = core.player(owner);
        let source = d.source == Some(owner);
        let target = d.target == owner;
        let fire = d.nature == DamageNature::Fire;
        let sha = d.card.is_some_and(|card| {
            let kind = card_kind
                .or_else(|| {
                    parent.and_then(|p| match &p.data {
                        EventData::CardUsed(u) if u.card == card => u.use_as,
                        _ => None,
                    })
                })
                .unwrap_or_else(|| catalog.card_kind(card));
            kind.is_sha_like()
        });
        let equip = |kind| p.equips.iter().any(|c| catalog.card_kind(*c) == kind);
        if !p.alive {
            return false;
        }
        match self {
            Self::JidingRecover=>source,
            Self::Jueqing => source,
            Self::Wuxin => target,
            Self::Zuijiu => {
                source
                    && sha
                    && parent.is_some_and(|p| {
                        p.stage == Stage::Intrinsic
                            && matches!(&p.data, EventData::CardUsed(u)
                    if u.use_as.unwrap_or_else(||catalog.card_kind(u.card)).is_sha_like())
                    })
            }
            Self::Jidong => target && p.jidong_active,
            Self::Chengxin => target && p.chengxin_active,
            Self::Chilongya => source && fire && !d.from_chain && equip(CardKind::Chilongya),
            Self::Xingjunyan => (source || target) && sha && equip(CardKind::Xingjunyan),
            Self::Liangou => {
                target && core.turn.liangou_hook == Some(owner) && !core.turn.liangou_bonus_used
            }
            Self::Feiren => {
                source
                    && sha
                    && !d.from_chain
                    && d.card.is_some_and(|c| {
                        catalog.cards[c as usize].suit == crate::catalog::Suit::Spade
                    })
            }
            Self::Ciqiu => {
                source && sha && core.player(d.target).hp == core.player(d.target).max_hp as i32
            }
            Self::Duwen => {
                source && !d.from_chain && p.hand.len() == core.player(d.target).hand.len()
            }
            Self::MailunOut3 => source && p.mailun_mode == 3 && !p.mailun_out_used,
            Self::MailunOut4 => source && p.mailun_mode == 4 && !p.mailun_out_used,
            Self::MailunIn3 => target && p.mailun_mode == 3 && !p.mailun_in_used,
            Self::MailunIn4 => target && p.mailun_mode == 4 && !p.mailun_in_used,
            Self::Hutian => {
                target
                    && !p.hutian_hold.is_empty()
                    && i64::from(p.hp) - i64::from(d.amount) < p.hutian_hold.len() as i64
            }
            Self::Huoshen => target && fire,
            Self::NitaiOut => target && fire && core.turn.current != owner,
            Self::NitaiIn => target && core.turn.current == owner,
            Self::Zhaohuo => target && (fire || source),
            Self::ZhaohuoSource => source,
            Self::Bufo | Self::LingquCancel => target && d.amount > 1,
            Self::Shuchui => {
                source
                    && core.turn.current == owner
                    && core.turn.shuchui_armed
                    && !core.turn.shuchui_hit
                    && sha
            }
            Self::Paotai => target && p.paotai_lv > 0,
            Self::Xixue => {
                source && sha && p.hp < p.max_hp as i32 && equip(CardKind::Xixueguizhihuan)
            }
            Self::Qinglong => {
                source
                    && !target
                    && d.nature != DamageNature::Normal
                    && core.player(d.target).alive
                    && equip(CardKind::Qinglonglingzhu)
                    && (!core.player(d.target).hand.is_empty()
                        || !core.player(d.target).equips.is_empty())
            }
            Self::Yinzei => {
                target
                    && p.hand.is_empty()
                    && p.hp > 0
                    && d.source.is_some_and(|s| {
                        s != owner
                            && core.player(s).alive
                            && (!core.player(s).hand.is_empty()
                                || !core.player(s).equips.is_empty())
                    })
            }
            Self::Zhangwu => target && d.source.is_some_and(|s| core.player(s).alive),
            Self::Zlfanshi | Self::Lingqu => target,
            Self::Zhue => d.source.is_some_and(|s| s != owner && core.player(s).alive),
            Self::Abi => target && d.source.is_some_and(|s| core.player(s).alive),
            Self::Yuance => {
                d.source
                    .is_some_and(|s| s != d.target && core.player(s).alive)
                    && core.player(d.target).alive

            }
            Self::Jiding => {
                !target
                    && core.is_boss_side(owner) == core.is_boss_side(d.target)
                    && d.source.is_some_and(|s| {
                        s != owner
                            && core.player(s).alive
                            && core.is_boss_side(s) != core.is_boss_side(owner)
                    })
            }
            Self::Renxing => core.turn.current != owner,
            Self::Cuidui => {
                source
                    && core.source_is_enemy_of(catalog, owner, d.target) == Some(true)
                    && core.player(d.target).alive
                    && catalog
                        .skill_by_name(crate::vocabulary::names::BOSS_ZHONGDU)
                        .is_some_and(|def| !core.has_skill_any_state(d.target, def))
            }
            Self::Duqu => {
                target
                    && d.source.is_some_and(|source| {
                        core.player(source).alive
                            && !core.skills.iter().any(|((holder, def), inst)| {
                                *holder == source
                                    && inst.enabled
                                    && catalog.skills[*def as usize]
                                        .behavior
                                        .native_damage_effect(EventKind::DamageEnd)
                                        == Some(Self::Duqu)
                            })
                    })
            }
            Self::Huanxia => {
                source
                    && d.card.is_some_and(|card| {
                        core.turn
                            .huanxia_track
                            .last()
                            .is_some_and(|(c, _)| *c == card)
                    })
            }
            Self::Jidian => source,
            Self::Minwan => source && core.turn.current == owner,
        }
    }
}
