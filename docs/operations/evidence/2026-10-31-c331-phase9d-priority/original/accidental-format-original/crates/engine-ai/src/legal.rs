//! Legal action generator (authority-side evaluation for the seated AI).
//!
//! Uses full rules for legality of *own* actions only; never exposes others' hidden cards
//! as selectable tokens beyond what the player would know.

use engine_api::{CardId, ChoiceId, LegalAction, PlayerId};
use engine_core::{query, CardKind, ContinuationKind, Engine, Phase, QueryContext};
use rule_api::{QueryRequest, QueryResponse};
/// Enumerate legal actions for `actor` given full engine (server AI for that seat).
pub fn legal_actions(engine: &Engine, actor: PlayerId) -> Vec<LegalAction> {
    let st = engine.state();
    if st.outcome.is_finished() {
        return vec![];
    }

    // Choice barrier
    if st.has_pending_choice() {
        if st.resolution.pending_observer != actor.raw() {
            return vec![];
        }
        return accepted_actions(engine, actor, legal_choices(engine, actor));
    }

    if st.turn.current != actor {
        return vec![];
    }

    let mut actions = Vec::new();
    match st.turn.phase {
        Phase::Play => {
            actions.extend(legal_skills(engine, actor));
            actions.extend(legal_plays(engine, actor));
            actions.push(LegalAction::EndPhase);
        }
        Phase::Draw | Phase::Prepare | Phase::Judge | Phase::Discard | Phase::End | Phase::Idle => {
            actions.push(LegalAction::EndPhase);
        }
    }
    accepted_actions(engine, actor, actions)
}

fn legal_skills(engine: &Engine, actor: PlayerId) -> Vec<LegalAction> {
    let state = engine.state();
    let Some(player) = state.players.get(actor) else {
        return Vec::new();
    };
    let targets: Vec<PlayerId> = state
        .players
        .iter_live()
        .filter(|candidate| candidate.alive && candidate.id != actor)
        .map(|candidate| candidate.id)
        .collect();
    let hand = state.hand_cards(actor);
    let mut candidates = Vec::new();

    for skill_id in &player.skills {
        let Some(skill) = state.skills.get(*skill_id) else {
            continue;
        };
        if !skill.enabled {
            continue;
        }

        let view_as = state.content.view_as_defs_for_skill(skill.def);
        if !view_as.is_empty() {
            let dynamic: Vec<_> = view_as
                .iter()
                .filter(|rule| {
                    rule.source_zone.as_deref() == Some("none")
                        && rule.branch == Some(skill.view_as_branch)
                        && (!rule.once_per_round
                            || skill.view_as_use_round != state.turn.round
                            || rule.branch.is_none_or(|branch| {
                                skill.view_as_used_branches & (1u32 << branch) == 0
                            }))
                })
                .collect();
            if !dynamic.is_empty() {
                for rule in dynamic {
                    let kinds: &[CardKind] = match rule.as_kind.as_str() {
                        "basic" => &[
                            CardKind::Sha,
                            CardKind::ShaFire,
                            CardKind::ShaThunder,
                            CardKind::Tao,
                            CardKind::Shan,
                            CardKind::Jiu,
                        ],
                        "ordinary_trick" => &[
                            CardKind::TrickWuzhong,
                            CardKind::TrickGuohe,
                            CardKind::TrickShunshou,
                            CardKind::TrickJuedou,
                            CardKind::TrickNanman,
                            CardKind::TrickWanjian,
                            CardKind::TrickTaoyuan,
                            CardKind::TrickWugu,
                            CardKind::TrickJiedao,
                            CardKind::TrickTiesuo,
                            CardKind::TrickHuogong,
                            CardKind::TrickWuxie,
                        ],
                        _ => &[],
                    };
                    let equip_slots: Vec<u8> = match rule.cost.as_deref() {
                        Some("gain_max_hp_1") => vec![0],
                        Some("disable_equip_slot") => (1..=5)
                            .filter(|tag| {
                                engine_core::EquipSlot::from_tag(*tag)
                                    .is_some_and(|slot| player.equip_slot_enabled(slot))
                            })
                            .collect(),
                        _ => Vec::new(),
                    };
                    for kind in kinds {
                        for equip_slot in &equip_slots {
                            for selected in source_free_target_sets(state, actor, *kind) {
                                candidates.push(LegalAction::UseSkillCard {
                                    skill: *skill_id,
                                    card_kind: kind.tag(),
                                    targets: selected,
                                    equip_slot: *equip_slot,
                                });
                            }
                        }
                    }
                }
                continue;
            }
            let has_sha = view_as.iter().any(|rule| rule.as_kind == "sha");
            let has_other = view_as.iter().any(|rule| rule.as_kind != "sha");
            for card in &hand {
                if has_other {
                    candidates.push(LegalAction::ActivateSkill {
                        skill: *skill_id,
                        targets: Vec::new(),
                        card_targets: vec![*card],
                    });
                    for target in &targets {
                        candidates.push(LegalAction::ActivateSkill {
                            skill: *skill_id,
                            targets: vec![*target],
                            card_targets: vec![*card],
                        });
                    }
                }
                if has_sha {
                    for selected in sha_target_sets(engine, actor, *card, &targets) {
                        candidates.push(LegalAction::ActivateSkill {
                            skill: *skill_id,
                            targets: selected,
                            card_targets: vec![*card],
                        });
                    }
                }
            }
            continue;
        }

        if let Some(active) = state.content.first_active_for_skill(skill.def) {
            let target_sets = if let Some(spec) = &active.targets {
                let tokens: Vec<u32> = state
                    .players
                    .iter_live()
                    .filter(|candidate| candidate.alive)
                    .map(|candidate| candidate.id.raw())
                    .collect();
                let mut combinations = Vec::new();
                for count in usize::from(spec.min)..=usize::from(spec.max).min(tokens.len()) {
                    collect_combinations(&tokens, count, 0, &mut Vec::new(), &mut combinations);
                }
                combinations
                    .into_iter()
                    .map(|set| set.into_iter().map(PlayerId::new).collect())
                    .collect()
            } else {
                vec![Vec::new()]
            };
            for selected_targets in target_sets {
                candidates.push(LegalAction::ActivateSkill {
                    skill: *skill_id,
                    targets: selected_targets,
                    card_targets: Vec::new(),
                });
            }
        }
    }

    candidates
}

fn source_free_target_sets(
    state: &engine_core::state::GameState,
    actor: PlayerId,
    kind: CardKind,
) -> Vec<Vec<PlayerId>> {
    let alive: Vec<PlayerId> = state.players.alive_by_seat();
    let others: Vec<PlayerId> = alive
        .iter()
        .copied()
        .filter(|player| *player != actor)
        .collect();
    match kind {
        CardKind::Sha | CardKind::ShaFire | CardKind::ShaThunder => {
            if engine_core::query::sha_targets_all_for(state, actor) {
                return vec![others];
            }
            let max = engine_core::query::sha_target_limit_for(state, actor) as usize;
            let tokens: Vec<u32> = others.iter().map(|player| player.raw()).collect();
            let mut combinations = Vec::new();
            for count in 1..=max.min(tokens.len()) {
                collect_combinations(&tokens, count, 0, &mut Vec::new(), &mut combinations);
            }
            combinations
                .into_iter()
                .map(|set| set.into_iter().map(PlayerId::new).collect())
                .collect()
        }
        CardKind::TrickGuohe | CardKind::TrickShunshou | CardKind::TrickJuedou => {
            let max =
                engine_core::query::card_target_limit_for_kind(state, actor, kind, 1) as usize;
            let tokens: Vec<u32> = others.iter().map(|player| player.raw()).collect();
            let mut combinations = Vec::new();
            for count in 1..=max.min(tokens.len()) {
                collect_combinations(&tokens, count, 0, &mut Vec::new(), &mut combinations);
            }
            combinations
                .into_iter()
                .map(|set| set.into_iter().map(PlayerId::new).collect())
                .collect()
        }
        CardKind::TrickHuogong => {
            let max =
                engine_core::query::card_target_limit_for_kind(state, actor, kind, 1) as usize;
            let tokens: Vec<u32> = alive.iter().map(|player| player.raw()).collect();
            let mut combinations = Vec::new();
            for count in 1..=max.min(tokens.len()) {
                collect_combinations(&tokens, count, 0, &mut Vec::new(), &mut combinations);
            }
            combinations
                .into_iter()
                .map(|set| set.into_iter().map(PlayerId::new).collect())
                .collect()
        }
        CardKind::TrickTiesuo => {
            let mut out = vec![Vec::new()];
            let max =
                engine_core::query::card_target_limit_for_kind(state, actor, kind, 2) as usize;
            let tokens: Vec<u32> = alive.iter().map(|player| player.raw()).collect();
            let mut combinations = Vec::new();
            for count in 1..=max.min(tokens.len()) {
                collect_combinations(&tokens, count, 0, &mut Vec::new(), &mut combinations);
            }
            out.extend(
                combinations
                    .into_iter()
                    .map(|set| set.into_iter().map(PlayerId::new).collect::<Vec<_>>()),
            );
            out
        }
        CardKind::TrickJiedao => {
            let mut out = Vec::new();
            for holder in &others {
                for victim in &alive {
                    if holder != victim {
                        out.push(vec![*holder, *victim]);
                    }
                }
            }
            out
        }
        CardKind::Shan | CardKind::TrickWuxie => Vec::new(),
        _ => vec![Vec::new()],
    }
}

fn accepted_actions(
    engine: &Engine,
    actor: PlayerId,
    candidates: Vec<LegalAction>,
) -> Vec<LegalAction> {
    let revision = engine.state().revision;
    candidates
        .into_iter()
        .filter(|action| {
            let mut trial = engine.clone();
            matches!(
                trial.submit(action.to_command(actor, revision)),
                engine_api::CommandResult::Accepted { .. }
            )
        })
        .collect()
}

fn legal_choices(engine: &Engine, actor: PlayerId) -> Vec<LegalAction> {
    let st = engine.state();
    if let Some(request) = engine
        .last_choice()
        .filter(|request| request.observer == actor)
    {
        let tokens: Vec<u32> = request.options.iter().map(|option| option.token).collect();
        if st.resolution.continuation_kind == ContinuationKind::ActiveSkillSelection {
            let selection = st
                .resolution
                .continuation_payload
                .first()
                .map(|skill| engine_api::SkillId::new(*skill))
                .and_then(|skill| st.skills.get(skill))
                .and_then(|skill| st.content.first_active_for_skill(skill.def))
                .and_then(|active| active.selection);
            if selection
                .as_ref()
                .is_some_and(|selection| selection.same_suit)
            {
                let cards: Vec<_> = tokens.iter().copied().map(CardId::new).collect();
                let mut selections = Vec::new();
                for group in engine_core::query::effective_suit_card_groups(st, actor, &cards) {
                    if let Some(sum) = selection
                        .as_ref()
                        .and_then(|selection| selection.sum_effective_number_exact)
                    {
                        selections.extend(engine_core::query::effective_number_sum_subsets(
                            st,
                            actor,
                            &group,
                            usize::from(request.min_count),
                            usize::from(request.max_count),
                            sum,
                            64 - selections.len(),
                        ));
                    } else {
                        let tokens: Vec<_> = group.into_iter().map(CardId::raw).collect();
                        let max = usize::from(request.max_count).min(tokens.len());
                        for count in usize::from(request.min_count)..=max {
                            collect_combinations(
                                &tokens,
                                count,
                                0,
                                &mut Vec::new(),
                                &mut selections,
                            );
                            if selections.len() >= 64 {
                                break;
                            }
                        }
                    }
                    if selections.len() >= 64 {
                        break;
                    }
                }
                let mut actions: Vec<_> = selections
                    .into_iter()
                    .take(64)
                    .map(|selections| LegalAction::ResolveChoice {
                        choice_id: request.choice_id,
                        selections,
                        cancelled: false,
                    })
                    .collect();
                if request.allow_cancel {
                    actions.push(LegalAction::ResolveChoice {
                        choice_id: request.choice_id,
                        selections: vec![],
                        cancelled: true,
                    });
                }
                return actions;
            }
            if let Some(sum) = selection
                .as_ref()
                .and_then(|selection| selection.sum_effective_number_exact)
            {
                let cards: Vec<_> = tokens.iter().copied().map(CardId::new).collect();
                let mut actions: Vec<_> = engine_core::query::effective_number_sum_subsets(
                    st,
                    actor,
                    &cards,
                    usize::from(request.min_count),
                    usize::from(request.max_count),
                    sum,
                    64,
                )
                .into_iter()
                .map(|selections| LegalAction::ResolveChoice {
                    choice_id: request.choice_id,
                    selections,
                    cancelled: false,
                })
                .collect();
                if request.allow_cancel {
                    actions.push(LegalAction::ResolveChoice {
                        choice_id: request.choice_id,
                        selections: vec![],
                        cancelled: true,
                    });
                }
                return actions;
            }
        }
        let mut selections = Vec::new();
        let max = usize::from(request.max_count).min(tokens.len());
        for count in usize::from(request.min_count)..=max {
            collect_combinations(&tokens, count, 0, &mut Vec::new(), &mut selections);
            if selections.len() >= 64 {
                break;
            }
        }
        let mut actions: Vec<LegalAction> = selections
            .into_iter()
            .take(64)
            .map(|selections| LegalAction::ResolveChoice {
                choice_id: request.choice_id,
                selections,
                cancelled: false,
            })
            .collect();
        if request.allow_cancel {
            actions.push(LegalAction::ResolveChoice {
                choice_id: request.choice_id,
                selections: vec![],
                cancelled: true,
            });
        }
        return actions;
    }

    let choice_id = ChoiceId::new(st.resolution.pending_choice_id);
    match st.resolution.continuation_kind {
        ContinuationKind::DyingSave => {
            vec![
                LegalAction::ResolveChoice {
                    choice_id,
                    selections: vec![0],
                    cancelled: false,
                },
                LegalAction::ResolveChoice {
                    choice_id,
                    selections: vec![1],
                    cancelled: false,
                },
            ]
        }
        ContinuationKind::DiscardToLimit => {
            let need = *st.resolution.continuation_payload.get(1).unwrap_or(&0) as usize;
            let hand = engine_core::query::discard_to_limit_tokens(st, actor);
            if hand.len() < need {
                return vec![];
            }
            // Generate combinations of `need` cards — cap for search (first N combos)
            let mut out = Vec::new();
            // Simple: contiguous windows + first-need greedy
            if need == 0 {
                return vec![LegalAction::ResolveChoice {
                    choice_id,
                    selections: vec![],
                    cancelled: false,
                }];
            }
            // All single-card discards when need==1
            if need == 1 {
                for c in &hand {
                    out.push(LegalAction::ResolveChoice {
                        choice_id,
                        selections: vec![c.raw()],
                        cancelled: false,
                    });
                }
                return out;
            }
            // For need>=2: take first `need` in order (one option) + rotate starts
            for start in 0..=hand.len().saturating_sub(need) {
                let sel: Vec<u32> = hand[start..start + need].iter().map(|c| c.raw()).collect();
                out.push(LegalAction::ResolveChoice {
                    choice_id,
                    selections: sel,
                    cancelled: false,
                });
                if out.len() >= 16 {
                    break;
                }
            }
            out
        }
        _ => vec![LegalAction::ResolveChoice {
            choice_id,
            selections: vec![0],
            cancelled: false,
        }],
    }
}

fn collect_combinations(
    tokens: &[u32],
    needed: usize,
    start: usize,
    current: &mut Vec<u32>,
    out: &mut Vec<Vec<u32>>,
) {
    if out.len() >= 64 {
        return;
    }
    if current.len() == needed {
        out.push(current.clone());
        return;
    }
    let remaining = needed - current.len();
    for index in start..=tokens.len().saturating_sub(remaining) {
        current.push(tokens[index]);
        collect_combinations(tokens, needed, index + 1, current, out);
        current.pop();
        if out.len() >= 64 {
            break;
        }
    }
}

fn legal_plays(engine: &Engine, actor: PlayerId) -> Vec<LegalAction> {
    let st = engine.state();
    let Some(me) = st.players.get(actor) else {
        return vec![];
    };
    if !me.alive || me.dying {
        return vec![];
    }
    let hand = st.hand_cards(actor);
    let mut out = Vec::new();
    let enemies: Vec<PlayerId> = st
        .players
        .iter_live()
        .filter(|p| p.alive && p.id != actor)
        .map(|p| p.id)
        .collect();

    for &card in &hand {
        let Some(c) = st.cards.get(card) else {
            continue;
        };
        match engine_core::query::effective_card_kind(st, actor, c) {
            CardKind::Shan => {}
            CardKind::Ying => {}
            CardKind::Tao => {
                if me.hp < me.max_hp as i32 {
                    out.push(LegalAction::PlayCard {
                        card,
                        targets: vec![],
                    });
                }
            }
            CardKind::Jiu => {
                out.push(LegalAction::PlayCard {
                    card,
                    targets: vec![],
                });
            }
            CardKind::Sha | CardKind::ShaFire | CardKind::ShaThunder => {
                let used = st
                    .turn
                    .flag(engine_core::state::turn_flag::SHA_PLAYED)
                    .max(0) as u32;
                if used >= engine_core::query::sha_use_limit_for_card(st, actor, card) {
                    continue;
                }
                for targets in sha_target_sets(engine, actor, card, &enemies) {
                    out.push(LegalAction::PlayCard { card, targets });
                }
            }
            CardKind::Weapon
            | CardKind::Armor
            | CardKind::HorseOff
            | CardKind::HorseDef
            | CardKind::Treasure
            | CardKind::TrickWuzhong
            | CardKind::TrickNanman
            | CardKind::TrickWanjian
            | CardKind::TrickTaoyuan
            | CardKind::TrickWugu
            | CardKind::TrickDiaobingqianjiang => {
                out.push(LegalAction::PlayCard {
                    card,
                    targets: vec![],
                });
            }
            CardKind::TrickWangmeizhike => {
                for target in st.players.iter_live().filter(|target| target.alive) {
                    out.push(LegalAction::PlayCard {
                        card,
                        targets: vec![target.id],
                    });
                }
            }
            CardKind::TrickChuqibuyi => {
                for target in st.players.iter_live().filter(|target| {
                    target.alive && target.id != me.id && st.hand_count(target.id) > 0
                }) {
                    out.push(LegalAction::PlayCard {
                        card,
                        targets: vec![target.id],
                    });
                }
            }
            CardKind::TrickYiyi => {
                // 以逸待劳（国战）：己方全体自动目标，无需选择。
                out.push(LegalAction::PlayCard {
                    card,
                    targets: vec![],
                });
            }
            CardKind::TrickShuiyanqijunx => {
                // 水淹七军（国战）：目标须为装备区有牌的其他角色。
                for target in st.players.iter_live().filter(|target| target.alive) {
                    if target.id != actor
                        && !engine_core::query::discardable_equipment_cards(
                            st, target.id, target.id,
                        )
                        .is_empty()
                    {
                        out.push(LegalAction::PlayCard {
                            card,
                            targets: vec![target.id],
                        });
                    }
                }
            }
            CardKind::TrickLulitongxin => {
                // 戮力同心（国战）：选任一存活角色为代表，自动扩展到同势群体。
                for target in st.players.iter_live().filter(|target| target.alive) {
                    out.push(LegalAction::PlayCard {
                        card,
                        targets: vec![target.id],
                    });
                }
            }
            CardKind::TrickKefuzhongyuan => {
                // 克复中原（国战）：任意其他角色目标。
                for target in st.players.iter_live().filter(|target| target.alive) {
                    if target.id != actor {
                        out.push(LegalAction::PlayCard {
                            card,
                            targets: vec![target.id],
                        });
                    }
                }
            }
            CardKind::TrickGuguoanbang => {
                // 固国安邦（国战）：只对自己使用。
                out.push(LegalAction::PlayCard {
                    card,
                    targets: vec![actor],
                });
            }
            CardKind::TrickHuoshaolianying => {
                // 火烧连营（国战）：目标自动 = 下家及其队列。
                out.push(LegalAction::PlayCard {
                    card,
                    targets: vec![],
                });
            }
            CardKind::TrickLianjunshengyan => {
                // 联军盛宴（国战）：选一名敌方势力代表。
                for target in st.players.iter_live().filter(|target| target.alive) {
                    if target.id != actor && !engine_core::mode::same_side(st, actor, target.id) {
                        out.push(LegalAction::PlayCard {
                            card,
                            targets: vec![target.id],
                        });
                    }
                }
            }
            CardKind::TrickWenheluanwu => {
                // 文和乱武（国战）：目标自动 = 除自己外所有有手牌的角色。
                if st.players.iter_live().any(|target| {
                    target.alive && target.id != actor && st.hand_count(target.id) > 0
                }) {
                    out.push(LegalAction::PlayCard {
                        card,
                        targets: vec![],
                    });
                }
            }
            CardKind::TrickHaolingtianxia => {
                // 号令天下（国战）：选一名体力值非全场最少（isMinHp 取反）的其他角色。
                let min_hp = st
                    .players
                    .iter_live()
                    .filter(|target| target.alive)
                    .map(|target| target.hp)
                    .min()
                    .unwrap_or(i32::MAX);
                for target in st.players.iter_live().filter(|target| target.alive) {
                    if target.id != actor && target.hp > min_hp {
                        out.push(LegalAction::PlayCard {
                            card,
                            targets: vec![target.id],
                        });
                    }
                }
            }
            CardKind::TrickChiling => {
                // 敕令（国战）：目标自动 = 所有有暗置武将的角色（含使用者）。
                if st
                    .players
                    .iter_live()
                    .any(|target| target.alive && (target.unseen[0] || target.unseen[1]))
                {
                    out.push(LegalAction::PlayCard {
                        card,
                        targets: vec![],
                    });
                }
            }
            CardKind::TrickYoudishenru => {}
            CardKind::TrickGuohe => {
                for &t in &enemies {
                    if target_has_zone_cards(st, t, true, true, true) {
                        out.push(LegalAction::PlayCard {
                            card,
                            targets: vec![t],
                        });
                    }
                }
            }
            CardKind::TrickShunshou => {
                for &t in &enemies {
                    if engine_core::query::card_target_within_distance(st, actor, card, t, 1)
                        && target_has_zone_cards(st, t, true, true, false)
                    {
                        out.push(LegalAction::PlayCard {
                            card,
                            targets: vec![t],
                        });
                    }
                }
            }
            CardKind::TrickHuogong => {
                // 可对有手牌的存活角色使用（含自己）
                for p in st.players.iter_live() {
                    if !st.hand_cards(p.id).is_empty() {
                        out.push(LegalAction::PlayCard {
                            card,
                            targets: vec![p.id],
                        });
                    }
                }
            }
            CardKind::TrickJuedou => {
                for &t in &enemies {
                    out.push(LegalAction::PlayCard {
                        card,
                        targets: vec![t],
                    });
                }
            }
            CardKind::TrickYuanjiao => {
                for &target in &enemies {
                    out.push(LegalAction::PlayCard {
                        card,
                        targets: vec![target],
                    });
                }
            }
            CardKind::TrickTiaojiyanmei => {
                for (index, &left) in enemies.iter().enumerate() {
                    for &right in &enemies[index + 1..] {
                        if st.hand_count(left) != st.hand_count(right) {
                            out.push(LegalAction::PlayCard {
                                card,
                                targets: vec![left, right],
                            });
                        }
                    }
                }
            }
            CardKind::TrickMantianguohai => {
                let options: Vec<_> = enemies
                    .iter()
                    .copied()
                    .filter(|target| target_has_zone_cards(st, *target, true, true, true))
                    .collect();
                for (index, &left) in options.iter().enumerate() {
                    out.push(LegalAction::PlayCard {
                        card,
                        targets: vec![left],
                    });
                    for &right in &options[index + 1..] {
                        out.push(LegalAction::PlayCard {
                            card,
                            targets: vec![left, right],
                        });
                    }
                }
            }
            CardKind::TrickLe => {
                for &t in &enemies {
                    out.push(LegalAction::PlayCard {
                        card,
                        targets: vec![t],
                    });
                }
            }
            CardKind::TrickBingliang => {
                for &t in &enemies {
                    if engine_core::query::card_target_within_distance(st, actor, card, t, 1) {
                        out.push(LegalAction::PlayCard {
                            card,
                            targets: vec![t],
                        });
                    }
                }
            }
            CardKind::TrickShandian => {
                out.push(LegalAction::PlayCard {
                    card,
                    targets: vec![],
                });
            }
            CardKind::TrickTiesuo => {
                // recast (0 targets) always legal for AI
                out.push(LegalAction::PlayCard {
                    card,
                    targets: vec![],
                });
                for &t in &enemies {
                    out.push(LegalAction::PlayCard {
                        card,
                        targets: vec![t],
                    });
                }
            }
            CardKind::TrickJiedao => {
                // AI: skip complex 2-target for now
            }
            CardKind::TrickWuxie => {
                // response-only
            }
            CardKind::Other => {
                out.push(LegalAction::PlayCard {
                    card,
                    targets: vec![],
                });
            }
        }
    }
    out
}

fn sha_in_range(engine: &Engine, actor: PlayerId, card: CardId, target: PlayerId) -> bool {
    if engine_core::query::sha_distance_unlimited_for(engine.state(), actor)
        || engine_core::query::card_distance_unlimited_for_card(engine.state(), actor, card)
    {
        return true;
    }
    let mut ctx = QueryContext::new();
    let dist = query(
        engine.state(),
        &QueryRequest::Distance {
            from: actor,
            to: target,
        },
        &mut ctx,
    );
    let range = query(
        engine.state(),
        &QueryRequest::AttackRange { player: actor },
        &mut ctx,
    );
    matches!(
        (dist, range),
        (QueryResponse::Int(d), QueryResponse::Int(r)) if d <= r
    )
}

fn sha_target_sets(
    engine: &Engine,
    actor: PlayerId,
    card: CardId,
    candidates: &[PlayerId],
) -> Vec<Vec<PlayerId>> {
    if engine_core::query::sha_targets_all_for(engine.state(), actor) {
        let targets: Vec<PlayerId> = candidates
            .iter()
            .copied()
            .filter(|target| sha_in_range(engine, actor, card, *target))
            .collect();
        return if targets.is_empty() {
            Vec::new()
        } else {
            vec![targets]
        };
    }
    let tokens: Vec<u32> = candidates
        .iter()
        .copied()
        .filter(|target| sha_in_range(engine, actor, card, *target))
        .map(PlayerId::raw)
        .collect();
    let max = engine_core::query::sha_target_limit_for(engine.state(), actor) as usize;
    let mut combinations = Vec::new();
    for count in 1..=max.min(tokens.len()) {
        collect_combinations(&tokens, count, 0, &mut Vec::new(), &mut combinations);
        if combinations.len() >= 64 {
            break;
        }
    }
    combinations
        .into_iter()
        .map(|targets| targets.into_iter().map(PlayerId::new).collect())
        .collect()
}

/// Whether target has cards in the requested zones (手/装/判).
fn target_has_zone_cards(
    st: &engine_core::state::GameState,
    target: PlayerId,
    hand: bool,
    equip: bool,
    judge: bool,
) -> bool {
    if hand && !st.hand_cards(target).is_empty() {
        return true;
    }
    if equip {
        if let Some(eq) = st.zones.find(engine_core::state::ZoneKind::Equip, target) {
            if st
                .zones
                .get(eq)
                .map(|z| !z.cards.is_empty())
                .unwrap_or(false)
            {
                return true;
            }
        }
    }
    if judge {
        if let Some(jz) = st.zones.find(engine_core::state::ZoneKind::Judge, target) {
            if st
                .zones
                .get(jz)
                .map(|z| !z.cards.is_empty())
                .unwrap_or(false)
            {
                return true;
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use engine_api::{
        CardDefId, CardId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
        SkillDefId, Suit,
    };
    use engine_core::content::skill_def;
    use engine_core::effect::run_effects;
    use engine_core::state::{Card, CardKind, ZoneKind};
    use engine_core::transaction::Transaction;
    use engine_core::Snapshot;
    use rule_api::EffectRequest;

    fn start_eng() -> Engine {
        let mut eng = Engine::new();
        let r = eng.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(0),
            kind: CommandKind::StartGame {
                player_count: 2,
                rng_seed: 11,
                ruleset: "identity-standard".into(),
            },
        });
        assert!(matches!(r, CommandResult::Accepted { .. }));
        eng
    }

    #[test]
    fn draw_phase_only_end_phase() {
        let eng = start_eng();
        let actor = eng.state().turn.current;
        let acts = legal_actions(&eng, actor);
        assert!(acts.iter().all(|a| matches!(a, LegalAction::EndPhase)));
    }

    #[test]
    fn active_skill_is_enumerated_and_accepted() {
        let mut eng = start_eng();
        let actor = eng.state().turn.current;
        let mut tx = Transaction::begin(eng.state().clone());
        tx.state_mut().turn.phase = Phase::Play;
        run_effects(
            &mut tx,
            vec![EffectRequest::AddSkill {
                player: actor,
                skill_def: SkillDefId::new(skill_def::KUROU),
            }],
        )
        .unwrap();
        let skill = *tx
            .state()
            .players
            .get(actor)
            .unwrap()
            .skills
            .last()
            .unwrap();
        eng.restore_snapshot(&Snapshot::capture(tx.state()))
            .unwrap();

        let action = legal_actions(&eng, actor)
            .into_iter()
            .find(|action| {
                matches!(
                    action,
                    LegalAction::ActivateSkill {
                        skill: found,
                        ..
                    } if *found == skill
                )
            })
            .expect("active skill action");
        assert!(matches!(
            eng.submit(action.to_command(actor, 0)),
            CommandResult::Accepted { .. }
        ));
    }

    #[test]
    fn same_suit_choice_enumeration_never_offers_mixed_cards() {
        let package = content_compiler::compile_source_named(
            "same-suit-legal-test",
            "skill same_suit(4200003041) {
                active play_phase select_hand 3 3 allow_cancel same_suit {
                    effect discard_selected(self);
                }
            }",
        )
        .unwrap();
        let mut eng = Engine::new();
        eng.load_package(&package).unwrap();
        assert!(matches!(
            eng.submit(Command {
                actor: PlayerId::NONE,
                expected_revision: ExpectedRevision(0),
                idempotency_key: IdempotencyKey(0),
                kind: CommandKind::StartGame {
                    player_count: 2,
                    rng_seed: 304,
                    ruleset: "identity-standard".into(),
                },
            }),
            CommandResult::Accepted { .. }
        ));
        let actor = eng.state().turn.current;
        let mut tx = Transaction::begin(eng.state().clone());
        run_effects(
            &mut tx,
            vec![
                EffectRequest::DiscardFromHand {
                    player: actor,
                    count: u32::MAX,
                },
                EffectRequest::AddSkill {
                    player: actor,
                    skill_def: SkillDefId::new(4_200_003_041),
                },
            ],
        )
        .unwrap();
        let skill = *tx
            .state()
            .players
            .get(actor)
            .unwrap()
            .skills
            .last()
            .unwrap();
        let hand = tx.state().zones.find(ZoneKind::Hand, actor).unwrap();
        for (index, suit) in [Suit::Heart, Suit::Heart, Suit::Heart, Suit::Spade]
            .into_iter()
            .enumerate()
        {
            let card = tx.state_mut().cards.alloc(Card {
                id: CardId::NONE,
                def: CardDefId::new(304_100 + index as u32),
                kind: CardKind::Sha,
                suit,
                rank: 7,
                weapon_range: 0,
                zone: hand,
                owner: actor,
                face_up: false,
            });
            tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
        }
        tx.state_mut().turn.phase = Phase::Play;
        eng.restore_snapshot(&Snapshot::capture(tx.state()))
            .unwrap();
        assert!(matches!(
            eng.submit(Command {
                actor,
                expected_revision: ExpectedRevision(eng.state().revision),
                idempotency_key: IdempotencyKey(1),
                kind: CommandKind::ActivateSkill {
                    skill,
                    targets: Vec::new(),
                    card_targets: Vec::new(),
                },
            }),
            CommandResult::Accepted { .. }
        ));

        let actions = legal_actions(&eng, actor);
        assert!(actions.iter().any(|action| matches!(
            action,
            LegalAction::ResolveChoice {
                cancelled: true,
                ..
            }
        )));
        let selections: Vec<_> = actions
            .iter()
            .filter_map(|action| match action {
                LegalAction::ResolveChoice {
                    selections,
                    cancelled: false,
                    ..
                } => Some(selections),
                _ => None,
            })
            .collect();
        assert_eq!(selections.len(), 1);
        assert_eq!(selections[0].len(), 3);
        assert!(selections[0].iter().all(|card| {
            eng.state()
                .cards
                .get(CardId::new(*card))
                .is_some_and(|card| card.suit == Suit::Heart)
        }));
    }
}
