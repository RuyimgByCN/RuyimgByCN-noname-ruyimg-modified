//! 武将装配：明置、调试装配和内容转将共用的状态更新。

use crate::ids::PlayerId;

/// 装配授予（P2/N08）：按武将表配置体力上限（主将槽）并授予技能集。
/// 国战明置（`RevealGeneral`，ADR 0769）与原型装配原语
/// （`debug_assemble_general`）共用此逻辑，授予语义不得分叉。
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
            } else {
                p.max_hp = hp;
                if p.hp > hp as i32 {
                    p.hp = hp as i32;
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
    }
}
