//! 武将装配：明置、调试装配和内容转将共用的状态更新。

use crate::ids::PlayerId;

/// 装配授予（P2/N08）：按武将表配置体力上限（主将槽）并授予技能集。
/// 国战明置（`RevealGeneral`，ADR 0769）与原型装配原语
/// （`debug_assemble_general`）共用此逻辑，授予语义不得分叉。
/// Whether `name` overrides starting HP to differ from `max_hp` (or uses `ddd_zhangkai` `max_hp - 1`).
pub fn has_custom_starting_hp(name: &str) -> bool {
    matches!(
        name,
        "ddd_zhangkai"
            | "th_guanning"
            | "ssr_machao"
            | "ssr_taishici"
            | "ssr_re_lidian"
            | "txhj_molijue"
            | "txhj_handiliuxie"
            | "txhj_niufu"
            | "txhj_dongxie"
            | "txhj_xurongy"
            | "txhj_mamidi"
            | "txhj_beimihu"
            | "txhj_resunjian"
            | "txhj_caocaox"
            | "txhj_yuanshaoyou"
            | "txhj_shenganningx"
            | "txhj_sunguan"
            | "txhj_sunjian"
            | "txhj_guanqiujian"
            | "txhj_shensimayi"
            | "txhj_baimaxiaoqi"
            | "txhj_huanjingnantong"
            | "txhj_lingjitongren"
            | "txhj_fuguangtongren"
            | "txhj_hanlingdi"
            | "txhj_dongzhuo"
            | "txhj_huanyanzhangliang"
            | "txhj_huanyanzhangbao"
            | "txhj_huanyanzhangjiaoshi"
            | "txhj_huanyanzhangjiaolei"
            | "txhj_huanyanzhangjiaohuo"
            | "txhj_huanyanzhangjiaofeng"
            | "txhj_zhaozhong"
    )
}

/// Starting HP when assembling `name` with resolved `max_hp` (`L11` `hp/maxHp[/hujia]` + `U01`/`L28`/`L25`).
pub fn general_starting_hp(name: &str, max_hp: u32) -> i32 {
    let base = match name {
        "ddd_zhangkai" => 3,
        "th_guanning" => 3,
        "ssr_machao" | "ssr_taishici" => 2,
        "ssr_re_lidian" => 3,
        "txhj_handiliuxie" | "txhj_xurongy" | "txhj_yuanshaoyou" | "txhj_huanjingnantong" => 2,
        "txhj_mamidi" | "txhj_resunjian" | "txhj_shenganningx" | "txhj_baimaxiaoqi" => 3,
        "txhj_dongxie"
        | "txhj_sunguan"
        | "txhj_sunjian"
        | "txhj_guanqiujian"
        | "txhj_shensimayi"
        | "txhj_lingjitongren"
        | "txhj_fuguangtongren"
        | "txhj_hanlingdi" => 4,
        "txhj_caocaox" => 5,
        "txhj_niufu" | "txhj_dongzhuo" | "txhj_zhaozhong" => 6,
        "txhj_huanyanzhangliang" => 7,
        "txhj_huanyanzhangjiaoshi" => 8,
        "txhj_huanyanzhangbao" | "txhj_huanyanzhangjiaolei" => 10,
        "txhj_huanyanzhangjiaohuo" | "txhj_huanyanzhangjiaofeng" => 12,
        "txhj_beimihu" => 15,
        "txhj_molijue" => 17,
        _ => return max_hp as i32,
    };
    base.min(max_hp as i32)
}

/// Starting Hujia (`护甲`) when assembling `name` (`L11` `hp/maxHp/hujia` + `L25`).
pub fn general_starting_hujia(name: &str) -> u8 {
    match name {
        "ssr_machao" | "ssr_taishici" => 2,
        "ssr_re_lidian" => 1,
        "txhj_huangjinzhangyan" => 1,
        "txhj_molijue" | "txhj_huangfusongxx" | "txhj_bossbocai" => 2,
        "txhj_buji" => 3,
        "txhj_digongzhangbaox" => 7,
        _ => 0,
    }
}

pub fn apply_general_assembly(
    state: &mut crate::state::GameStateCore,
    catalog: &crate::catalog::Catalog,
    player: PlayerId,
    slot: usize,
) {
    let g = state.players[player as usize].general_slots[slot].expect("general slot configured");
    let meta = catalog.general_meta(g);
    let p = state.players.get_mut(player as usize).expect("player");
    if slot == 0 {
        if let Some((name, _, _, hp, _)) = meta {
            if name == "ddd_zhangkai" {
                p.max_hp = 4;
                p.hp = p.hp.min(3);
            } else if name == "th_guanning" {
                p.max_hp = 7;
                p.hp = p.hp.min(3);
            } else if name == "ssr_machao" || name == "ssr_taishici" {
                p.max_hp = 4;
                p.hp = p.hp.min(2);
                p.hujia = 2;
            } else if name == "ssr_re_lidian" {
                p.max_hp = 4;
                p.hp = p.hp.min(3);
                p.hujia = 1;
            } else {
                p.max_hp = hp;
                let start_hp = general_starting_hp(name, hp);
                if p.hp > start_hp {
                    p.hp = start_hp;
                }
                let start_hujia = general_starting_hujia(name);
                if start_hujia > 0 {
                    p.hujia = start_hujia;
                }
            }
        }
    }
    let skills: Vec<&'static str> = meta.map(|(_, _, skills, _, _)| skills).unwrap_or_default();
    // 禳星（T11 Boss 诸葛亮族批 115，规则 v303）：装配即初始化星系统
    //（上游 `hlss_xiangxing` init：3 枚星、计数 0、无受伤记录）。
    if skills.contains(&crate::vocabulary::host::HLSS_XIANGXING_KEY) {
        let p = state.players.get_mut(player as usize).expect("player");
        p.hlss_stars = 3;
        p.hlss_count = 0;
        p.hlss_damage = 0;
    }
    for skill in skills {
        let def = catalog
            .skill_by_name(skill)
            .unwrap_or_else(|| panic!("catalog skill {skill}"));
        // 隐贼/延妒（Boss 混沌/穷奇，规则 v210/v213）：装配即未激活
        //（上游 gameStart switch_off disableSkill 同口径；体力首次≤
        // 一半时启用）
        let enabled = skill != crate::vocabulary::names::BOSS_YINZEI
            && skill != crate::vocabulary::names::BOSS_YANDU
            && skill != crate::vocabulary::names::BOSS_JICAI
            && skill != crate::vocabulary::names::BOSS_LUANCHANG
            && skill != crate::vocabulary::names::BOSS_ECHOU
            && skill != crate::vocabulary::names::BOSS_XUSHI
            && skill != crate::vocabulary::names::BOSS_YANYU
            && skill != crate::vocabulary::names::BOSS_SIPU;
        state.skills.insert(
            (player, def),
            crate::state::SkillInstance {
                owner: player,
                enabled,
                once_used: false,
                expires_end_of_turn: false,
                rounds: 0,
            },
        );
        if skill == "ssr_luoyi" {
            let _ = crate::kernel::discard_armor_for_ssr_luoyi(state, catalog, player);
        }
        if let Some(behavior) = catalog.skills[def as usize].behavior.as_host_behavior() {
            for &(child_name, _) in behavior.group_children() {
                if let Some(cdef) = catalog.skill_by_name(child_name) {
                    state.skills.entry((player, cdef)).or_insert(
                        crate::state::SkillInstance {
                            owner: player,
                            enabled: true,
                            once_used: false,
                            expires_end_of_turn: false,
                            rounds: 0,
                        },
                    );
                }
            }
        }
        for &sub in group_subskills(skill) {
            if let Some(sub_def) = catalog.skill_by_name(sub) {
                state.skills.entry((player, sub_def)).or_insert(crate::state::SkillInstance {
                    owner: player,
                    enabled,
                    once_used: false,
                    expires_end_of_turn: false,
                    rounds: 0,
                });
            }
        }
    }
    sync_dddfusi_global(state, catalog);
}

/// 上游技能声明的 `group:` 子技（主技随将装配时一并挂载）。
fn group_subskills(skill: &str) -> &'static [&'static str] {
    match skill {
        "yanzhan" => &["yanzhan2"],
        "xfenxin" => &["xfenxin2"],
        "xuanyan" => &["xuanyan2", "xuanyan3"],
        "tianshe" => &["tianshe2"],
        "husha" => &["husha_count"],
        "xshuangren" => &["xshuangren2"],
        "shuiyun" => &["shuiyun5"],
        "wangyou" => &["wangyou2"],
        "luanjian" => &["luanjian2"],
        "ctianfu" => &["ctianfu3"],
        "leiyu" => &["leiyu2", "leiyu4"],
        "danqing" => &["danqing_count"],
        "zhuyue" => &["zhuyue2"],
        "xjzhimeng" => &["xjzhimeng3"],
        _ => &[],
    }
}

/// Grants global U01 companion skills (`dddfusi_global`, `dddfengzheng_global`) to every seat iff at least one seat holds their owner skill.
pub fn sync_dddfusi_global(
    state: &mut crate::state::GameStateCore,
    catalog: &crate::catalog::Catalog,
) {
    let sync_pair = |state: &mut crate::state::GameStateCore, src: &str, glob: &str| {
        let (Some(src_def), Some(global_def)) =
            (catalog.skill_by_name(src), catalog.skill_by_name(glob))
        else {
            return;
        };
        let n = state.players.len() as PlayerId;
        if (0..n).any(|p| state.has_skill(p, src_def)) {
            for p in 0..n {
                state.skills.entry((p, global_def)).or_insert(crate::state::SkillInstance {
                    owner: p,
                    enabled: true,
                    once_used: false,
                    expires_end_of_turn: false,
                    rounds: 0,
                });
            }
        } else {
            for p in 0..n {
                state.skills.remove(&(p, global_def));
            }
        }
    };
    sync_pair(state, "dddfusi", "dddfusi_global");
    sync_pair(state, "dddfengzheng", "dddfengzheng_global");
}
