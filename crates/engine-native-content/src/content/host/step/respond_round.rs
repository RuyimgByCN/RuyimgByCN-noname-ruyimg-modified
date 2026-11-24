//! 宿主响应技能选项/激活与轮次开始聚合（自 `content/host/step.rs` 职责拆出）。

use super::*;

pub(in crate::content::host) fn sha_card_use_data(
    ctx: &Ctx<'_>,
    actor: PlayerId,
    card: CardId,
    targets: Vec<PlayerId>,
    use_as: Option<CardKind>,
) -> crate::event::CardUseData {
    let eff_kind = use_as.unwrap_or_else(|| ctx.catalog.card_kind(card));
    let nature = match eff_kind {
        CardKind::FireSha => crate::event::DamageNature::Fire,
        CardKind::LeiSha => crate::event::DamageNature::Thunder,
        _ => crate::event::DamageNature::Normal,
    };
    crate::machine::card_use_with_nature(card, actor, targets, nature, use_as)
}

pub(crate) fn u01_respond_skill_options(
    core: &crate::state::GameStateCore,
    catalog: &crate::catalog::Catalog,
    player: crate::ids::PlayerId,
    need: CardKind,
    is_use: bool,
) -> Vec<crate::choice::ChoiceOption> {
    let mut out = u01_fixes_1::u01_respond_skill_options(core, catalog, player, need, is_use);
    u01_fixes_2::u01_respond_skill_options_2(core, catalog, player, need, is_use, &mut out);
    u01_fixes_3::u01_respond_skill_options_3(core, catalog, player, need, is_use, &mut out);
    l11_npc_b::respond_skill_options_l11_npc_b(core, catalog, player, need, is_use, &mut out);
    l11_npc_a::aocai_respond_skill_options(core, catalog, player, need, is_use, &mut out);
    out
}

pub(crate) fn u01_try_activate_respond_skill(
    ctx: &mut Ctx<'_>,
    player: crate::ids::PlayerId,
    def: crate::ids::SkillDefId,
    need: CardKind,
    fid: crate::ids::EventId,
) -> Option<bool> {
    if let Some(res) = u01_fixes_1::u01_try_activate_respond_skill(ctx, player, def, need, fid) {
        return Some(res);
    }
    let vs_sha = ctx.res.stack.last().is_some_and(|f| {
        matches!(&f.data, crate::event::EventData::CardUsed(u) if u.effective_kind(ctx.catalog).is_sha_like())
    });
    let is_use = matches!(need, CardKind::Tao | CardKind::Nullify)
        || (need == CardKind::Shan && vs_sha);
    u01_fixes_2::u01_try_activate_respond_skill_2(ctx, player, def, need, fid, is_use)
        .or_else(|| {
            u01_fixes_3::u01_try_activate_respond_skill_3(ctx, player, def, need, fid, is_use)
        })
        .or_else(|| {
            l11_npc_b::try_activate_respond_skill_l11_npc_b(ctx, player, def, need, fid, is_use)
        })
        .or_else(|| {
            l11_npc_a::aocai_try_activate_respond_skill(ctx, player, def, need, fid, is_use)
        })
}

/// 轮次开始复合宿主钩子：依次执行 L27 轮标/展示清理、L19 `_relifeskill` 倒数复活与 U05-6a hearth 清理。
pub(crate) fn host_on_round_start(ctx: &mut Ctx<'_>) {
    host_on_round_start_l27(ctx);
    host_on_round_start_l19(ctx);
    u05_hearth::host_on_round_start_hearth(ctx);
    u05_b5a_gwent::u05_gwent_on_round_start(ctx);
    l11_own_npc::on_round_start_l11_own_npc(ctx);
}
