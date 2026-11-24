//! 宿主事件聚合分派（跨扩展钩子顺序扇出，自 `content/host.rs` 职责拆出）。

use super::*;

pub(crate) fn host_on_outer_settled(ctx: &mut Ctx<'_>) {
    on_outer_settled_taipingyaoshu(ctx);
    host_on_outer_settled_hearth(ctx);
    step::l11_npc_b::on_outer_settled_l11_npc_b(ctx);
}

pub(crate) fn host_on_phase_end(ctx: &mut Ctx<'_>) {
    on_phase_end_l10_scs(ctx);
    host_on_phase_end_hearth(ctx);
    step::l11_npc_b::on_phase_end_l11_npc_b(ctx);
}

pub(crate) fn host_play_begin_step(ctx: &mut Ctx<'_>, player: PlayerId) {
    play_begin_tianshu(ctx, player);
    play_begin_l19(ctx, player);
    host_play_begin_xianjian(ctx, player);
}

pub(crate) fn host_sha_hit_step_all(
    ctx: &mut Ctx<'_>,
    sha_frame_id: crate::ids::EventId,
    source: PlayerId,
    target: PlayerId,
    sha_card: Option<CardId>,
) -> bool {
    step::swd_u05_7b::host_sha_hit_step_u05_7b(ctx, sha_frame_id, source, target, sha_card)
}

pub(crate) fn host_sha_end_step_all(
    ctx: &mut Ctx<'_>,
    sha_frame_id: crate::ids::EventId,
    source: PlayerId,
    target: PlayerId,
    sha_card: Option<CardId>,
    is_damage: bool,
) -> bool {
    let u05_yield = step::swd_u05_7b::host_sha_end_step_u05_7b(ctx, source, is_damage);
    let step_yield = host_sha_end_step(ctx, sha_frame_id, source, target, sha_card, is_damage);
    let xianjian_yield = host_sha_end_xianjian(ctx, sha_frame_id, source, target);
    let npc_a_yield =
        step::l11_npc_a::on_sha_end_step_l11_npc_a(ctx, sha_frame_id, source, target, is_damage);
    u05_yield || step_yield || xianjian_yield || npc_a_yield
}

pub(crate) fn host_draw_begin_step_all(ctx: &mut Ctx<'_>, player: PlayerId, pending: &mut i32) {
    host_draw_begin_step(ctx, player, pending);
    host_draw_begin_xianjian(ctx, player, pending);
    step::swd_u05_7b::host_draw_begin_step_u05_7b(ctx, player, pending);
    step::l11_npc_b::on_draw_begin_step_l11_npc_b(ctx, player, pending);
}

pub(crate) fn host_card_use_after_all(
    ctx: &mut Ctx<'_>,
    fid: crate::ids::EventId,
    u: &crate::event::CardUseData,
) {
    host_card_use_after(ctx, fid, u);
    host_card_use_after_xianjian(ctx, u);
    step::swd_u05_7a::host_card_use_after_swd(ctx, u);
    step::swd_u05_7b::host_card_use_after_u05_7b(ctx, u);
    step::l11_npc_b::on_card_use_after_l11_npc_b(ctx, fid, u);
    step::l11_npc_a::on_card_use_after_l11_npc_a(ctx, fid, u);
    step::l11_vm_fix_a::on_card_use_after(ctx, fid, u);
}

pub(crate) fn host_on_outer_settled_all(ctx: &mut Ctx<'_>) {
    host_on_outer_settled(ctx);
    step::swd_u05_7b::on_outer_settled_u05_7b(ctx);
}

pub(crate) fn host_on_phase_use_end_all(ctx: &mut Ctx<'_>, player: PlayerId) {
    host_on_phase_use_end(ctx, player);
    host_on_phase_use_end_xianjian(ctx, player);
    step::l11_npc_b::on_phase_use_end_l11_npc_b(ctx, player);
    step::l11_npc_a::on_phase_use_end_l11_npc_a(ctx, player);
    step::l11_vm_fix_a::on_phase_use_end(ctx, player);
}

pub(crate) fn host_on_round_start_all(ctx: &mut Ctx<'_>) {
    host_on_round_start(ctx);
    host_on_round_start_xianjian(ctx);
    step::l11_npc_b::on_round_start_l11_npc_b(ctx);
}

pub(crate) fn host_on_hand_card_removed_all(ctx: &mut Ctx<'_>, player: PlayerId, card: CardId) {
    host_on_hand_card_removed(ctx, player, card);
    host_on_hand_card_removed_xianjian(ctx, player, card);
    step::l11_npc_a::on_hand_card_removed_l11_npc_a(ctx, player, card);
}

pub(crate) fn host_on_phase_end_all(ctx: &mut Ctx<'_>) {
    host_on_phase_end(ctx);
    host_on_phase_end_xianjian(ctx);
    step::swd_u05_7b::on_phase_end_u05_7b(ctx);
    step::l11_npc_a::on_phase_end_l11_npc_a(ctx);
}

pub(crate) fn host_on_cards_to_discard_all(
    ctx: &mut Ctx<'_>,
    actor: PlayerId,
    cards: &[CardId],
    from_discard_phase: bool,
) {
    on_cards_to_discard_chengye(ctx, actor, cards, from_discard_phase);
    step::l11_npc_a::on_cards_to_discard_l11_npc_a(ctx, actor, cards);
}

pub(crate) fn host_on_game_start_all(core: &mut crate::state::GameStateCore, catalog: &Catalog) {
    host_on_game_start(core, catalog);
    step::l11_npc_a::on_game_start_l11_npc_a(core, catalog);
}

pub(crate) fn host_on_death_before_all(ctx: &mut Ctx<'_>, dying: PlayerId) -> bool {
    host_on_death_before(ctx, dying) || step::l11_npc_a::on_death_before_l11_npc_a(ctx, dying)
}

pub(crate) fn host_on_turn_phase_after_all(ctx: &mut Ctx<'_>, player: PlayerId) {
    host_on_turn_phase_after(ctx, player);
    step::l11_npc_b::on_turn_phase_after_l11_npc_b(ctx, player);
    step::l11_npc_a::on_turn_phase_after_l11_npc_a(ctx, player);
}

pub(crate) fn host_on_turn_begin_all(ctx: &mut Ctx<'_>, player: PlayerId) {
    host_on_turn_begin(ctx, player);
    step::l11_npc_b::on_turn_begin_l11_npc_b(ctx, player);
    step::l11_npc_a::on_turn_begin_l11_npc_a(ctx, player);
}

pub(crate) fn host_on_equip_gained_all(ctx: &mut Ctx<'_>, player: PlayerId, card: CardId) {
    host_on_equip_gained_hearth(ctx, player, card);
    step::l11_npc_a::on_equip_changed_l11_npc_a(ctx.core, ctx.catalog, player, card, false);
}

pub(crate) fn host_on_equip_revoked_all(
    core: &mut crate::state::GameStateCore,
    catalog: &Catalog,
    player: PlayerId,
    card: CardId,
) {
    host_on_equip_revoked_hearth(core, catalog, player, card);
    step::l11_npc_b::on_equip_revoked_l11_npc_b(core, catalog, player, card);
    step::l11_npc_a::on_equip_changed_l11_npc_a(core, catalog, player, card, true);
}

pub(crate) fn host_on_damage_end_all(ctx: &mut Ctx<'_>, d: &crate::event::DamageData) {
    host_on_damage_end_xianjian(ctx, d);
    step::l11_npc_b::on_damage_end_l11_npc_b(ctx, d);
    step::l11_npc_a::on_damage_end_l11_npc_a(ctx, d);
}

pub(crate) fn host_on_player_died_all(
    ctx: &mut Ctx<'_>,
    dead: PlayerId,
    killer: Option<PlayerId>,
) {
    host_on_player_died(ctx, dead, killer);
    step::l11_npc_b::on_player_died_l11_npc_b(ctx, dead, killer);
}

pub(crate) fn host_on_phase_skipped_all(ctx: &mut Ctx<'_>, player: PlayerId) {
    u01_phase_skipped_hook(ctx, player);
    step::l11_npc_b::on_phase_skipped_l11_npc_b(ctx, player);
}

pub(crate) fn host_on_host_event_all(ctx: &mut Ctx<'_>, event: &EventData) {
    on_l25_host_event(ctx, event);
    step::l11_npc_b::on_host_event_l11_npc_b(ctx, event);
}

pub(crate) fn host_on_hp_changed_all(ctx: &mut Ctx<'_>, player: PlayerId, delta: i32) {
    host_on_hp_changed_xianjian(ctx, player, delta);
    step::l11_npc_b::on_hp_changed_l11_npc_b(ctx, player, delta);
}

pub(crate) fn host_on_recover_before_all(ctx: &mut Ctx<'_>, player: PlayerId, amount: &mut u32) {
    host_on_recover_before(ctx, player, amount);
    host_on_recover_before_xianjian(ctx, player, amount);
}
