//! L11 太虚幻境（`taixuhuanjing`）原生战斗局构建器（`taixu_battle.rs`）。
//!
//! 从战役层传入己方与敌方参数，直接构造 `GameMode::Versus` 权威对局状态：
//! - 保留标准牌堆守恒（所有初始手牌/牌库标签牌/装备牌均从目录实牌调度）；
//! - 自动装配武将、体力/上限、装备与技能、系统胜负钩子、难度与突发效果修饰；
//! - 完整支持二进制快照保存与恢复。

use std::sync::Arc;

use crate::catalog::{EquipSlot, SharedCatalog};
use crate::content::extension::{ExtensionError, ExtensionPack};
use crate::engine::{Engine, OpeningPlan};
use crate::ids::{CardId, PlayerId};
use crate::state::{apply_general_assembly, GameMode, SkillInstance};

/// 太虚幻境对局中的单个 NPC（敌方或友方）配置。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaixuBattleNpc {
    pub seat: PlayerId,
    pub is_friend: bool,
    pub is_boss: bool,
    pub general_name: String,
    pub max_hp: u32,
    pub hp: u32,
    pub min_hs: u32,
    pub equip_names: Vec<String>,
    pub card_names: Vec<String>,
    pub skills: Vec<String>,
}

/// 太虚幻境单场战役战斗的原生建局参数（对应 `extension_framework.js:9505-10243`）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaixuBattleSetup {
    pub seed: u64,
    pub player_seat: PlayerId,
    pub player_general: String,
    pub player_max_hp: u32,
    pub player_hp: u32,
    pub player_min_hs: u32,
    pub player_max_hs_bonus: u32,
    pub player_adjust: u32,
    pub rank: u8,
    pub servant_grade: u8,
    pub target_kill: bool,
    pub target_repair: bool,
    pub target_subsist: bool,
    pub max_rounds: u32,
    pub player_paiku_cards: Vec<String>,
    pub player_equip_names: Vec<String>,
    pub player_skills: Vec<String>,
    pub npcs: Vec<TaixuBattleNpc>,
}

fn find_general_def(catalog: &crate::catalog::Catalog, name: &str) -> Option<u16> {
    (0..catalog.general_count()).find_map(|idx| {
        let def = idx as u16;
        catalog
            .general_meta(def)
            .is_some_and(|(gname, _, _, _, _)| gname == name)
            .then_some(def)
    })
}

fn resolve_standard_skill_alias(name: &str) -> Option<&'static str> {
    use crate::vocabulary::names::*;
    match name {
        "mashu" => Some(MA_SHU),
        "yaowu" => Some(YAO_WU),
        "wushuang" => Some(WU_SHUANG),
        "yingzi" => Some(YING_ZI),
        "paoxiao" => Some(PAO_XIAO),
        "tieji" => Some(TIE_JI),
        "kongcheng" => Some(KONG_CHENG),
        "qianxun" => Some(QIAN_XUN),
        "qicai" => Some(QI_CAI),
        "fankui" => Some(FAN_KUI),
        "ganglie" => Some(GANG_LIE),
        "jianxiong" => Some(JIAN_XIONG),
        "guicai" => Some(GUI_CAI),
        "guidao" => Some(GUI_DAO),
        "tiandu" => Some(TIAN_DU),
        "tuxi" => Some(TU_XI),
        "luoyi" => Some(LUO_YI),
        "qingguo" => Some(QING_GUO),
        "luoshen" => Some(LUO_SHEN),
        "biyue" => Some(BI_YUE),
        "wusheng" => Some(WU_SHENG),
        "longdan" => Some(LONG_DAN),
        "rende" => Some(REN_DE),
        "jijiang" => Some(JI_JIANG),
        "guanxing" => Some(GUAN_XING),
        "guose" => Some(GUO_SE),
        "keji" => Some(KE_JI),
        "zhiheng" => Some(ZHI_HENG),
        "qixi" => Some(QI_XI),
        "kurou" => Some(KU_ROU),
        "lianying" => Some(LIAN_YING),
        "lijian" => Some(LI_JIAN),
        "xiaoji" => Some(XIAO_JI),
        "qingnang" => Some(QING_NANG),
        "jijiu" => Some(JI_JIU),
        "jiuyuan" => Some(JIU_YUAN),
        "hujia" => Some(HU_JIA),
        "fanjian" => Some(FAN_JIAN),
        "jieyin" => Some(JIE_YIN),
        "liuli" => Some(LIU_LI),
        "jueqing" => Some(JUE_QING),
        _ => None,
    }
}

fn grant_named_skill_on_core(
    core: &mut crate::state::GameStateCore,
    catalog: &crate::catalog::Catalog,
    seat: PlayerId,
    name: &str,
) {
    let def = catalog
        .skill_by_name(name)
        .or_else(|| resolve_standard_skill_alias(name).and_then(|al| catalog.skill_by_name(al)));
    if let Some(def) = def {
        core.skills.insert(
            (seat, def),
            SkillInstance {
                owner: seat,
                enabled: true,
                once_used: false,
                expires_end_of_turn: false,
                rounds: 0,
            },
        );
    }
}

fn take_named_card_exact(
    core: &mut crate::state::GameStateCore,
    catalog: &crate::catalog::Catalog,
    name: &str,
    allow_detached: bool,
) -> Option<CardId> {
    if let Some(pos) = core.deck.iter().position(|&c| catalog.cards[c as usize].name == name) {
        return Some(core.deck.remove(pos));
    }
    if let Some(pos) = core
        .discard
        .iter()
        .position(|&c| catalog.cards[c as usize].name == name)
    {
        return Some(core.discard.remove(pos));
    }
    if allow_detached {
        if let Some(pos) = core
            .detached_cards
            .iter()
            .position(|&(c, k)| {
                k == crate::event::DetachedKind::Unclaimed && catalog.cards[c as usize].name == name
            })
        {
            return Some(core.detached_cards.remove(pos).0);
        }
    }
    None
}

fn taixu_card_fallback_candidates(name: &str) -> &'static [&'static str] {
    match name {
        "wanjian" => &["archery"],
        "nanman" => &["savage"],
        "guohe" => &["dismantle"],
        "juedou" => &["duel"],
        "lebu" => &["le"],
        "bingliang" => &["bing"],
        "shunshou" => &["snatch"],
        "wuxie" => &["nullify"],
        "tiesuo" => &["chain"],
        "bagua" => &["armor"],
        "zixin" | "zixing" | "dawan" | "dayuan" | "chitu" | "jingfanma" => &["horse_minus"],
        "dilu" | "jueying" | "zhuahuang" | "hualiu" => &["horse_plus"],
        "txzhaoshu" => &["txzhaoshu_skill", "txzhaoshu_global"],
        "txhj_shengdongjixi" => &["txhj_shengdong"],
        "txhj_shuiyanqijun" => &["txhj_shuiyan"],
        "txhj_feilongduofeng" => &["txhj_feilong"],
        _ => &[],
    }
}

fn is_taixu_treasure_name(name: &str) -> bool {
    matches!(
        name,
        "txhj_xuwangzhimian"
            | "txhj_dinglanyemingzhu"
            | "txzhaoshu"
            | "txzhaoshu_skill"
            | "txzhaoshu_global"
    )
}

fn take_named_card(
    core: &mut crate::state::GameStateCore,
    catalog: &crate::catalog::Catalog,
    name: &str,
    allow_detached: bool,
) -> Option<CardId> {
    if let Some(cid) = take_named_card_exact(core, catalog, name, allow_detached) {
        return Some(cid);
    }
    for &cand in taixu_card_fallback_candidates(name) {
        if let Some(cid) = take_named_card_exact(core, catalog, cand, allow_detached) {
            return Some(cid);
        }
    }
    None
}

fn equip_named_on_core(
    core: &mut crate::state::GameStateCore,
    catalog: &crate::catalog::Catalog,
    seat: PlayerId,
    name: &str,
) {
    let (lookup_name, is_rewritten) = match name.strip_prefix("rewrite_") {
        Some(base) => (base, true),
        None => (name, false),
    };
    let Some(cid) = take_named_card(core, catalog, lookup_name, true) else {
        return;
    };
    if is_taixu_treasure_name(lookup_name) {
        core.declared_slots.insert(cid, EquipSlot::Treasure);
    }
    let slot = catalog.card_slot_of(core, cid);
    let allow_dual_treasure = slot == Some(EquipSlot::Treasure)
        && core.player(seat).equips.iter().filter(|&&c| {
            catalog.card_slot_of(core, c) == Some(EquipSlot::Treasure)
        }).count() < 2
        && catalog
            .skill_by_name("_txhj_equip5")
            .is_some_and(|def| core.has_skill(seat, def));
    if let Some(target_slot) = slot {
        if !allow_dual_treasure {
            let displaced: Vec<CardId> = core
                .player(seat)
                .equips
                .iter()
                .copied()
                .filter(|&c| catalog.card_slot_of(core, c) == Some(target_slot))
                .collect();
            for old in displaced {
                core.player_mut(seat).equips.retain(|&c| c != old);
                crate::kernel::revoke_equip_skills(core, catalog, seat, old);
                core.discard.push(old);
            }
        }
    }
    core.player_mut(seat).equips.push(cid);
    if is_rewritten {
        core.vm_aux
            .flags
            .insert((0, format!("th_jingxie_rewritten:{cid}")), 1);
    }
    for sk in &catalog.cards[cid as usize].equip_skills {
        grant_named_skill_on_core(core, catalog, seat, sk);
    }
}

fn artifact_buff_for_card(name: &str) -> Option<&'static str> {
    match name {
        "txhj_feilong" | "txhj_feilongduofeng" => Some("buff_txhj_pozhenzhifeng"),
        "txhj_chiyanzhenhunqin" => Some("buff_txhj_yanhuozhiren"),
        "txhj_lingbaoxianhu" => Some("buff_txhj_xianhujiqu"),
        "txhj_taijifuchen" => Some("buff_txhj_fuchendangmo"),
        "txhj_chongyingshenfu" | "txhj_xuwangzhimian" => Some("buff_txhj_shenfuhuaxie"),
        "txhj_guofengyupao" => Some("buff_txhj_juejingzhice"),
        "txhj_juechenjinge" => Some("buff_txhj_wuzhongshengshan"),
        "txhj_liulongcanjia" => Some("buff_txhj_wuzhongshengsha"),
        _ => None,
    }
}

impl Engine {
    /// 构造一场原生太虚幻境战役战斗对局。
    pub fn new_taixu_versus(
        setup: &TaixuBattleSetup,
        packs: &[&ExtensionPack],
    ) -> Result<Self, ExtensionError> {
        let seats = 1 + setup.npcs.len();
        assert!(seats >= 2 && seats <= 8, "太虚幻境对局座位数需在 2..=8 之间");
        let mut friends = vec![setup.player_seat];
        let mut enemies = Vec::new();
        for npc in &setup.npcs {
            assert!((npc.seat as usize) < seats, "NPC 座位越界");
            if npc.is_friend {
                friends.push(npc.seat);
            } else {
                enemies.push(npc.seat);
            }
        }
        assert!(!enemies.is_empty(), "太虚幻境对局至少需要一名敌方角色");

        let catalog: SharedCatalog =
            Arc::new(crate::content::standard::standard_catalog_with_extensions(packs)?);
        let mut engine = Self::new_with_catalog(
            setup.seed,
            seats,
            GameMode::Versus {
                teams: vec![friends, enemies],
                substitutes: vec![Vec::new(), Vec::new()],
            },
            catalog,
            OpeningPlan::Standard,
        );

        let me = setup.player_seat;
        let catalog_ref = engine.catalog.clone();
        let core = &mut engine.state.core;

        // 回收 OpeningPlan 临时手牌至牌库尾部，确保按各角色真实 `minHs` 与「牌库」卡组发牌且守恒。
        let mut returned = Vec::new();
        for seat in 0..seats as PlayerId {
            returned.append(&mut core.player_mut(seat).hand);
        }
        core.deck.extend(returned);

        // 重置为第 1 轮起始状态（从 0 号位 TurnBegin → Phase::Prepare 启动）
        core.turn.current = 0;
        core.turn.phase = crate::event::Phase::Prepare;
        core.turn.round = 1;
        core.turn.turn_opened = true;

        // 装配玩家武将与初始属性
        if let Some(def) = find_general_def(&catalog_ref, &setup.player_general) {
            core.players[me as usize].general_slots[0] = Some(def);
            apply_general_assembly(core, &catalog_ref, me, 0);
        }
        core.player_mut(me).max_hp = setup.player_max_hp.max(1);
        core.player_mut(me).hp = setup.player_hp.clamp(1, setup.player_max_hp.max(1)) as i32;

        // 授予太虚幻境系统级钩子技
        for sys in [
            "_txhj_skill_Timelimit",
            "_txhj_skill_maxCards",
            "_txhj_skill_addEffectStyle",
            "_txhj_equip5",
            "_txhj_shenqi",
            "_txhj_XXXshenqi",
            "_txhj_skill_gaincard",
            "_txhj_skill_discard",
            "_txhj_skill_usecard",
            "_txhj_skill_damage",
            "_txhj_skills_kill",
        ] {
            grant_named_skill_on_core(core, &catalog_ref, me, sys);
        }
        for sk in &setup.player_skills {
            grant_named_skill_on_core(core, &catalog_ref, me, sk);
        }

        // 玩家「牌库」持久化手牌与初始手牌
        for cname in &setup.player_paiku_cards {
            if let Some(cid) = take_named_card(core, &catalog_ref, cname, false) {
                core.player_mut(me).hand.push(cid);
                core.vm_aux
                    .flags
                    .insert((me, format!("l11_gaintag_paiku:{cid}")), 1);
            }
        }
        for _ in 0..setup.player_min_hs {
            if let Some(cid) = core.deck.pop() {
                core.player_mut(me).hand.push(cid);
            }
        }
        for eq in &setup.player_equip_names {
            equip_named_on_core(core, &catalog_ref, me, eq);
            if let Some(buff) = artifact_buff_for_card(eq) {
                grant_named_skill_on_core(core, &catalog_ref, me, buff);
            }
        }

        // 装配友方/敌方 NPC
        for npc in &setup.npcs {
            let s = npc.seat;
            if let Some(def) = find_general_def(&catalog_ref, &npc.general_name) {
                core.players[s as usize].general_slots[0] = Some(def);
                apply_general_assembly(core, &catalog_ref, s, 0);
            }
            if npc.max_hp > 0 {
                core.player_mut(s).max_hp = npc.max_hp;
            }
            if npc.hp > 0 {
                core.player_mut(s).hp = npc.hp as i32;
            }
            for sk in &npc.skills {
                grant_named_skill_on_core(core, &catalog_ref, s, sk);
            }
            for cname in &npc.card_names {
                if let Some(cid) = take_named_card(core, &catalog_ref, cname, false) {
                    core.player_mut(s).hand.push(cid);
                }
            }
            for _ in 0..npc.min_hs {
                if let Some(cid) = core.deck.pop() {
                    core.player_mut(s).hand.push(cid);
                }
            }
            for eq in &npc.equip_names {
                equip_named_on_core(core, &catalog_ref, s, eq);
            }
            if npc.is_boss {
                core.vm_aux.flags.insert((s, "l11_npc_boss".to_string()), 1);
            }
        }

        // 扩展全局技能 + 模式旗标
        for &def in &catalog_ref.extension_global_skills {
            for p in 0..seats as PlayerId {
                core.skills.insert(
                    (p, def),
                    SkillInstance {
                        owner: p,
                        enabled: true,
                        once_used: false,
                        expires_end_of_turn: false,
                        rounds: 0,
                    },
                );
            }
        }

        core.vm_aux
            .flags
            .insert((me, "l11_txhj_active_battle".to_string()), 1);
        core.vm_aux
            .flags
            .insert((me, "l11_txhj_rank".to_string()), u32::from(setup.rank.max(1)));
        if setup.player_max_hs_bonus > 0 {
            core.vm_aux
                .flags
                .insert((me, "l11_txhj_max_hs".to_string()), setup.player_max_hs_bonus);
        }
        if setup.player_adjust > 0 {
            core.vm_aux
                .flags
                .insert((me, "l11_txhj_adjust_rem".to_string()), setup.player_adjust);
        }
        if setup.max_rounds > 1 {
            core.vm_aux
                .flags
                .insert((me, "l11_timelimit_max".to_string()), setup.max_rounds);
        }
        if setup.target_repair {
            core.vm_aux
                .flags
                .insert((me, "l11_timelimit_repair".to_string()), 1);
        }
        if setup.target_subsist {
            core.vm_aux
                .flags
                .insert((me, "l11_timelimit_subsist".to_string()), 1);
        }
        if setup.target_kill {
            core.vm_aux
                .flags
                .insert((me, "l11_timelimit_target_kill".to_string()), 1);
        }
        core.vm_aux.flags.insert(
            (me, "l11_servant_grade".to_string()),
            u32::from(setup.servant_grade),
        );

        crate::kernel::elo_on_game_start(core, &catalog_ref);
        crate::resolution::push_frame(
            &mut engine.state.resolution,
            crate::event::EventData::TurnBegin { player: 0 },
        );
        let _ = crate::resolution::run(
            &mut engine.state.core,
            &mut engine.state.resolution,
            &engine.catalog,
            &mut engine.log,
            None,
        );
        Ok(engine)
    }
}
