use super::*;

/// N05：攻杀负载抽干牌库迫使洗回，逐命令断言无重复无丢失。
#[test]
fn reshuffle_conserves_every_card_exactly_once() {
    let mut e = Engine::new(301, 4);
    advance_to_play(&mut e, 0);
    let mut attacked = false;
    for _cmd in 0..600 {
        if e.outcome().is_some() {
            break;
        }
        if e.turn().phase == Phase::Play && !attacked {
            let actor = e.turn().current;
            if let Some(sha) = e
                .debug_hand(actor)
                .iter()
                .copied()
                .find(|c| e.catalog().card_kind(*c) == CardKind::Sha)
            {
                let target = e.state_core().next_alive(actor);
                let out = e.submit(GameCommand {
                    actor,
                    expected_revision: e.revision(),
                    dedup_id: fresh_dedup(),
                    action: GameAction::PlayCard {
                        card: sha,
                        targets: vec![target],
                    },
                });
                answer_and_drive(&mut e, out);
                attacked = true;
                assert_zone_conservation_named(&mut e, "sha 后");
                continue;
            }
        }
        attacked = false;
        let out = next_phase(&mut e);
        answer_and_drive(&mut e, out);
        assert_zone_conservation_named(&mut e, "推进后");
    }
}
