//! Primitive effect scheduler.
//!
//! All rule mutations go through typed effects. The scheduler uses an explicit
//! FIFO queue (architecture §7) — not Rust recursion or host Promise chains.

mod primitives;
mod source_skill;
pub(crate) use primitives::discardable_weapon_cards;
pub(crate) use primitives::enable_equip_slot;
pub(crate) use primitives::gaoyuan_redirect_candidates;
pub(crate) use primitives::jsrgdaoren_entry;
pub(crate) use primitives::materialize_virtual_card;
pub(crate) use primitives::meiyong_hej_cards;
pub(crate) use primitives::pxqixian_step;
pub(crate) use primitives::qiangyong_bonus_if_no_hand;
pub(crate) use primitives::recover_hp;
pub(crate) use primitives::resolve_biaozhao3_stage;
pub(crate) use primitives::resolve_canyun_discarded;
pub(crate) use primitives::resolve_canyun_resolve;
pub(crate) use primitives::resolve_choose_group;
pub(crate) use primitives::resolve_choose_others_add_mark;
pub(crate) use primitives::resolve_choose_others_insert_extra_turn;
pub(crate) use primitives::resolve_choose_place_hand_xumou;
pub(crate) use primitives::resolve_chumei_give;
pub(crate) use primitives::resolve_chumei_give_pick;
pub(crate) use primitives::resolve_chumei_stage;
pub(crate) use primitives::resolve_city_consume;
pub(crate) use primitives::resolve_dcbingji_entry;
pub(crate) use primitives::resolve_dcmoukui_conseq_discard;
pub(crate) use primitives::resolve_dcmoukui_discard;
pub(crate) use primitives::resolve_dcmoukui_stage;
pub(crate) use primitives::resolve_discard_basic_draw_equal;
pub(crate) use primitives::resolve_distribute_shadow_cards;
pub(crate) use primitives::resolve_duoduan;
pub(crate) use primitives::resolve_duoduan_draw_and_cancel;
pub(crate) use primitives::resolve_jsrgzuozhan_gain_stage;
pub(crate) use primitives::resolve_jsrgzuozhan_stage;
pub(crate) use primitives::resolve_mbbeiming_grant;
pub(crate) use primitives::resolve_ol_jie_danshou_choice;
pub(crate) use primitives::resolve_pin_debate_opinions;
pub(crate) use primitives::resolve_qiangyong;
pub(crate) use primitives::resolve_qianlong_stage;
pub(crate) use primitives::resolve_qiantun_player_card;
pub(crate) use primitives::resolve_qiantun_resolve;
pub(crate) use primitives::resolve_qiantun_shown;
pub(crate) use primitives::resolve_qiantun_stage;
pub(crate) use primitives::resolve_register_turn_end_debate;
pub(crate) use primitives::resolve_reliangyin;
pub(crate) use primitives::resolve_reliangyin_discard;
pub(crate) use primitives::resolve_reliangyin_stage;
pub(crate) use primitives::resolve_shuaiyan_give;
pub(crate) use primitives::resolve_shuaiyan_stage;
pub(crate) use primitives::resolve_start_debate_targets;
pub(crate) use primitives::resolve_twdaigui_round;
pub(crate) use primitives::resolve_twdaigui_stage;
pub(crate) use primitives::resolve_twjieqiu_select;
pub(crate) use primitives::resolve_twmitu_round;
pub(crate) use primitives::resolve_twmitu_stage;
pub(crate) use primitives::resolve_twxiongjun;
pub(crate) use primitives::resolve_xinfu_qiai;
pub(crate) use primitives::resolve_yangming_round;
pub(crate) use primitives::resolve_yangming_stage;
pub(crate) use primitives::resolve_yixiang;
pub(crate) use primitives::resolve_zhongyan_pick;
pub(crate) use primitives::resolve_zhongyan_stage;
pub(crate) use primitives::resolve_zhongyan_swap;
pub(crate) use primitives::xinfu_qiai_next_giver;
pub(crate) use primitives::ZHENGJIAN_MARK;
pub(crate) use primitives::{
    choose_gain_other_skill, lose_gained_skill, resolve_choose_gain_other_skill,
    resolve_lose_gained_skill, resolve_potheyun, resolve_potheyun_pick,
};
pub(crate) use primitives::{
    commit_guanxing_top, commit_hand_compare, commit_hand_compare_between, commit_hand_compare_ext,
    commit_hand_compare_second_failure_seat_damage, commit_mbfeijing_color,
    commit_mbjianji_compare, commit_opening_extra_play_hp_loss_gain, commit_potbihan,
    commit_potfuan_branch, commit_potfuan_suit, commit_potfuan_target,
    commit_potsifeng_distribution, continue_mbfeijing_card_choices, discard_cards,
    effective_card_category, eu_tuxi_context_is_current, eu_tuxi_gain_options,
    hand_compare_result_card, kongsheng_pile_cards, mbkuangxiang_exhausted_role,
    mbshanjia_discard_category, move_target_equipment_to_player_effects, qizhi_count,
    queue_end_extra_phases, queue_extra_draw_phase, set_potfaen_pending, skill_instance_by_def,
    start_end_extra_phases, twyuyan_gift_cards, twzhibian_movable_cards, twzhibian_move_effects,
};
use source_skill::effect_source_skill_of;

pub(crate) use primitives::apply_twchayi_initial;
pub(crate) use primitives::card_category_of;
pub(crate) use primitives::commit_twguoyi_choice;
pub(crate) use primitives::finish_twyizhu_used_cards;
pub(crate) use primitives::finish_twyuejian_cards;
pub(crate) use primitives::open_potweiyan_hp_choice;
pub(crate) use primitives::player_target_matches;
pub(crate) use primitives::pothaoshi_refill_effects;
pub(crate) use primitives::{
    add_mark, arm_potwanglie, attack_range_population, bound_damage_matches,
    bound_damage_source_matches, bound_sha_damage_matches, bound_sha_damage_matches_any_segment,
    commit_pothongyi_prepare, continue_beige_judgment_rewriters,
    continue_beige_judgment_rewriters_configured, current_attack_range,
    damage_endpoint_discard_cards, distance_hand_discard_count, draw_cards,
    ganglie_discardable_cards, increase_bound_damage, increase_bound_damage_by_one, move_cards,
    open_attack_range_discard_choice, open_ganglie_target_discard_or_damage,
    open_gongjian_discard_choice, open_judgment_damage_other_choice,
    open_take_larger_hand_card_choice, open_xiaoguo_target_equipment_or_damage,
    physical_equipped_weapons, physical_weapon_range, reduce_bound_damage_by_one,
    resolve_clanshenjun_use_choice, resolve_clanyuzhi_end_choice, resolve_clanyuzhi_reveal_choice,
    resolve_jianzhi_assign_card, resolve_jianzhi_count_choice, resolve_qieyi_put_top_choice,
    resolve_target_discard_duel, standard_ganglie_discardable_hand_cards,
    target_discard_cards_share_category,
};
pub use primitives::{
    apply_effect, bootstrap_match, move_one_card, open_choice, reshuffle_discard_into_draw,
    RESOLVE_GZBUQU_RECOVER_CARD, RESOLVE_TWJUXIANG_CARD,
};
pub(crate) use primitives::{
    begin_pottaishici_response_cost, dcsbmengmou_tracks_sha, dcsbyingmou_tracks_card,
    pottaishici_response_cost_armed, resolve_caoyi_choice, resolve_dc_sb_lusu_choice,
    resolve_dc_sb_zhangxiu_choice, resolve_dc_sb_zhouyu_choice, resolve_ddd_baosanniang_choice,
    resolve_ddd_caomao_choice, resolve_ddd_caoshuang_choice, resolve_ddd_duanbing_choice,
    resolve_ddd_guanning_choice, resolve_ddd_jianshuo_choice, resolve_ddd_liangxi_choice,
    resolve_ddd_liuba_choice, resolve_ddd_liuchong_choice, resolve_ddd_liuhong_choice,
    resolve_ddd_liuye_choice, resolve_ddd_wuzhi_choice, resolve_ddd_xiaheng_choice,
    resolve_ddd_xiahouxuan_choice, resolve_ddd_xianglang_choice, resolve_ddd_xianxi_choice,
    resolve_ddd_xinxianying_choice, resolve_ddd_xuelingyun_choice, resolve_ddd_xujing_choice,
    resolve_ddd_yujin_choice, resolve_ddd_zhangkai_choice, resolve_ddd_zhaoang_choice,
    resolve_ddd_zhenji_choice, resolve_gzbuqu_recover_commit, resolve_malingli_choice,
    resolve_nsshuaiyan_choice, resolve_pottaishici_choice, resolve_wdb_fuchou_choice,
    resolve_wdb_liufu_choice, resolve_wdb_liuzan_choice, resolve_wdb_tianyu_choice,
    resolve_wdb_yangyi_choice, resolve_wdb_zaozhirenjun_choice, resolve_zhangjian_choice,
    resolve_zhugeruoxue_choice, POTTAISHICI_CUSTOM_CHOICE, RESOLVE_DDD_BAOSANNIANG_CHOICE,
    RESOLVE_DDD_CAOSHUANG_CHOICE, RESOLVE_DDD_GUANNING_CHOICE, RESOLVE_DDD_JIANSHUO_CHOICE,
    RESOLVE_DDD_LIANGXI_CHOICE, RESOLVE_DDD_LIUBA_CHOICE, RESOLVE_DDD_LIUHONG_CHOICE,
    RESOLVE_DDD_LIUYE_CHOICE, RESOLVE_DDD_XIAHOUXUAN_CHOICE, RESOLVE_DDD_XIANGLANG_CHOICE,
    RESOLVE_DDD_XUELINGYUN_CHOICE, RESOLVE_DDD_YUJIN_CHOICE, RESOLVE_DDD_ZHANGKAI_CHOICE,
    RESOLVE_DDD_ZHAOANG_CHOICE, RESOLVE_DDD_ZHENJI_CHOICE,
};
pub(crate) use primitives::{
    bianyue_wuxie_targets, cilv_tracks_card, cilv_used_mask, dcchiying_gain_basics,
    dcchiying_in_range, dcctjiuxian_recovery_targets, discard_phase_discarded_cards_for,
    discard_phase_discards_for, ensure_skill_pile_zone, move_cards_with_reason,
    resolve_guying_commit, resolve_potcaoshuang_choice, resolve_potlvyi_choice,
    resolve_potzhouyu_choice, resolve_rechuhai_choice, skill_pile_def, twchengpu_field_cards,
    twchengpu_hand_equip_cards, CATEGORY_GIFT_DAMAGE_PLUS_ONE_MODE, CHILING_CUSTOM_CHOICE,
    CHOOSE_DISCARD_HAND_RANGE_CHOICE, CHOOSE_OTHER_GAIN_ONE_OF_TWO_SKILLS_CUSTOM_CHOICE,
    CHOOSE_TARGET_DRAW_MAX_HP_CAP_THEN_ADD_SKILL_CUSTOM_CHOICE, DCKUIZHEN_PENDING_MARK,
    DCMANZHI_GAIN_MARK, DCMANZHI_GIVE_MARK, DCMOYU_ADD_MARK, DCXIANG_SHU_NUMBER_MARK,
    DISCARD_HAND_EQUIP_COUNT_CUSTOM_CHOICE, GUGUOANBANG_CUSTOM_CHOICE, GZJINFA_CUSTOM_CHOICE,
    HAOLINGTIANXIA_CUSTOM_CHOICE, JIEBING_CUSTOM_CHOICE, KEFUZHONGYUAN_CUSTOM_CHOICE,
    LIANJUNSHENGYAN_CUSTOM_CHOICE, MBCHENSHE_CUSTOM_CHOICE, MBCHIZHANG_CUSTOM_CHOICE,
    MBDUANYANG_CUSTOM_CHOICE, MBFEIJING_CUSTOM_CHOICE, MBJIANJI_CUSTOM_CHOICE,
    MBQINGDAO_CUSTOM_CHOICE, MBRENXING_CUSTOM_CHOICE, MBRUNWEI_CUSTOM_CHOICE,
    MBSHUANGHUAI_CUSTOM_CHOICE, MBSIJIAN_CUSTOM_CHOICE, MBXIONGTU_CUSTOM_CHOICE,
    MBXIUGENG_CUSTOM_CHOICE, MBXUYE_CUSTOM_CHOICE, MBYUANMO_CUSTOM_CHOICE,
    MBZHONGHUI_CUSTOM_CHOICE, MBZHUJI_CUSTOM_CHOICE, MOBILEDANSHOU_CUSTOM_CHOICE,
    POTCAOSHUANG_CUSTOM_CHOICE, POTCHENDAO_CUSTOM_CHOICE, POTCHENGPU_CUSTOM_CHOICE,
    POTCHENQUN_CUSTOM_CHOICE, POTDIMENG_CUSTOM_CHOICE, POTFUAN_CUSTOM_CHOICE,
    POTHAOSHI_CUSTOM_CHOICE, POTHUANJIE_CUSTOM_CHOICE, POTJIEJIE_CUSTOM_CHOICE,
    POTJIXI_CUSTOM_CHOICE, POTJUGU_CUSTOM_CHOICE, POTLULIAN_CUSTOM_CHOICE, POTLVYI_CUSTOM_CHOICE,
    POTNIGU_CUSTOM_CHOICE, POTPOXI_CUSTOM_CHOICE, POTQINGSHI_CUSTOM_CHOICE,
    POTSIFENG_CUSTOM_CHOICE, POTTUNTIAN_CUSTOM_CHOICE, POTWEIYAN_CUSTOM_CHOICE,
    POTZHOUYU_CUSTOM_CHOICE, RANDOM_GAIN_REGION_COMPARE_COLOR_CUSTOM_CHOICE, REFUHAI_CUSTOM_CHOICE,
    REKUANSHI_WATCH_MARK, RESOLVE_BAILINGYUN_CHOICE, RESOLVE_BIANYUE_CHOICE, RESOLVE_CAOYI_CHOICE,
    RESOLVE_CHENGLVE_DISCARD, RESOLVE_DBQUEDI_CHOICE, RESOLVE_DBZHUIFENG_TARGET,
    RESOLVE_DCCHIYING_CUSTOM_CHOICE, RESOLVE_DCCTJIUXIAN_CHOICE, RESOLVE_DCDICHOU_CHOICE,
    RESOLVE_DCDUFENG_CHOICE, RESOLVE_DCJUE_CHOICE, RESOLVE_DCKUIZHEN_CUSTOM_CHOICE,
    RESOLVE_DCMANZHI_CUSTOM_CHOICE, RESOLVE_DCMOYU_CUSTOM_CHOICE, RESOLVE_DCNUCHEN_CUSTOM_CHOICE,
    RESOLVE_DCTINGXIAN_CUSTOM_CHOICE, RESOLVE_DCXIANG_SHU_CUSTOM_CHOICE,
    RESOLVE_DCYINGTU_CUSTOM_CHOICE, RESOLVE_DCZHIHENG_CUSTOM_CHOICE, RESOLVE_DC_SB_LUSU_CHOICE,
    RESOLVE_DC_SB_ZHANGXIU_CHOICE, RESOLVE_DC_SB_ZHOUYU_CHOICE, RESOLVE_DIMENG_CUSTOM_CHOICE,
    RESOLVE_DRLT_CONGJIAN_CUSTOM_CHOICE, RESOLVE_DRLT_WEIDI_CUSTOM_CHOICE, RESOLVE_DUANBI_CHOICE,
    RESOLVE_EJIAN_CHOICE, RESOLVE_FANGQUAN_DISCARD, RESOLVE_FANGQUAN_TARGET,
    RESOLVE_GONGJIAN_CUSTOM_CHOICE, RESOLVE_GUYING_CHOICE, RESOLVE_GUZHENG_CUSTOM_CHOICE,
    RESOLVE_HAOSHI_CUSTOM_CHOICE, RESOLVE_HEJI_CHOICE, RESOLVE_HFJIEYING_CUSTOM_CHOICE,
    RESOLVE_HONGJU_CUSTOM_CHOICE, RESOLVE_HOUFENG_CHOICE, RESOLVE_HUANGTIAN_CUSTOM_CHOICE,
    RESOLVE_HUZUN_CHOICE, RESOLVE_JIANXIANG_TARGET, RESOLVE_JIELIANG_GAIN_CUSTOM_CHOICE,
    RESOLVE_JINQU_DISCARD, RESOLVE_JSRGCHIYING_CUSTOM_CHOICE, RESOLVE_JUZHAN_GAIN,
    RESOLVE_KONGSHENG_PLACE, RESOLVE_KONGSHENG_TARGET, RESOLVE_KUIZHU_DAMAGE, RESOLVE_KUIZHU_DRAW,
    RESOLVE_LIANGYIN_COMMIT, RESOLVE_LIANGYIN_DISCARD, RESOLVE_LIANGYIN_DRAW,
    RESOLVE_LIANGYIN_RECOVER, RESOLVE_LIANGYIN_TARGET, RESOLVE_LIANZHOU_CUSTOM_CHOICE,
    RESOLVE_LULVE_CUSTOM_CHOICE, RESOLVE_MALINGLI_CHOICE, RESOLVE_MINGREN_PLACE,
    RESOLVE_MINGREN_REPLACE, RESOLVE_NSDINGZHOU_CUSTOM_CHOICE, RESOLVE_NSSHUAIYAN_CHOICE,
    RESOLVE_NZRY_FEIJUN_CUSTOM_CHOICE, RESOLVE_QIAOBIAN_DISCARD, RESOLVE_QIAOBIAN_DRAW_PICK,
    RESOLVE_QIAOBIAN_DRAW_TARGETS, RESOLVE_QIAOBIAN_PLAY_CARD, RESOLVE_QIAOBIAN_PLAY_TARGET,
    RESOLVE_QIAOJIAN_CHOICE, RESOLVE_QINGJIAN_CUSTOM_CHOICE, RESOLVE_QINGYU_CUSTOM_CHOICE,
    RESOLVE_QINGYU_FAIL_TARGET, RESOLVE_QIZHI_CARD, RESOLVE_QIZHI_TARGET, RESOLVE_RECHUHAI_CHOICE,
    RESOLVE_REKUANSHI_CUSTOM_CHOICE, RESOLVE_RELEIJI_CUSTOM_CHOICE,
    RESOLVE_RETIANXIANG_CUSTOM_CHOICE, RESOLVE_RUANJI_CHOICE, RESOLVE_SHENSHI_GIVE,
    RESOLVE_SHENSHI_REWARD, RESOLVE_SHENSHI_TARGET, RESOLVE_SHEYI_CHOICE, RESOLVE_SHICAI_PLACE,
    RESOLVE_SIBIAN_CUSTOM_CHOICE, RESOLVE_SPFENGYIN_CUSTOM_CHOICE, RESOLVE_SPJIEDAO_CUSTOM_CHOICE,
    RESOLVE_SPQIAI_CHOICE, RESOLVE_SPSHANGYI_CHOICE, RESOLVE_SPSHANXI_CHOICE,
    RESOLVE_SPYAJUN_CUSTOM_CHOICE, RESOLVE_SPYINJU_CHOICE, RESOLVE_SPZUNDI_CUSTOM_CHOICE,
    RESOLVE_STDDECHONG_CUSTOM_CHOICE, RESOLVE_STDGAOBIAN_CUSTOM_CHOICE,
    RESOLVE_STDJUJIAN_CUSTOM_CHOICE, RESOLVE_SUOMING_CHOICE, RESOLVE_SXRMLANJIAO_CUSTOM_CHOICE,
    RESOLVE_TIANXIANG_CUSTOM_CHOICE, RESOLVE_TIAOXIN_CUSTOM_CHOICE, RESOLVE_TINGWEI_OPTION,
    RESOLVE_TINGWEI_SELECT, RESOLVE_TONGDUO_CHOICE, RESOLVE_WDB_LIUFU_CHOICE,
    RESOLVE_WDB_LIUZAN_CHOICE, RESOLVE_XIANGLE_CUSTOM_CHOICE, RESOLVE_XICHA_CHOICE,
    RESOLVE_XINFU_LIANPIAN_CUSTOM_CHOICE, RESOLVE_XINFU_QINGUO_CUSTOM_CHOICE,
    RESOLVE_XINKUANGFU_CUSTOM_CHOICE, RESOLVE_XINMINGSHI_CUSTOM_CHOICE,
    RESOLVE_XINXINGLUAN_CUSTOM_CHOICE, RESOLVE_XINYANGJIE_CUSTOM_CHOICE, RESOLVE_XIONGSHU_CHOICE,
    RESOLVE_XIZHEN_CUSTOM_CHOICE, RESOLVE_XUNYI_SELECT, RESOLVE_XUXIE_CUSTOM_CHOICE,
    RESOLVE_ZAOLI_CHOICE, RESOLVE_ZHANGJIAN_CHOICE, RESOLVE_ZHENGBI_CHOICE,
    RESOLVE_ZHENGRONG_CUSTOM_CHOICE, RESOLVE_ZHENGSU_CHOICE, RESOLVE_ZHENLIANG_DAMAGE,
    RESOLVE_ZHENLIANG_DISCARD, RESOLVE_ZHENLIANG_DRAW, RESOLVE_ZHIBA_CUSTOM_CHOICE,
    RESOLVE_ZHIJIAN_CUSTOM_CHOICE, RESOLVE_ZHIZHENG_CARD, RESOLVE_ZHUGEMENGXUE_CHOICE,
    RESOLVE_ZHUGERUOXUE_CHOICE, RESOLVE_ZHUPEILAN_CHOICE, RESOLVE_ZUILUN_PICK,
    RESOLVE_ZUILUN_TARGET, REVEAL_CATEGORY_DISTRIBUTION_CUSTOM_CHOICE, SHUANGXIONG_COLOR_MARK,
    SHUIYANQIJUNX_CUSTOM_CHOICE, SPMIAOLVE_CUSTOM_CHOICE, SPYINGJIA_CUSTOM_CHOICE,
    STDANRAN_COUNTER_MARK, TWCAOANG_CUSTOM_CHOICE, TWCHENGPU_CUSTOM_CHOICE,
    TWCHENXUN_CUSTOM_CHOICE, TWFENXIAN_CUSTOM_CHOICE, TWMOUKUI_CUSTOM_CHOICE,
    TWNIUJIN_CUSTOM_CHOICE, WENHELUANWU_CUSTOM_CHOICE, XIONGSHU_MARK,
    YANGWU_GAIN_HAND_CUSTOM_CHOICE, YUANLVE_CUSTOM_CHOICE,
};
pub(crate) use primitives::{complete_potceduan, POTCEDUAN_CUSTOM_CHOICE};
pub(crate) use primitives::{ddd_shijian_watched, ddd_xuyu_pending};
pub(crate) use primitives::{
    DDD_DONGCHA_DEF, RESOLVE_DDD_CAOMAO_CHOICE, RESOLVE_DDD_DUANBING_CHOICE,
    RESOLVE_DDD_LIUCHONG_CHOICE, RESOLVE_DDD_WUZHI_CHOICE, RESOLVE_DDD_XIAHENG_CHOICE,
    RESOLVE_DDD_XIANXI_CHOICE, RESOLVE_DDD_XINXIANYING_CHOICE, RESOLVE_DDD_XUJING_CHOICE,
    RESOLVE_WDB_FUCHOU_CHOICE, RESOLVE_WDB_TIANYU_CHOICE, RESOLVE_WDB_YANGYI_CHOICE,
    RESOLVE_WDB_ZAOZHIRENJUN_CHOICE,
};

use crate::state::{CardKind, ContinuationKind};
use crate::transaction::Transaction;
use engine_api::{DomainEvent, FaultKind, PlayerId};
use rule_api::EffectRequest;
use std::collections::VecDeque;
use thiserror::Error;
mod damage_prep;
use damage_prep::*;
mod continuations;
pub(crate) use continuations::*;

/// Effect application error.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum EffectError {
    /// Illegal arguments / state for effect.
    #[error("illegal effect: {0}")]
    Illegal(String),
    /// Budget exceeded.
    #[error("effect budget exceeded")]
    BudgetExceeded,
    /// Choice requested — pause segment (successful barrier).
    #[error("choice requested")]
    ChoiceRequested,
    /// Content / VM style fault.
    #[error("content fault: {0}")]
    ContentFault(String),
    /// Engine invariant would break.
    #[error("engine fault: {0}")]
    EngineFault(String),
}

impl EffectError {
    /// Map to public fault kind when aborting.
    pub fn fault_kind(&self) -> FaultKind {
        match self {
            EffectError::BudgetExceeded => FaultKind::BudgetExceeded,
            EffectError::EngineFault(_) => FaultKind::EngineFault,
            EffectError::ContentFault(_) => FaultKind::ContentFault,
            EffectError::Illegal(_) => FaultKind::ContentFault,
            EffectError::ChoiceRequested => FaultKind::ContentFault,
        }
    }
}

/// Explicit FIFO effect scheduler with follow-up enqueue.
#[derive(Debug, Default)]
pub struct EffectScheduler {
    queue: VecDeque<EffectRequest>,
}

impl EffectScheduler {
    /// Empty scheduler.
    pub fn new() -> Self {
        Self::default()
    }

    /// Enqueue at back (stable order).
    pub fn push_back(&mut self, req: EffectRequest) {
        self.queue.push_back(req);
    }

    /// Enqueue at front (used only for interrupt timing).
    pub fn push_front(&mut self, req: EffectRequest) {
        self.queue.push_front(req);
    }

    /// Enqueue many in order.
    pub fn extend<I: IntoIterator<Item = EffectRequest>>(&mut self, iter: I) {
        self.queue.extend(iter);
    }

    /// Pop front.
    pub fn pop_front(&mut self) -> Option<EffectRequest> {
        self.queue.pop_front()
    }

    /// Remaining length.
    pub fn len(&self) -> usize {
        self.queue.len()
    }

    /// Empty?
    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    fn drain_remaining(&mut self) -> Vec<EffectRequest> {
        self.queue.drain(..).collect()
    }
}

/// Apply a queue of effects (FIFO) until empty, choice barrier, or error.
///
/// After damage that reduces HP to ≤0, automatically enqueues dying + save choice
/// (Phase 2 simplified peach window — system save option).
/// After each effect's domain events, Phase 3 trigger dispatch may enqueue more effects.
pub fn run_effects(
    tx: &mut Transaction,
    initial: Vec<EffectRequest>,
) -> Result<Vec<DomainEvent>, EffectError> {
    use crate::trigger::TriggerFireRecord;

    // Nested card pipelines invoked by a skill effect retain that skill's provenance so the
    // resulting CardUsed/CardTargeted events can suppress exact-skill recursion.
    let inherited_effect_source_skill = tx.state().resolution.effect_source_skill;

    // A nested effect queue (for example, moving a judgment card) must not
    // consume a damage tail suspended by its caller. The caller that entered
    // with no pre-existing tail remains responsible for draining tails it
    // creates while dispatching damage-before triggers.
    let owns_deferred_tail = tx.state().resolution.deferred_tail_effects.is_empty();
    let mut sched = EffectScheduler::new();
    sched.extend(initial);
    let mut produced = Vec::new();
    let mut fires = TriggerFireRecord::new();

    while let Some(mut req) = sched.pop_front() {
        let aborts_resolution = matches!(
            req,
            EffectRequest::FinishCurrentPhaseAbortResolution { stage: 0, .. }
        );
        if aborts_resolution {
            sched.drain_remaining();
            let resolution = &mut tx.state_mut().resolution;
            resolution.clear_choice();
            resolution.current_damage_id = 0;
            resolution.current_recovery_id = 0;
            resolution.damage_redirect_in_progress = false;
            resolution.prevented_damage_ids.clear();
            resolution.prevented_recovery_ids.clear();
            resolution.current_hand_compare_player = 0;
            resolution.current_hand_compare_target = 0;
            resolution.current_hand_compare_random_mask = 0;
            resolution.resume_kind = ContinuationKind::None;
            resolution.resume_payload.clear();
            resolution.sha_no_damage_draw_skills.clear();
            resolution.sha_resolution_records.clear();
            resolution.pending_card_replays.clear();
            resolution.deferred_effects.clear();
            resolution.deferred_tail_effects.clear();
            resolution.deferred_triggers.clear();
        }
        if let EffectRequest::RecoverHp {
            source,
            player,
            amount,
        } = req
        {
            let recoverable = tx.state().players.get(player).is_some_and(|target| {
                target.alive && amount > 0 && target.hp < target.max_hp as i32
            });
            if !recoverable {
                continue;
            }
            let context = crate::trigger::TriggerContext {
                kind: crate::trigger::EventKind::RecoverBefore,
                player,
                source,
                respond_to: PlayerId::NONE,
                amount,
                nature: 0,
                turn_damage_sources: 0,
                turn_damaged_players: 0,
                move_reason: engine_api::CardMoveReason::Unspecified,
                moved_has_basic: false,
                from_zone: None,
                to_zone: None,
                card: None,
                card_kind: None,
                card_flags: 0,
                target_count: 0,
                targets: Vec::new(),
                damaged_targets: Vec::new(),
                cards: Vec::new(),
                added_skill_def: 0,
                debate_red: Vec::new(),
                debate_black: Vec::new(),
            };
            let pending = crate::trigger::plan_triggers(tx, context, &mut fires)?;
            let mut prepared = EffectRequest::CommitRecovery {
                recovery_id: 0,
                source,
                player,
                amount,
            };
            if !pending.is_empty() {
                let recovery_id = allocate_recovery_id(tx);
                if let EffectRequest::CommitRecovery {
                    recovery_id: prepared_id,
                    ..
                } = &mut prepared
                {
                    *prepared_id = recovery_id;
                }
                tx.state_mut().resolution.current_recovery_id = recovery_id;
                match crate::trigger::run_pending_triggers(tx, pending) {
                    Ok(effects) => {
                        defer_damage_tail(tx, prepared, &mut sched);
                        sched.extend(effects);
                        continue;
                    }
                    Err(EffectError::ChoiceRequested) => {
                        defer_damage_tail(tx, prepared, &mut sched);
                        return Err(EffectError::ChoiceRequested);
                    }
                    Err(error) => return Err(error),
                }
            }
            req = prepared;
        }
        if let Some(mut prepared) = prepare_damage_request(tx, &req)? {
            let (source, target, amount, is_lose_hp, nature, card, card_kind, card_flags) =
                match &prepared {
                    EffectRequest::CommitDamage {
                        source,
                        target,
                        amount,
                        is_lose_hp,
                        nature,
                        card,
                        card_kind,
                        card_flags,
                        ..
                    } => (
                        *source,
                        *target,
                        *amount,
                        *is_lose_hp,
                        *nature,
                        *card,
                        *card_kind,
                        *card_flags,
                    ),
                    EffectRequest::CommitDamageBanActiveIfDying {
                        source,
                        target,
                        amount,
                        nature,
                        ..
                    } => (
                        *source,
                        *target,
                        *amount,
                        false,
                        *nature,
                        engine_api::CardId::NONE,
                        0,
                        0,
                    ),
                    EffectRequest::CommitLoseHpThenDamageOtherLoseMaxHpIfDying {
                        player,
                        amount,
                        ..
                    } => (
                        PlayerId::NONE,
                        *player,
                        *amount,
                        true,
                        rule_api::damage_nature::NORMAL,
                        engine_api::CardId::NONE,
                        0,
                        0,
                    ),
                    _ => unreachable!(),
                };
            if !is_lose_hp && amount > 0 {
                let context = crate::trigger::TriggerContext {
                    kind: crate::trigger::EventKind::DamageBefore,
                    player: target,
                    source,
                    respond_to: PlayerId::NONE,
                    amount,
                    nature,
                    turn_damage_sources: 0,
                    turn_damaged_players: 0,
                    move_reason: engine_api::CardMoveReason::Unspecified,
                    moved_has_basic: false,
                    from_zone: None,
                    to_zone: None,
                    card: card.is_some().then_some(card),
                    card_kind: (card_kind != 0).then_some(card_kind),
                    card_flags,
                    target_count: 0,
                    targets: Vec::new(),
                    damaged_targets: Vec::new(),
                    cards: Vec::new(),
                    added_skill_def: 0,
                    debate_red: Vec::new(),
                    debate_black: Vec::new(),
                };
                let pending = crate::trigger::plan_triggers(tx, context, &mut fires)?;
                if !pending.is_empty() {
                    let damage_id = allocate_damage_id(tx);
                    match &mut prepared {
                        EffectRequest::CommitDamage {
                            damage_id: prepared_id,
                            ..
                        }
                        | EffectRequest::CommitDamageBanActiveIfDying {
                            damage_id: prepared_id,
                            ..
                        } => *prepared_id = damage_id,
                        _ => unreachable!(),
                    }
                    tx.state_mut().resolution.current_damage_id = damage_id;
                    match crate::trigger::run_pending_triggers(tx, pending) {
                        Ok(effects) => {
                            defer_damage_tail(tx, prepared, &mut sched);
                            sched.extend(effects);
                            continue;
                        }
                        Err(EffectError::ChoiceRequested) => {
                            defer_damage_tail(tx, prepared, &mut sched);
                            return Err(EffectError::ChoiceRequested);
                        }
                        Err(error) => return Err(error),
                    }
                }
            }
            req = prepared;
        } else if matches!(
            req,
            EffectRequest::DealDamage { .. }
                | EffectRequest::DealShaDamage { .. }
                | EffectRequest::DealCardDamage { .. }
                | EffectRequest::DealDamageBanActiveIfDying { .. }
                | EffectRequest::LoseHpThenDamageOtherLoseMaxHpIfDying { .. }
        ) {
            if tx.state().resolution.effect_budget == 0 {
                return Err(EffectError::BudgetExceeded);
            }
            tx.state_mut().resolution.effect_budget -= 1;
            continue;
        }
        if tx.state().resolution.effect_budget == 0 {
            return Err(EffectError::BudgetExceeded);
        }
        tx.state_mut().resolution.effect_budget -= 1;

        if let EffectRequest::CommitDamage {
            target,
            is_lose_hp,
            card_flags,
            ..
        } = &mut req
        {
            let propagated = *card_flags & rule_api::damage_card_flag::CHAIN_PROPAGATED != 0;
            let linked = tx.state().players.get(*target).is_some_and(|player| {
                player
                    .marks
                    .iter()
                    .any(|(mark, count)| *mark == crate::content::MARK_CHAIN && *count > 0)
            });
            if !*is_lose_hp && (linked || propagated) {
                *card_flags |= rule_api::damage_card_flag::TARGET_WAS_LINKED;
            }
        }

        // Snapshot life state for dying chain detection
        let pre_dying = match &req {
            EffectRequest::DealDamage { target, .. }
            | EffectRequest::DealShaDamage { target, .. }
            | EffectRequest::DealCardDamage { target, .. }
            | EffectRequest::DealDamageBanActiveIfDying { target, .. }
            | EffectRequest::CommitDamage { target, .. }
            | EffectRequest::CommitDamageBanActiveIfDying { target, .. } => tx
                .state()
                .players
                .get(*target)
                .map(|p| (p.hp, p.dying, p.alive)),
            EffectRequest::LoseHpThenDamageOtherLoseMaxHpIfDying { player, .. }
            | EffectRequest::CommitLoseHpThenDamageOtherLoseMaxHpIfDying { player, .. } => tx
                .state()
                .players
                .get(*player)
                .map(|p| (p.hp, p.dying, p.alive)),
            _ => None,
        };

        tx.state_mut().resolution.effect_source_skill =
            effect_source_skill_of(&req, inherited_effect_source_skill);
        let applied = match &req {
            EffectRequest::CommitDamage {
                damage_id,
                source,
                target,
                amount,
                base_amount,
                is_lose_hp,
                nature,
                card,
                card_kind,
                card_flags,
            } => primitives::commit_damage(
                tx,
                *damage_id,
                *source,
                *target,
                *amount,
                *base_amount,
                *is_lose_hp,
                *nature,
                *card,
                *card_kind,
                *card_flags,
            ),
            EffectRequest::CommitDamageBanActiveIfDying {
                damage_id,
                source,
                target,
                amount,
                base_amount,
                nature,
                skill,
            } => primitives::commit_damage_ban_active_if_dying(
                tx,
                *damage_id,
                *source,
                *target,
                *amount,
                *base_amount,
                *nature,
                *skill,
            ),
            EffectRequest::CommitLoseHpThenDamageOtherLoseMaxHpIfDying {
                player,
                target,
                amount,
                skill,
            } => primitives::commit_lose_hp_then_damage_other_lose_max_hp_if_dying(
                tx, *player, *target, *amount, *skill,
            ),
            EffectRequest::ChooseCompareSelectedHandCard {
                player,
                target,
                card,
                win_skill_def,
                lose_skill_def,
                skill,
            } => primitives::begin_choose_compare_selected_hand_card(
                tx,
                *player,
                *target,
                *card,
                *win_skill_def,
                *lose_skill_def,
                *skill,
            ),
            EffectRequest::ChooseCompareSelectedHandCardSecondFailureSeatDamage {
                player,
                target,
                card,
                draw_count,
                failure_count,
                lose_hp,
                damage,
                skill,
            } => primitives::begin_choose_compare_selected_hand_card_second_failure_seat_damage(
                tx,
                *player,
                *target,
                *card,
                *draw_count,
                *failure_count,
                *lose_hp,
                *damage,
                *skill,
            ),
            EffectRequest::RecordCompareFailureSeatDamage {
                player,
                target,
                failure_count,
                lose_hp,
                damage,
                skill,
            } => primitives::record_compare_failure_seat_damage(
                tx,
                *player,
                *target,
                *failure_count,
                *lose_hp,
                *damage,
                *skill,
            )
            .map(|followups| (Vec::new(), followups)),
            EffectRequest::DealDamageEachInOrder {
                source,
                targets,
                amount,
                skill,
            } => primitives::continue_damage_each_in_order(
                tx,
                *source,
                targets.clone(),
                *amount,
                *skill,
            )
            .map(|followups| (Vec::new(), followups)),
            _ => apply_effect(tx, req.clone()).map(|events| (events, Vec::new())),
        };
        let (events, followups) = match applied {
            Ok(applied) => applied,
            Err(error) => {
                tx.state_mut().resolution.effect_source_skill = inherited_effect_source_skill;
                return Err(error);
            }
        };
        for e in &events {
            tx.push_event(e.clone());
            produced.push(e.clone());
        }
        for followup in followups.into_iter().rev() {
            sched.push_front(followup);
        }

        // Damage → dying chain (noname: player.dying after hp<=0)
        if let (
            EffectRequest::DealDamage { source, target, .. }
            | EffectRequest::DealShaDamage { source, target, .. }
            | EffectRequest::DealCardDamage { source, target, .. }
            | EffectRequest::DealDamageBanActiveIfDying { source, target, .. }
            | EffectRequest::CommitDamage { source, target, .. }
            | EffectRequest::CommitDamageBanActiveIfDying { source, target, .. },
            Some((_prev_hp, prev_dying, prev_alive)),
        ) = (&req, pre_dying)
        {
            if prev_alive {
                if let Some((hp, dying)) = tx
                    .state()
                    .players
                    .get(*target)
                    .map(|participant| (participant.hp, participant.dying))
                {
                    if hp <= 0 && !prev_dying {
                        // 旧不屈：本次体力流失免死标记（创牌点数均不同）在伤害链
                        // 结算前由 gzbuqu 触发置位，此处消费并跳过濒死。
                        if tx.state().resolution.gzbuqu_nodying {
                            tx.state_mut().resolution.gzbuqu_nodying = false;
                            continue;
                        }
                        tx.state_mut().turn.record_dying_caused(*source, *target);
                        if let EffectRequest::CommitDamage {
                            card, card_kind, ..
                        } = &req
                        {
                            let skills: Vec<_> = tx
                                .state()
                                .skills
                                .sorted_live()
                                .into_iter()
                                .filter(|skill| {
                                    skill.owner == *source
                                        && skill.twlihuo_dying_cost_card() == Some(*card)
                                })
                                .map(|skill| skill.id)
                                .collect();
                            for skill in skills {
                                tx.state_mut()
                                    .skills
                                    .get_mut(skill)
                                    .unwrap()
                                    .mark_twlihuo_caused_dying();
                            }
                            if CardKind::from_tag(*card_kind).is_some_and(CardKind::is_sha) {
                                tx.state_mut()
                                    .resolution
                                    .mark_sha_target_entered_dying(*source, *card, *target);
                            }
                        }
                        // deal_damage usually already set dying + EnteredDying event
                        if !dying {
                            sched.push_back(EffectRequest::EnterDying { player: *target });
                        }
                        // Deterministic multi-rescuer save queue: Tao from all legal responders,
                        // plus Jiu for the dying player themselves.
                        sched.push_back(EffectRequest::RequestDyingSave {
                            player: *target,
                            source: *source,
                        });
                        let r = &mut tx.state_mut().resolution;
                        // Preserve AOE / juedou queue across dying interrupt.
                        // Progress (idx bump) is owned by advance_aoe_queue / resolve paths
                        // so resume never re-hits a target whose damage already applied.
                        if matches!(
                            r.continuation_kind,
                            ContinuationKind::RespondAoeQueue | ContinuationKind::JuedouDuel
                        ) {
                            r.resume_kind = r.continuation_kind;
                            r.resume_payload = r.continuation_payload.clone();
                        }
                    }
                }
            }
        }
        if let (
            EffectRequest::LoseHpThenDamageOtherLoseMaxHpIfDying { player, .. }
            | EffectRequest::CommitLoseHpThenDamageOtherLoseMaxHpIfDying { player, .. },
            Some((_prev_hp, prev_dying, prev_alive)),
        ) = (&req, pre_dying)
        {
            if prev_alive {
                if let Some(participant) = tx.state().players.get(*player) {
                    if participant.hp <= 0 && !prev_dying {
                        if !participant.dying {
                            sched.push_back(EffectRequest::EnterDying { player: *player });
                        }
                        sched.push_back(EffectRequest::RequestDyingSave {
                            player: *player,
                            source: PlayerId::NONE,
                        });
                    }
                }
            }
        }

        // Phase 3: stable-order trigger dispatch → more effects
        // Skip re-entry on pure trigger marker notes to reduce noise loops:
        // still allow DrawSelf etc. EmitRuleEvent code 100 is marker only.
        // 连营/枭姬等走 DSL 过滤器（owner_from_hand_empty / owner_from_equip）。
        let damage_context = match &req {
            EffectRequest::CommitDamage {
                card,
                card_kind,
                card_flags,
                ..
            } => Some((*card, *card_kind, *card_flags)),
            _ => None,
        };
        for player in events.iter().filter_map(|event| match event {
            DomainEvent::EnteredDying { player, .. } => Some(*player),
            _ => None,
        }) {
            if tx.state().content.tracks_entered_dying_history() {
                tx.state_mut().turn.record_entered_dying(player);
            }
            tx.state_mut().lifetime_dying_players.insert(player);
            tx.state_mut().lifetime_dying_events += 1;
        }
        let trigger_result = crate::trigger::dispatch_for_events_with_damage_context(
            tx,
            &events,
            &mut fires,
            damage_context,
        );
        let interrupt_triggers = events.iter().any(|event| match event {
            DomainEvent::EnteredDying { .. }
            | DomainEvent::TurnBegan { .. }
            | DomainEvent::HandCompareStarted { .. }
            | DomainEvent::HandCompareFinished { .. } => true,
            DomainEvent::CardUsed { .. } => true,
            DomainEvent::CardResponded { respond_to, .. } => tx
                .state()
                .players
                .get(*respond_to)
                .is_some_and(|player| player.dying && player.hp <= 0),
            _ => false,
        });
        tx.state_mut().resolution.effect_source_skill = inherited_effect_source_skill;
        let trigger_effects = match trigger_result {
            Ok(effects) => effects,
            Err(EffectError::ChoiceRequested) => {
                if aborts_resolution {
                    if let EffectRequest::FinishCurrentPhaseAbortResolution { player, .. } = req {
                        sched.push_back(EffectRequest::FinishCurrentPhaseAbortResolution {
                            player,
                            stage: 1,
                        });
                    }
                }
                if matches!(
                    tx.state().resolution.continuation_kind,
                    crate::state::ContinuationKind::OptionalTrigger
                ) && interrupt_triggers
                {
                    defer_scheduler_tail(tx, &mut sched);
                } else {
                    defer_scheduler_front(tx, &mut sched);
                }
                return Err(EffectError::ChoiceRequested);
            }
            Err(error) => return Err(error),
        };
        if !trigger_effects.is_empty() {
            if interrupt_triggers {
                // Replacement triggers must be able to inspect or rewrite the suspended tail.
                defer_scheduler_tail(tx, &mut sched);
                sched.extend(trigger_effects);
            } else {
                for effect in trigger_effects {
                    sched.push_back(effect);
                }
            }
        }
        if aborts_resolution {
            if let EffectRequest::FinishCurrentPhaseAbortResolution { player, .. } = req {
                sched.push_back(EffectRequest::FinishCurrentPhaseAbortResolution {
                    player,
                    stage: 1,
                });
            }
        }

        if tx.has_buffered_choice() {
            defer_scheduler_front(tx, &mut sched);
            return Err(EffectError::ChoiceRequested);
        }
    }
    let tail = if owns_deferred_tail
        && tx.state().resolution.deferred_effects.is_empty()
        && tx.state().resolution.deferred_triggers.is_empty()
    {
        std::mem::take(&mut tx.state_mut().resolution.deferred_tail_effects)
    } else {
        Vec::new()
    };
    if !tail.is_empty() {
        produced.extend(run_effects(tx, tail)?);
    }
    Ok(produced)
}

/// Continue the canonical effect/trigger queues suspended behind a choice.
pub(crate) fn resume_deferred_resolution(tx: &mut Transaction) -> Result<(), EffectError> {
    loop {
        let effects = std::mem::take(&mut tx.state_mut().resolution.deferred_effects);
        if !effects.is_empty() {
            run_effects(tx, effects)?;
            continue;
        }

        let triggers = std::mem::take(&mut tx.state_mut().resolution.deferred_triggers);
        if !triggers.is_empty() {
            let effects = crate::trigger::run_pending_triggers(tx, triggers)?;
            if !effects.is_empty() {
                tx.state_mut().resolution.deferred_effects = effects;
            }
            continue;
        }
        let tail = std::mem::take(&mut tx.state_mut().resolution.deferred_tail_effects);
        if !tail.is_empty() {
            run_effects(tx, tail)?;
            continue;
        }
        return finish_pending_card_replays_if_idle(tx);
    }
}

/// Convenience: deal damage as a one-shot scheduled chain (for tests / skills).
pub fn schedule_damage(
    tx: &mut Transaction,
    source: PlayerId,
    target: PlayerId,
    amount: u32,
) -> Result<Vec<DomainEvent>, EffectError> {
    run_effects(
        tx,
        vec![EffectRequest::DealDamage {
            source,
            target,
            amount,
            is_lose_hp: false,
            nature: 0,
        }],
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command::apply_command;
    use crate::effect::bootstrap_match;
    use crate::state::GameState;
    use crate::transaction::Transaction;
    use engine_api::{
        ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    };

    fn bootstrap_tx() -> Transaction {
        let mut tx = Transaction::begin(GameState::empty());
        bootstrap_match(&mut tx, 2, 1, "bootstrap").unwrap();
        tx
    }

    #[test]
    fn lethal_damage_opens_dying_choice() {
        let mut tx = bootstrap_tx();
        let p1 = PlayerId::new(1);
        let p2 = PlayerId::new(2);
        let card = tx.state().hand_cards(p1)[0];
        tx.state_mut().cards.get_mut(card).unwrap().kind = crate::state::CardKind::Tao;
        let err = schedule_damage(&mut tx, p1, p2, 4).unwrap_err();
        assert_eq!(err, EffectError::ChoiceRequested);
        assert!(tx.state().has_pending_choice());
        assert_eq!(
            tx.state().resolution.continuation_kind,
            ContinuationKind::DyingSave
        );
        assert!(tx.state().players.get(p2).unwrap().dying);
    }

    #[test]
    fn dying_save_then_kill_via_resolve_choice() {
        let mut tx = bootstrap_tx();
        let p1 = PlayerId::new(1);
        let p2 = PlayerId::new(2);
        // Grant a 桃 so system-save consumes it.
        let _ = run_effects(
            &mut tx,
            vec![EffectRequest::DrawCards {
                player: p2,
                count: 4,
            }],
        );
        // Keep exactly one deterministic 桃: one card must recover exactly one HP.
        {
            use crate::state::{CardKind, ZoneKind};
            let hand_cards = tx.state().hand_cards(p2);
            for cid in &hand_cards {
                tx.state_mut().cards.get_mut(*cid).unwrap().kind = CardKind::Other;
            }
            if let Some(cid) = hand_cards.first().copied() {
                if let Some(c) = tx.state_mut().cards.get_mut(cid) {
                    c.kind = CardKind::Tao;
                }
            } else {
                let hand = tx.state().zones.find(ZoneKind::Hand, p2).unwrap();
                use crate::state::Card;
                use engine_api::{CardDefId, Suit};
                let card = Card {
                    id: engine_api::CardId::NONE,
                    def: CardDefId::new(9991),
                    kind: CardKind::Tao,
                    suit: Suit::Heart,
                    rank: 1,
                    weapon_range: 0,
                    zone: hand,
                    owner: p2,
                    face_up: false,
                };
                let id = tx.state_mut().cards.alloc(card);
                if let Some(z) = tx.state_mut().zones.get_mut(hand) {
                    z.cards.push(id);
                }
            }
        }
        for card in tx.state().hand_cards(p1) {
            tx.state_mut().cards.get_mut(card).unwrap().kind = crate::state::CardKind::Other;
        }
        tx.state_mut().players.get_mut(p2).unwrap().hp = 1;
        let _ = schedule_damage(&mut tx, p1, p2, 1).unwrap_err();
        let committed = tx.commit();
        let choice_id = ChoiceId::new(committed.state.resolution.pending_choice_id);

        // Save (token 1) consumes 桃
        let (state, pipe) = apply_command(
            committed.state,
            Command {
                actor: p2,
                expected_revision: ExpectedRevision(0),
                idempotency_key: IdempotencyKey(0),
                kind: CommandKind::ResolveChoice {
                    choice_id,
                    selections: vec![1],
                    cancelled: false,
                },
            },
        );
        assert!(matches!(pipe.result, CommandResult::Accepted { .. }));
        assert!(state.players.get(p2).unwrap().alive);
        assert_eq!(state.players.get(p2).unwrap().hp, 1);
        assert!(!state.has_pending_choice());

        // Lethal again, decline an available self-save → kill after all responders pass.
        let mut tx = Transaction::begin(state);
        let card = tx.state().hand_cards(p2)[0];
        tx.state_mut().cards.get_mut(card).unwrap().kind = crate::state::CardKind::Tao;
        let _ = schedule_damage(&mut tx, p1, p2, 1).unwrap_err();
        let committed = tx.commit();
        let choice_id = ChoiceId::new(committed.state.resolution.pending_choice_id);
        let (state, pipe) = apply_command(
            committed.state,
            Command {
                actor: p2,
                expected_revision: ExpectedRevision(0),
                idempotency_key: IdempotencyKey(0),
                kind: CommandKind::ResolveChoice {
                    choice_id,
                    selections: vec![0],
                    cancelled: false,
                },
            },
        );
        assert!(matches!(pipe.result, CommandResult::Accepted { .. }));
        assert!(!state.players.get(p2).unwrap().alive);
    }
}
