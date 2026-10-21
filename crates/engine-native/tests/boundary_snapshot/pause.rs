use super::*;

#[test]
fn pause_roundtrip_sha_respond() {
    assert_pause_roundtrip(build_sha_respond, answer_by_prompt);
}

#[test]
fn pause_roundtrip_dying_save() {
    assert_pause_roundtrip(build_dying_save, answer_by_prompt);
}

#[test]
fn pause_roundtrip_duel_respond() {
    assert_pause_roundtrip(build_duel_respond, answer_by_prompt);
}

#[test]
fn pause_roundtrip_aoe_mid_queue() {
    assert_pause_roundtrip(build_aoe_mid_queue, answer_by_prompt);
}

#[test]
fn pause_roundtrip_judge_window() {
    assert_pause_roundtrip(build_judge_window, answer_by_prompt);
}

#[test]
fn pause_roundtrip_chain_target_window() {
    assert_pause_roundtrip(build_chain_window, answer_plays_nullify);
}

#[test]
fn pause_roundtrip_discard_to_limit() {
    assert_pause_roundtrip(build_discard_to_limit, answer_by_prompt);
}

#[test]
fn pause_roundtrip_yishang_chain() {
    assert_pause_roundtrip(build_yishang_target, answer_by_prompt);
}

/// 主动无懈与反无懈逐点往返（窗口 asked/队列重建/逐目标推进全链路）。
#[test]
fn pause_roundtrip_active_nullify_chain() {
    assert_pause_roundtrip(build_nullify_chain, answer_plays_nullify);
}

/// N05：主动鬼道改判（真实出黑牌）逐点往返；逐提交含机器持牌守恒。
#[test]
fn pause_roundtrip_guidao_active_rewrite() {
    assert_pause_roundtrip(build_guidao_active_rewrite, answer_active_judgement);
}

/// N05：天妒确认获得逐点往返（获得路径 judge_held→手牌守恒）。
#[test]
fn pause_roundtrip_tiandu_confirm() {
    assert_pause_roundtrip(build_tiandu_confirm, answer_active_judgement);
}

/// N05：改判→天妒两点链逐点往返（挂起于改判窗口与天妒窗口各一次）。
#[test]
fn pause_roundtrip_rewrite_then_tiandu_combo() {
    assert_pause_roundtrip(build_rewrite_then_tiandu_combo, answer_active_judgement);
}

/// N05：负体力多桃连续救援逐点往返——三个濒死询问点各挂起一次，
/// 基准线/快照线每点哈希、事件增量、终态与守恒一致。
#[test]
fn pause_roundtrip_negative_hp_multi_tao_rescue() {
    assert_pause_roundtrip(build_negative_dying_rescue, answer_plays_tao);
}

/// N05 Decline 分支：改判询问放弃——无 Rewritten、判定牌随效果弃置。
#[test]
fn pause_roundtrip_guidao_pass() {
    assert_pause_roundtrip(build_guidao_active_rewrite, answer_declines_judgement);
}

/// N05 Decline 分支：天妒询问放弃——判定牌从机器持有转入弃牌堆恰一份。
#[test]
fn pause_roundtrip_tiandu_decline() {
    fn build_tiandu_decline() -> Engine {
        build_judgement_window_with(&[("天妒", 1)], None)
    }
    assert_pause_roundtrip(build_tiandu_decline, answer_declines_judgement);
}

/// N05 无懈×洗牌布阵并入通用矩阵：南蛮多目标无懈链逐点往返
///（真实 DeckReshuffled×恢复组合见 savage_multi_nullify_judge_reshuffle_restore）。
#[test]
fn pause_roundtrip_nullify_reshuffle_chain() {
    assert_pause_roundtrip(build_savage_nullify_reshuffle, answer_plays_nullify);
}

/// 判定→闪电伤害→濒死救援嵌套链（JudgePhase→JudgeTrick→Damage→DyingSave）。
#[test]
fn pause_roundtrip_judge_lightning_dying_save() {
    assert_pause_roundtrip(build_judge_lightning_dying, answer_saves_dying);
}

/// 鬼道改判窗口挂起点往返（改判队列/进行态判定牌随快照延续）。
#[test]
fn pause_roundtrip_guidao_rewrite() {
    assert_pause_roundtrip(build_guidao_rewrite, answer_by_prompt);
}
