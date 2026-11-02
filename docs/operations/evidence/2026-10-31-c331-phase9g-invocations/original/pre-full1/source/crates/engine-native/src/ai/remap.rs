//! Typed card references in simulation continuations; ownership stays in its original zone.
use crate::{Engine, EventData, MachineSnapshot};

pub(super) fn event_cards(event: &mut EventData, map: &impl Fn(&mut u32)) {
    match event {
        EventData::Damage(d) => d.card.iter_mut().for_each(map),
        EventData::Recover { by_card, .. } => by_card.iter_mut().for_each(map),
        EventData::CardUsed(u) => map(&mut u.card),
        EventData::CardResponded { card, .. } | EventData::ShaDodged { card, .. } => map(card),
        EventData::Equipped { card, replaced, .. } => {
            map(card);
            replaced.iter_mut().for_each(map);
        }
        EventData::CardsDrawn { cards, .. }
        | EventData::CardsMoved { cards, .. }
        | EventData::CardsDiscarded { cards, .. }
        | EventData::CardsLost { cards, .. }
        | EventData::CardsGiven { cards, .. } => cards.iter_mut().for_each(map),
        _ => {}
    }
}

fn effects(bytes: &mut Vec<u8>, map: &impl Fn(&mut u32)) {
    use rule_api::EffectRequest;
    if bytes.is_empty() {
        return;
    }
    let mut effects = crate::rvm_ops::dec_effects(bytes).expect("validated VM effects");
    for effect in &mut effects {
        let mut card = |id: &mut rule_api::CardId| {
            // VM zero is a no-card sentinel, unlike native CardId zero.
            if id.raw() != 0 {
                let mut raw = id.raw();
                map(&mut raw);
                *id = rule_api::CardId::new(raw);
            }
        };
        match effect {
            EffectRequest::StoreEventCardInSkillPile { card: id, .. } => card(id),
            EffectRequest::MoveCards { cards, .. } | EffectRequest::RevealCards { cards, .. } => {
                cards.iter_mut().for_each(&mut card);
            }
            _ => {}
        }
    }
    bytes.clear();
    crate::rvm_ops::enc_effects(bytes, &effects);
}

pub(super) fn machine_cards(machine: &mut MachineSnapshot, map: &impl Fn(&mut u32)) {
    use MachineSnapshot::*;
    if let JudgeContinuation{payment:Some(p),..}=machine{map(&mut p.old);}
    if let NativeInvocation{progress,..}=machine{use engine_native_kernel::event::NativeInvocationProgress as P;match progress{P::HolderAsk{candidates}|P::SourceAsk{candidates,..}|P::HolderPay{cards:candidates}=>candidates.iter_mut().for_each(map),_=>{}}}
    match machine {
        Sha {
            card,
            liuli_card,
            qilin_horse,
            guanshi_cards,
            fuchou_card,
            ..
        } => {
            map(card);
            liuli_card
                .iter_mut()
                .chain(qilin_horse)
                .chain(fuchou_card)
                .for_each(map);
            guanshi_cards.iter_mut().for_each(map);
        }
        Duel { card, .. }
        | PlaceDelayed { card, .. }
        | AoeTarget { card, .. }
        | JianXiong { card, .. }
        | Dismantle { card, .. }
        | MonkeySteal { card, .. }
        | Snatch { card, .. } => map(card),
        NullifyWindow { card, effect, .. } => {
            map(card);
            if let Some(inner) = effect {
                machine_cards(inner, map);
            }
        }
        Aoe { card, inner, .. } | Chain { card, inner, .. } => {
            map(card);
            if let Some(inner) = inner {
                machine_cards(inner, map);
            }
        }
        OptionalGate { event, inner, .. } => {
            event_cards(event, map);
            if let Some(inner) = inner {
                machine_cards(inner, map);
            }
        }
        JudgeContinuation{queue,current,..}|JudgePhase { queue, current, .. } => {
            queue.iter_mut().for_each(map);
            if let Some(c) = current {
                if let Some(trick)=&mut c.trick{map(trick);}
                map(&mut c.judge_card);
                c.haotian_other.iter_mut().for_each(map);
                if let Some(cards) = &mut c.gaiming_cards {
                    cards.iter_mut().for_each(map);
                }
            }
        }
        Zhangba { cards, .. } | Wugu { cards, .. } | Rende { cards, .. } | JieYin { cards, .. } => {
            cards.iter_mut().for_each(map)
        }
        XunXun { top, .. } | GuanXing { top, .. } => top.iter_mut().for_each(map),
        JiJie { card, .. } => card.iter_mut().for_each(map),
        Jijiang { sha, inner, .. } => {
            sha.iter_mut().for_each(map);
            if let Some(inner) = inner {
                machine_cards(inner, map);
            }
        }
        LiJian { card, inner, .. } => {
            card.iter_mut().for_each(map);
            if let Some(inner) = inner {
                machine_cards(inner, map);
            }
        }
        // Host slots also encode suit/kind/seat/bool; they need behavior/stage semantics.
        Host { .. } => {}
        Rvm {
            event,
            paused,
            commit_pending,
            ..
        } => {
            event_cards(event, map);
            effects(commit_pending, map);
            if let Some(paused) = paused {
                effects(&mut paused.pending, map);
            }
        }
        _ => {}
    }
}

pub(super) fn requires_conservative_redaction(machine: &MachineSnapshot) -> bool {
    use MachineSnapshot::*;
    match machine {
        Rvm { paused, commit_pick, committing, commit_state, following, .. } => {
            paused.is_some() || commit_pick.is_some() || *committing
                || !commit_state.is_empty() || !following.is_empty()
        }
        // Preserve these pending owners until their sampling semantics are audited.
        XunXun { .. } | JiJie { .. } => true,
        JudgePhase {
            current: Some(current),
            ..
        } if current.haotian_other.is_some() || current.gaiming_cards.is_some() => true,
        Host { behavior, .. } => *behavior != crate::content::host::HostBehavior::D3Moyan.tag(),
        OptionalGate { inner, .. }
        | NullifyWindow { effect: inner, .. }
        | Aoe { inner, .. }
        | Chain { inner, .. }
        | Jijiang { inner, .. }
        | LiJian { inner, .. } => inner
            .as_deref()
            .is_some_and(requires_conservative_redaction),
        _ => false,
    }
}

/// These are physical owners in the current restore validator, not historical references.
pub(super) fn held_cards(machine: &MachineSnapshot, out: &mut Vec<u32>) {
    use MachineSnapshot::*;
    match machine {
        JudgeContinuation{queue,current,..}|JudgePhase { queue, current, .. } => {
            out.extend(queue);
            if let Some(c) = current {
                if !c.finalized {
                    out.extend(c.trick);
                }
                if c.judge_held {
                    out.push(c.judge_card);
                }
            }
        }
        GuanXing { top, .. } => out.extend(top),
        Host {
            behavior, cards, ..
        } if *behavior == crate::content::host::HostBehavior::D3Moyan.tag() => out.extend(cards),
        OptionalGate { inner, .. }
        | NullifyWindow { effect: inner, .. }
        | Aoe { inner, .. }
        | Chain { inner, .. }
        | Jijiang { inner, .. }
        | LiJian { inner, .. } => {
            if let Some(inner) = inner {
                held_cards(inner, out);
            }
        }
        _ => {}
    }
}

pub(super) fn remap_resolution(engine: &mut Engine, mapping: &[u32]) {
    let map = |card: &mut u32| {
        if let Some(next) = mapping.get(*card as usize) {
            *card = *next;
        }
    };
    let catalog = engine.catalog().clone();
    let resolution = engine.resolution_mut();
    for frame in &mut resolution.stack {
        event_cards(&mut frame.data, &map);
    }
    for park in &mut resolution.parks {
        let mut machine = park.machine.snapshot();
        let original = machine.clone();
        machine_cards(&mut machine, &map);
        if machine != original {
            park.machine = machine
                .restore(&catalog)
                .expect("simulation machine retains its type");
        }
    }
    let option = |o: &mut crate::ChoiceOption| {
        if let crate::ChoiceOption::Card(card) = o {
            map(card);
        }
    };
    if let Some(choice) = &mut resolution.open_choice {
        choice.spec.options.iter_mut().for_each(option);
    }
    if let Some(answer) = &mut resolution.answer {
        answer.iter_mut().for_each(option);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::host::{HostBehavior, HostStage};

    #[test]
    fn host_scalar_payloads_are_never_generic_card_references() {
        for (behavior, stage, card, cards) in [
            (
                HostBehavior::Liaoyuan,
                HostStage::SelectCard,
                Some(3),
                vec![],
            ),
            (
                HostBehavior::Reguhuo,
                HostStage::TargetChoice,
                Some(81),
                vec![1],
            ),
            (
                HostBehavior::SwdQixingzhenCard,
                HostStage::SelectCard,
                Some(82),
                vec![1, 2],
            ),
            (
                HostBehavior::WudianPingzhang,
                HostStage::SelectCard,
                Some(1),
                vec![],
            ),
        ] {
            let mut machine = MachineSnapshot::Host {
                def_key: "payload-probe".into(),
                behavior: behavior.tag(),
                stage: stage.tag(),
                owner: 0,
                target: Some(1),
                card,
                cards,
                targets: vec![],
            };
            let original = machine.clone();
            machine_cards(&mut machine, &|card| *card += 100);
            assert_eq!(
                machine, original,
                "{behavior:?}: payload slots contain scalars as well as cards"
            );
            assert!(requires_conservative_redaction(&machine));
        }
    }
}
