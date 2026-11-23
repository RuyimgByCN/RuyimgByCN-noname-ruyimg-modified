//! 扩展包挂起询问策略共享辅助函数（`public_attitude`、`extension_card_value`）。

pub(in crate::ai) fn public_attitude(view: &crate::view::PlayerView, seat: u8, target: u8) -> i32 {
    if target == seat {
        return 10;
    }
    if let (Some(my_team), Some(target_team)) = (
        view.players.get(seat as usize).and_then(|p| p.team),
        view.players.get(target as usize).and_then(|p| p.team),
    ) {
        return if my_team == target_team { 6 } else { -5 };
    }
    let my_role = view.players.get(seat as usize).and_then(|p| p.role);
    let target_role = view.players.get(target as usize).and_then(|p| p.role);
    match my_role {
        Some(crate::state::Role::Lord) | Some(crate::state::Role::Loyalist) => match target_role {
            Some(crate::state::Role::Lord) | Some(crate::state::Role::Loyalist) => 6,
            _ => -5,
        },
        Some(crate::state::Role::Rebel) => match target_role {
            Some(crate::state::Role::Lord) => -6,
            Some(crate::state::Role::Rebel) => 6,
            _ => -4,
        },
        _ => -5,
    }
}

pub(in crate::ai) fn extension_card_value(
    catalog: &crate::catalog::Catalog,
    input: &crate::view::PolicyInput,
    card: crate::ids::CardId,
) -> i64 {
    let kind = input
        .own_hand_kinds
        .iter()
        .find(|(c, _)| *c == card)
        .map(|(_, k)| *k)
        .or_else(|| catalog.cards.get(card as usize).map(|d| d.kind));
    match kind {
        Some(crate::catalog::CardKind::Tao) => 80,
        Some(crate::catalog::CardKind::Jiu) | Some(crate::catalog::CardKind::Wuzhong) => 70,
        Some(crate::catalog::CardKind::Shan) => 40,
        Some(crate::catalog::CardKind::Sha) => 35,
        Some(_) => 45,
        None => 30,
    }
}
