//! Authoritative `VmViewAsSpec` conversion table for Decision 34 (`U01`, `U05`, `U08`,
//! `L11`, `L20`, `L25`) skills and equipment flags.

use std::sync::OnceLock;

use crate::catalog::{CardKind, Catalog};
use crate::ids::{CardId, PlayerId};
use crate::rvm_bridge::{VmViewAsFilter, VmViewAsSpec, VmViewAsWindow, VmViewAsZone};
use crate::state::GameStateCore;

use super::{flag_val, has_ext_skill};

enum D34ViewAsActivation {
    Skill(&'static str),
    Flag(&'static str),
    SkillOrFlag(&'static str, &'static str),
    JiminNoEquip,
    FzRewushengShouyue,
}

struct D34ViewAsRule {
    activation: D34ViewAsActivation,
    spec: VmViewAsSpec,
}

fn make_spec(
    id: u32,
    name: &str,
    as_kind: CardKind,
    window: VmViewAsWindow,
    zone: VmViewAsZone,
    filter: VmViewAsFilter,
    ignore_distance: bool,
) -> VmViewAsSpec {
    VmViewAsSpec {
        skill_def_id: id,
        skill_name: name.to_string(),
        as_kind,
        window,
        zone,
        filter,
        uses_per_turn: None,
        ignore_distance,
        red_unrespondable: false,
        black_no_use_count: false,
        no_use_count: false,
        target_lacks_source_suit_unrespondable: false,
        refund_use_count_if_no_damage: false,
        min_cards: 1,
        max_cards: 1,
        owner_hp_min_one_cards: false,
        owner_basic_uses_plus_one_cards: false,
        once_per_turn_per_branch: false,
        same_suit: false,
        multi_card_bonus: None,
        branch: None,
        once_per_round: false,
        next_branch: None,
        cost: None,
        mark_cost: None,
        post_draw: 0,
    }
}

fn d34_view_as_table() -> &'static Vec<D34ViewAsRule> {
    static TABLE: OnceLock<Vec<D34ViewAsRule>> = OnceLock::new();
    TABLE.get_or_init(|| {
        let mut v = vec![
            D34ViewAsRule {
                activation: D34ViewAsActivation::SkillOrFlag("ssr_newlongdan", "l25_longdan_viewas"),
                spec: make_spec(
                    1_926_900_001,
                    "ssr_newlongdan",
                    CardKind::Sha,
                    VmViewAsWindow::Any,
                    VmViewAsZone::Hand,
                    VmViewAsFilter::Shan,
                    false,
                ),
            },
            D34ViewAsRule {
                activation: D34ViewAsActivation::SkillOrFlag("ssr_newlongdan", "l25_longdan_viewas"),
                spec: make_spec(
                    1_926_900_002,
                    "ssr_newlongdan",
                    CardKind::Shan,
                    VmViewAsWindow::Any,
                    VmViewAsZone::Hand,
                    VmViewAsFilter::Sha,
                    false,
                ),
            },
            D34ViewAsRule {
                activation: D34ViewAsActivation::SkillOrFlag("zylongdan", "l11_zylongdan_viewas"),
                spec: make_spec(
                    1_926_900_051,
                    "zylongdan",
                    CardKind::Sha,
                    VmViewAsWindow::Any,
                    VmViewAsZone::Hand,
                    VmViewAsFilter::Shan,
                    false,
                ),
            },
            D34ViewAsRule {
                activation: D34ViewAsActivation::SkillOrFlag("zylongdan", "l11_zylongdan_viewas"),
                spec: make_spec(
                    1_926_900_052,
                    "zylongdan",
                    CardKind::Shan,
                    VmViewAsWindow::Any,
                    VmViewAsZone::Hand,
                    VmViewAsFilter::Sha,
                    false,
                ),
            },
            D34ViewAsRule {
                activation: D34ViewAsActivation::SkillOrFlag("ssr_wusheng", "l25_wusheng_viewas"),
                spec: make_spec(
                    1_926_900_003,
                    "ssr_wusheng",
                    CardKind::Sha,
                    VmViewAsWindow::Any,
                    VmViewAsZone::HandEquip,
                    VmViewAsFilter::Red,
                    true,
                ),
            },
            D34ViewAsRule {
                activation: D34ViewAsActivation::SkillOrFlag(
                    "ssr_newguose",
                    "l25_guose_diamond_lebu",
                ),
                spec: make_spec(
                    1_926_900_004,
                    "ssr_newguose",
                    CardKind::Le,
                    VmViewAsWindow::PlayPhase,
                    VmViewAsZone::HandEquip,
                    VmViewAsFilter::Diamond,
                    false,
                ),
            },
            D34ViewAsRule {
                activation: D34ViewAsActivation::Skill("ssr_newjijie"),
                spec: make_spec(
                    1_926_900_005,
                    "ssr_newjijie",
                    CardKind::Shan,
                    VmViewAsWindow::Response,
                    VmViewAsZone::HandExpansion,
                    VmViewAsFilter::Any,
                    false,
                ),
            },
            D34ViewAsRule {
                activation: D34ViewAsActivation::SkillOrFlag("ssr_newjiang", "l25_jiang_active"),
                spec: make_spec(
                    1_926_900_006,
                    "ssr_newjiang",
                    CardKind::Sha,
                    VmViewAsWindow::Any,
                    VmViewAsZone::Hand,
                    VmViewAsFilter::Basic,
                    false,
                ),
            },
            D34ViewAsRule {
                activation: D34ViewAsActivation::Flag("dddxiaxing_viewas_taojiu"),
                spec: make_spec(
                    1_926_900_007,
                    "dddxiaxing",
                    CardKind::Tao,
                    VmViewAsWindow::Any,
                    VmViewAsZone::Hand,
                    VmViewAsFilter::Any,
                    false,
                ),
            },
            D34ViewAsRule {
                activation: D34ViewAsActivation::Flag("dddxiaxing_viewas_taojiu"),
                spec: make_spec(
                    1_926_900_008,
                    "dddxiaxing",
                    CardKind::Jiu,
                    VmViewAsWindow::Any,
                    VmViewAsZone::Hand,
                    VmViewAsFilter::Any,
                    false,
                ),
            },
            D34ViewAsRule {
                activation: D34ViewAsActivation::Flag("dddxiaxing_xia"),
                spec: make_spec(
                    1_926_900_009,
                    "dddxiaxing",
                    CardKind::Sha,
                    VmViewAsWindow::Any,
                    VmViewAsZone::None,
                    VmViewAsFilter::Any,
                    false,
                ),
            },
            D34ViewAsRule {
                activation: D34ViewAsActivation::Flag("dddxiaxing_xia"),
                spec: make_spec(
                    1_926_900_010,
                    "dddxiaxing",
                    CardKind::Shan,
                    VmViewAsWindow::Any,
                    VmViewAsZone::None,
                    VmViewAsFilter::Any,
                    false,
                ),
            },
            D34ViewAsRule {
                activation: D34ViewAsActivation::SkillOrFlag("ssr_yunchou", "l25_viewas_wuxie"),
                spec: make_spec(
                    1_926_900_011,
                    "ssr_yunchou",
                    CardKind::Nullify,
                    VmViewAsWindow::Any,
                    VmViewAsZone::Hand,
                    VmViewAsFilter::Any,
                    false,
                ),
            },
            D34ViewAsRule {
                activation: D34ViewAsActivation::Flag("l25_viewas_jiu"),
                spec: make_spec(
                    1_926_900_012,
                    "ssr_zhujiu",
                    CardKind::Jiu,
                    VmViewAsWindow::Any,
                    VmViewAsZone::Hand,
                    VmViewAsFilter::Any,
                    false,
                ),
            },
            D34ViewAsRule {
                activation: D34ViewAsActivation::Flag("l25_basic_as_sha"),
                spec: make_spec(
                    1_926_900_013,
                    "ssr_newjiang",
                    CardKind::Sha,
                    VmViewAsWindow::Any,
                    VmViewAsZone::Hand,
                    VmViewAsFilter::Basic,
                    false,
                ),
            },
            D34ViewAsRule {
                activation: D34ViewAsActivation::Flag("l25_bazhen_shan_guard"),
                spec: make_spec(
                    1_926_900_016,
                    "ssr_newbazhen",
                    CardKind::Shan,
                    VmViewAsWindow::Response,
                    VmViewAsZone::HandEquip,
                    VmViewAsFilter::Red,
                    false,
                ),
            },
            D34ViewAsRule {
                activation: D34ViewAsActivation::Flag("ddd_shilie_viewas_shashan"),
                spec: make_spec(
                    1_926_900_017,
                    "dddshilie",
                    CardKind::Sha,
                    VmViewAsWindow::Any,
                    VmViewAsZone::Hand,
                    VmViewAsFilter::Any,
                    false,
                ),
            },
            D34ViewAsRule {
                activation: D34ViewAsActivation::Flag("ddd_shilie_viewas_shashan"),
                spec: make_spec(
                    1_926_900_018,
                    "dddshilie",
                    CardKind::Shan,
                    VmViewAsWindow::Any,
                    VmViewAsZone::Hand,
                    VmViewAsFilter::Any,
                    false,
                ),
            },
            // `jimin` (`英雄杀/character/skill.js:1630-1664`): when equip zone is empty, any hand card -> Shan
            D34ViewAsRule {
                activation: D34ViewAsActivation::JiminNoEquip,
                spec: make_spec(
                    1_926_900_019,
                    "jimin",
                    CardKind::Shan,
                    VmViewAsWindow::Any,
                    VmViewAsZone::Hand,
                    VmViewAsFilter::Any,
                    false,
                ),
            },
            // `fz_new_rewusheng` (`标记补充/extension.js:4676-4823`): Red -> Sha (or Any -> Sha when `shouyue` is active on the table)
            D34ViewAsRule {
                activation: D34ViewAsActivation::SkillOrFlag(
                    "fz_new_rewusheng",
                    "l20_wusheng_diamond_unlimited",
                ),
                spec: make_spec(
                    1_926_900_020,
                    "fz_new_rewusheng",
                    CardKind::Sha,
                    VmViewAsWindow::Any,
                    VmViewAsZone::HandEquip,
                    VmViewAsFilter::Red,
                    false,
                ),
            },
            D34ViewAsRule {
                activation: D34ViewAsActivation::FzRewushengShouyue,
                spec: make_spec(
                    1_926_900_023,
                    "fz_new_rewusheng",
                    CardKind::Sha,
                    VmViewAsWindow::Any,
                    VmViewAsZone::HandEquip,
                    VmViewAsFilter::Any,
                    false,
                ),
            },
            // `fz_new_longdan` (`标记补充/extension.js:4860-5088`): Sha<->Shan
            D34ViewAsRule {
                activation: D34ViewAsActivation::SkillOrFlag(
                    "fz_new_longdan",
                    "l20_longdan_converted",
                ),
                spec: make_spec(
                    1_926_900_021,
                    "fz_new_longdan",
                    CardKind::Sha,
                    VmViewAsWindow::Any,
                    VmViewAsZone::Hand,
                    VmViewAsFilter::Shan,
                    false,
                ),
            },
            D34ViewAsRule {
                activation: D34ViewAsActivation::SkillOrFlag(
                    "fz_new_longdan",
                    "l20_longdan_converted",
                ),
                spec: make_spec(
                    1_926_900_022,
                    "fz_new_longdan",
                    CardKind::Shan,
                    VmViewAsWindow::Any,
                    VmViewAsZone::Hand,
                    VmViewAsFilter::Sha,
                    false,
                ),
            },
            // `txhj_leijimu` (`太虚幻境/extension_card.js:580`): Sha -> LeiSha
            D34ViewAsRule {
                activation: D34ViewAsActivation::SkillOrFlag(
                    "txhj_leijimu",
                    "l11_leijimu_thunder_sha",
                ),
                spec: make_spec(
                    1_926_900_026,
                    "txhj_leijimu",
                    CardKind::LeiSha,
                    VmViewAsWindow::Any,
                    VmViewAsZone::Hand,
                    VmViewAsFilter::Sha,
                    false,
                ),
            },
            D34ViewAsRule {
                activation: D34ViewAsActivation::Flag("u05_viewas:longdan"),
                spec: make_spec(
                    1_926_900_014,
                    "u05_longdan",
                    CardKind::Sha,
                    VmViewAsWindow::Any,
                    VmViewAsZone::Hand,
                    VmViewAsFilter::Shan,
                    false,
                ),
            },
            D34ViewAsRule {
                activation: D34ViewAsActivation::Flag("u05_viewas:longdan"),
                spec: make_spec(
                    1_926_900_015,
                    "u05_longdan",
                    CardKind::Shan,
                    VmViewAsWindow::Any,
                    VmViewAsZone::Hand,
                    VmViewAsFilter::Sha,
                    false,
                ),
            },
            // `twwushen` (`shenhua.js:1164`): Heart hand card -> distance-unlimited Sha
            D34ViewAsRule {
                activation: D34ViewAsActivation::SkillOrFlag("twwushen", "l11_mod:twwushen"),
                spec: make_spec(
                    1_926_900_060,
                    "twwushen",
                    CardKind::Sha,
                    VmViewAsWindow::Any,
                    VmViewAsZone::Hand,
                    VmViewAsFilter::Heart,
                    true,
                ),
            },
            // `rejinjiu` (`yijiang.js:5855`): Jiu hand card -> Sha
            D34ViewAsRule {
                activation: D34ViewAsActivation::SkillOrFlag("rejinjiu", "l11_mod:rejinjiu"),
                spec: make_spec(
                    1_926_900_061,
                    "rejinjiu",
                    CardKind::Sha,
                    VmViewAsWindow::Any,
                    VmViewAsZone::Hand,
                    VmViewAsFilter::Jiu,
                    false,
                ),
            },
            // `shizhi` (`sp.js:12650`): Shan hand card -> Sha when HP==1
            D34ViewAsRule {
                activation: D34ViewAsActivation::SkillOrFlag("shizhi", "l11_mod:shizhi"),
                spec: make_spec(
                    1_926_900_062,
                    "shizhi",
                    CardKind::Sha,
                    VmViewAsWindow::Any,
                    VmViewAsZone::Hand,
                    VmViewAsFilter::Shan,
                    false,
                ),
            },
            // `reguose` (`refresh.js:2443`) / `xinfu_limu` (`sp2.js:1760`): Diamond card -> Le
            D34ViewAsRule {
                activation: D34ViewAsActivation::SkillOrFlag("reguose", "l11_viewas:reguose"),
                spec: make_spec(
                    1_926_900_063,
                    "reguose",
                    CardKind::Le,
                    VmViewAsWindow::PlayPhase,
                    VmViewAsZone::HandEquip,
                    VmViewAsFilter::Diamond,
                    true,
                ),
            },
            D34ViewAsRule {
                activation: D34ViewAsActivation::Skill("xinfu_limu"),
                spec: make_spec(
                    1_926_900_064,
                    "xinfu_limu",
                    CardKind::Le,
                    VmViewAsWindow::PlayPhase,
                    VmViewAsZone::HandEquip,
                    VmViewAsFilter::Diamond,
                    true,
                ),
            },
            // `xinjiewei` (`refresh.js:3547`): Equip/hand card -> Nullify
            D34ViewAsRule {
                activation: D34ViewAsActivation::SkillOrFlag("xinjiewei", "l11_viewas:xinjiewei"),
                spec: make_spec(
                    1_926_900_065,
                    "xinjiewei",
                    CardKind::Nullify,
                    VmViewAsWindow::Any,
                    VmViewAsZone::HandEquip,
                    VmViewAsFilter::Any,
                    false,
                ),
            },
            // L19 `ganran` (`末日乱斗/extension.js:378`): Equip hand card -> Chain (`Tiesuo`)
            D34ViewAsRule {
                activation: D34ViewAsActivation::SkillOrFlag("ganran", "ganran_active"),
                spec: make_spec(
                    1_926_900_066,
                    "ganran",
                    CardKind::Chain,
                    VmViewAsWindow::PlayPhase,
                    VmViewAsZone::Hand,
                    VmViewAsFilter::Equip,
                    false,
                ),
            },
            // U08 `kanpo` (`英雄杀/character/skill.js`): Black hand card -> Nullify (`Wuxie`)
            D34ViewAsRule {
                activation: D34ViewAsActivation::SkillOrFlag("kanpo", "yxs_kanpo_viewas"),
                spec: make_spec(
                    1_926_900_067,
                    "kanpo",
                    CardKind::Nullify,
                    VmViewAsWindow::Any,
                    VmViewAsZone::Hand,
                    VmViewAsFilter::Black,
                    false,
                ),
            },
            // U08 `fenghuo` (`英雄杀/character/skill.js:2120-2143`): Equip zone card -> SavageAttack (`Nanman`) in PlayPhase
            D34ViewAsRule {
                activation: D34ViewAsActivation::SkillOrFlag("fenghuo", "yxs_fenghuo_viewas"),
                spec: make_spec(
                    1_926_900_068,
                    "fenghuo",
                    CardKind::SavageAttack,
                    VmViewAsWindow::PlayPhase,
                    VmViewAsZone::Equip,
                    VmViewAsFilter::Any,
                    false,
                ),
            },
        ];

        let u05_flag_map: &[(&str, CardKind)] = &[
            ("u05_viewas:sha", CardKind::Sha),
            ("u05_viewas:stonesha", CardKind::Sha),
            ("u05_viewas:shan", CardKind::Shan),
            ("u05_viewas:stoneshan", CardKind::Shan),
            ("u05_viewas:tao", CardKind::Tao),
            ("u05_viewas:caoyao", CardKind::Tao),
            ("u05_viewas:jiu", CardKind::Jiu),
            ("u05_viewas:jiuchi", CardKind::Jiu),
            ("u05_viewas:wuxie", CardKind::Nullify),
            ("u05_viewas:wuzhong", CardKind::Wuzhong),
            ("u05_viewas:juedou", CardKind::Duel),
            ("u05_viewas:nanman", CardKind::SavageAttack),
            ("u05_viewas:luanji", CardKind::ArcheryAttack),
            ("u05_viewas:shunshou", CardKind::Snatch),
            ("u05_viewas:taoyuan", CardKind::Taoyuan),
            ("u05_viewas:shuigong", CardKind::Shuigong),
            ("u05_viewas:toulianghuanzhu", CardKind::Toulianghuanzhu),
            ("u05_viewas:bingpotong", CardKind::Bingpotong),
            ("u05_viewas:qiankunbiao", CardKind::Qiankunbiao),
            ("u05_viewas:feibiao", CardKind::Feibiao),
            ("u05_viewas:shihuifen", CardKind::Shihuifen),
            ("u05_viewas:liuxinghuoyu", CardKind::FireSha),
            ("u05_viewas:jingleishan", CardKind::LeiSha),
            ("u05_viewas:chiyuxi", CardKind::FireSha),
            ("l11_viewas:kyou_zhidian", CardKind::Sha),
            ("l11_viewas:dclbjiuxian", CardKind::Jiu),
            ("l11_viewas:reluanji", CardKind::ArcheryAttack),
            ("l11_mod:reshuangxiong_duel", CardKind::Duel),
            ("l11_viewas:xinfu_sidao", CardKind::Snatch),
            ("l11_viewas:weijing", CardKind::Sha),
            ("l11_viewas:zhanyi_basic", CardKind::Sha),
            ("l11_viewas:mozhi", CardKind::Sha),
        ];
        for (idx, &(flag_name, kind)) in u05_flag_map.iter().enumerate() {
            v.push(D34ViewAsRule {
                activation: D34ViewAsActivation::Flag(flag_name),
                spec: make_spec(
                    1_926_900_100 + idx as u32,
                    flag_name,
                    kind,
                    VmViewAsWindow::Any,
                    VmViewAsZone::HandEquip,
                    VmViewAsFilter::Any,
                    false,
                ),
            });
        }
        // L20 标记补充 (`_aozhan_judge` / `aozhan`, `extension.js:2152`): Tao in hand can be used/responded as Sha or Shan
        v.push(D34ViewAsRule {
            activation: D34ViewAsActivation::SkillOrFlag("aozhan", "aozhan_active"),
            spec: make_spec(
                1_926_900_300,
                "aozhan",
                CardKind::Sha,
                VmViewAsWindow::Any,
                VmViewAsZone::Hand,
                VmViewAsFilter::Tao,
                false,
            ),
        });
        v.push(D34ViewAsRule {
            activation: D34ViewAsActivation::SkillOrFlag("aozhan", "aozhan_active"),
            spec: make_spec(
                1_926_900_301,
                "aozhan_shan",
                CardKind::Shan,
                VmViewAsWindow::Any,
                VmViewAsZone::Hand,
                VmViewAsFilter::Tao,
                false,
            ),
        });
        v
    })
}

pub fn find_d34_flag_view_as(
    core: &GameStateCore,
    catalog: &Catalog,
    player: PlayerId,
    card: CardId,
    in_play: bool,
    target_kind: Option<CardKind>,
) -> Option<&'static VmViewAsSpec> {
    if core.vm_aux.flags.is_empty() && catalog.skills.len() <= 24 {
        return None;
    }
    let sealed = catalog.is_tieji_sealed(core, player);
    for entry in d34_view_as_table() {
        if sealed && !Catalog::is_locked_skill_name(&entry.spec.skill_name, None) {
            continue;
        }
        let active = match entry.activation {
            D34ViewAsActivation::Skill(sk) => has_ext_skill(core, catalog, player, sk),
            D34ViewAsActivation::Flag(fl) => flag_val(core, player, fl) > 0,
            D34ViewAsActivation::SkillOrFlag(sk, fl) => {
                has_ext_skill(core, catalog, player, sk) || flag_val(core, player, fl) > 0
            }
            D34ViewAsActivation::JiminNoEquip => {
                core.player(player).equips.is_empty()
                    && (has_ext_skill(core, catalog, player, "jimin")
                        || flag_val(core, player, "yxs_jimin_shan") > 0)
            }
            D34ViewAsActivation::FzRewushengShouyue => {
                (has_ext_skill(core, catalog, player, "fz_new_rewusheng")
                    || flag_val(core, player, "l20_wusheng_diamond_unlimited") > 0)
                    && (0..core.players.len() as PlayerId).any(|p| {
                        core.player(p).alive
                            && (has_ext_skill(core, catalog, p, "shouyue")
                                || flag_val(core, p, "shouyue_active") > 0)
                    })
            }
        };
        if active && entry.spec.matches_card(core, catalog, player, card, in_play, target_kind) {
            return Some(&entry.spec);
        }
    }
    None
}
