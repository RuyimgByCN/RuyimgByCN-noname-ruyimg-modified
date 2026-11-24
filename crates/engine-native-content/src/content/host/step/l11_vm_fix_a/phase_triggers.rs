use super::super::super::*;
use super::super::l11_common::{
    alive_all, discard_player_cards, emit_damage, emit_lose_hp, flag, he_cards, set_flag,
};
use super::super::l11_common_b2::emit_virtual_kind_use;
use super::helpers::*;
use crate::catalog::CardKind;
use crate::event::{CardUseData, DamageNature, EventData, LostOrigin};
use crate::ids::PlayerId;
use crate::machine::{Ctx, Flow, LogEvent};

pub(super) fn step(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let owner = m.owner;
    match m.def_key.as_str() {
        "miji" => step_miji(m, ctx, owner),
        "fulin" => {
            set_flag(ctx.core, owner, "l11_fulin_phase_discard", 1);
            ctx.log(LogEvent::SkillTriggered {
                skill: "fulin",
                owner,
                presentation: None,
            });
            Flow::Done
        }
        "langmie" => step_langmie(m, ctx, owner),
        "yingjian" => step_yingjian(m, ctx, owner),
        "kunfen" => step_kunfen(m, ctx, owner),
        "lianpo" => step_lianpo(m, ctx, owner),
        "fenming" => step_fenming(m, ctx, owner),
        "guzheng" => step_guzheng(m, ctx, owner),
        "gzyinghun" => step_gzyinghun(m, ctx, owner),
        "chunlao" => step_chunlao(m, ctx, owner),
        "chunlao_save" => step_chunlao_save(m, ctx, owner),
        "nzry_longnu" => step_longnu(m, ctx, owner),
        _ => Flow::Done,
    }
}

fn step_miji(m: &mut HostMachine, ctx: &mut Ctx<'_>, owner: PlayerId) -> Flow {
    match m.stage {
        HostStage::Act => {
            let x = (ctx.core.player(owner).max_hp as i32 - ctx.core.player(owner).hp).max(0) as u32;
            if x == 0 {
                return Flow::Done;
            }
            m.card = Some(x);
            m.stage = HostStage::DrawSettle;
            ctx.start_draw(owner, x);
            Flow::Yield
        }
        HostStage::DrawSettle => {
            let x = m.card.unwrap_or(0) as usize;
            if x == 0 || !ctx.core.player(owner).alive || ctx.core.player(owner).hand.len() < x {
                return Flow::Done;
            }
            wait_ask(m, ctx, HostStage::SelectCard, ASK_MIJI_CARDS)
        }
        HostStage::SelectCard => {
            clear_ask(ctx, owner);
            let x = m.card.unwrap_or(0) as usize;
            let picks = answered_cards(ctx);
            if picks.len() != x
                || !picks.iter().all(|c| ctx.core.player(owner).hand.contains(c))
            {
                return Flow::Done;
            }
            m.cards = picks;
            wait_ask(m, ctx, HostStage::TargetChoice, ASK_MIJI_TARGET)
        }
        HostStage::TargetChoice => {
            clear_ask(ctx, owner);
            if let Some(t) = answered_players(ctx)
                .into_iter()
                .next()
                .filter(|&p| p != owner && ctx.core.player(p).alive)
            {
                give_he_cards_to_hand(ctx, owner, t, &m.cards);
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_langmie(m: &mut HostMachine, ctx: &mut Ctx<'_>, owner: PlayerId) -> Flow {
    match m.stage {
        HostStage::Act => {
            if he_cards(ctx.core, owner).is_empty() {
                return Flow::Done;
            }
            if m.card == Some(1) {
                wait_ask(m, ctx, HostStage::SelectCard, ASK_LANGMIE_PLAY_DISCARD)
            } else {
                wait_ask(m, ctx, HostStage::SelectNature, ASK_LANGMIE_END_DISCARD)
            }
        }
        HostStage::SelectCard => {
            clear_ask(ctx, owner);
            let Some(c) = answered_cards(ctx)
                .into_iter()
                .next()
                .filter(|cid| he_cards(ctx.core, owner).contains(cid))
            else {
                return Flow::Done;
            };
            ctx.log(LogEvent::SkillTriggered {
                skill: "langmie",
                owner,
                presentation: None,
            });
            discard_player_cards(ctx, owner, &[c]);
            m.stage = HostStage::DrawSettle;
            Flow::Yield
        }
        HostStage::SelectNature => {
            clear_ask(ctx, owner);
            let Some(c) = answered_cards(ctx)
                .into_iter()
                .next()
                .filter(|cid| he_cards(ctx.core, owner).contains(cid))
            else {
                return Flow::Done;
            };
            ctx.log(LogEvent::SkillTriggered {
                skill: "langmie",
                owner,
                presentation: None,
            });
            discard_player_cards(ctx, owner, &[c]);
            m.stage = HostStage::ShuchuiTargetLoop;
            Flow::Yield
        }
        HostStage::DrawSettle => {
            if ctx.core.player(owner).alive {
                ctx.start_draw(owner, 2);
            }
            Flow::Done
        }
        HostStage::ShuchuiTargetLoop => {
            if let Some(t) = m.target.filter(|&p| ctx.core.player(p).alive) {
                emit_damage(ctx, ctx.core.player(owner).alive.then_some(owner), t, 1, DamageNature::Normal, None);
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_yingjian(m: &mut HostMachine, ctx: &mut Ctx<'_>, owner: PlayerId) -> Flow {
    match m.stage {
        HostStage::Act => wait_ask(m, ctx, HostStage::TargetChoice, ASK_YINGJIAN_TARGET),
        HostStage::TargetChoice => {
            clear_ask(ctx, owner);
            let Some(t) = answered_players(ctx).into_iter().next().filter(|&p| {
                can_virtual_sha_target(ctx.core, ctx.catalog, owner, p, true)
            }) else {
                return Flow::Done;
            };
            ctx.log(LogEvent::SkillTriggered {
                skill: "yingjian",
                owner,
                presentation: None,
            });
            emit_virtual_kind_use(ctx, owner, CardKind::Sha, vec![t], DamageNature::Normal);
            m.stage = HostStage::DrawSettle;
            Flow::Yield
        }
        _ => Flow::Done,
    }
}

fn step_kunfen(m: &mut HostMachine, ctx: &mut Ctx<'_>, owner: PlayerId) -> Flow {
    match m.stage {
        HostStage::Act => {
            if flag(ctx.core, owner, "kunfen_unlocked") == 0 {
                ctx.log(LogEvent::SkillTriggered {
                    skill: "kunfen",
                    owner,
                    presentation: None,
                });
                emit_lose_hp(ctx, owner, 1);
                m.stage = HostStage::DrawSettle;
                Flow::Yield
            } else {
                wait_ask(m, ctx, HostStage::TargetChoice, ASK_KUNFEN_CONFIRM)
            }
        }
        HostStage::TargetChoice => {
            clear_ask(ctx, owner);
            if !is_confirmed(ctx) {
                return Flow::Done;
            }
            ctx.log(LogEvent::SkillTriggered {
                skill: "kunfen",
                owner,
                presentation: None,
            });
            emit_lose_hp(ctx, owner, 1);
            m.stage = HostStage::DrawSettle;
            Flow::Yield
        }
        HostStage::DrawSettle => {
            if ctx.core.player(owner).alive {
                ctx.start_draw(owner, 2);
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_lianpo(m: &mut HostMachine, ctx: &mut Ctx<'_>, owner: PlayerId) -> Flow {
    match m.stage {
        HostStage::Act => wait_ask(m, ctx, HostStage::TargetChoice, ASK_LIANPO_CONFIRM),
        HostStage::TargetChoice => {
            clear_ask(ctx, owner);
            if is_confirmed(ctx) && ctx.core.player(owner).alive {
                ctx.log(LogEvent::SkillTriggered {
                    skill: "lianpo",
                    owner,
                    presentation: None,
                });
                set_flag(ctx.core, owner, "extra_turn_insert_queued", 1);
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_fenming(m: &mut HostMachine, ctx: &mut Ctx<'_>, owner: PlayerId) -> Flow {
    match m.stage {
        HostStage::Act => wait_ask(m, ctx, HostStage::TargetChoice, ASK_FENMING_CONFIRM),
        HostStage::TargetChoice => {
            clear_ask(ctx, owner);
            if !is_confirmed(ctx) {
                return Flow::Done;
            }
            ctx.log(LogEvent::SkillTriggered {
                skill: "fenming",
                owner,
                presentation: None,
            });
            let n = ctx.core.players.len();
            let mut linked: Vec<PlayerId> = alive_all(ctx.core, owner)
                .into_iter()
                .filter(|&p| ctx.core.player(p).linked && !he_cards(ctx.core, p).is_empty())
                .collect();
            sort_by_seat_from(&mut linked, owner, n);
            m.targets = linked;
            fenming_next_target(m, ctx)
        }
        HostStage::SelectCard => {
            clear_ask(ctx, owner);
            if let (Some(t), Some(c)) = (m.target, answered_cards(ctx).into_iter().next()) {
                if ctx.core.player(t).alive && he_cards(ctx.core, t).contains(&c) {
                    discard_player_cards(ctx, t, &[c]);
                    m.stage = HostStage::ShuchuiTargetLoop;
                    return Flow::Yield;
                }
            }
            fenming_next_target(m, ctx)
        }
        HostStage::ShuchuiTargetLoop => fenming_next_target(m, ctx),
        _ => Flow::Done,
    }
}

fn fenming_next_target(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    while let Some(t) = m.targets.first().copied() {
        m.targets.remove(0);
        if ctx.core.player(t).alive && !he_cards(ctx.core, t).is_empty() {
            m.target = Some(t);
            return wait_ask(m, ctx, HostStage::SelectCard, ASK_FENMING_CARD);
        }
    }
    Flow::Done
}

fn step_guzheng(m: &mut HostMachine, ctx: &mut Ctx<'_>, owner: PlayerId) -> Flow {
    let Some(t) = m.target.filter(|&p| p != owner && ctx.core.player(p).alive) else {
        return Flow::Done;
    };
    match m.stage {
        HostStage::Act => {
            let (all, own) = guzheng_discarded_lists(ctx.core, t);
            if own.is_empty() {
                return Flow::Done;
            }
            if all.len() == 1 {
                m.card = Some(all[0]);
                wait_ask(m, ctx, HostStage::TargetChoice, ASK_GUZHENG_CONFIRM)
            } else {
                wait_ask(m, ctx, HostStage::SelectCard, ASK_GUZHENG_KEEP)
            }
        }
        HostStage::TargetChoice => {
            clear_ask(ctx, owner);
            if is_confirmed(ctx) {
                if let Some(c) = m.card {
                    ctx.log(LogEvent::SkillTriggered {
                        skill: "guzheng",
                        owner,
                        presentation: None,
                    });
                    let _ = give_single_discard_card_from_discard(ctx, t, c);
                }
            }
            Flow::Done
        }
        HostStage::SelectCard => {
            clear_ask(ctx, owner);
            let (_, own) = guzheng_discarded_lists(ctx.core, t);
            let Some(keep) = answered_cards(ctx)
                .into_iter()
                .next()
                .filter(|c| own.contains(c))
            else {
                return Flow::Done;
            };
            m.card = Some(keep);
            wait_ask(m, ctx, HostStage::ComparePlayerPick, ASK_GUZHENG_GAIN)
        }
        HostStage::ComparePlayerPick => {
            clear_ask(ctx, owner);
            let gain_rest = is_confirmed(ctx);
            let Some(keep) = m.card else {
                return Flow::Done;
            };
            ctx.log(LogEvent::SkillTriggered {
                skill: "guzheng",
                owner,
                presentation: None,
            });
            let _ = give_single_discard_card_from_discard(ctx, t, keep);
            if gain_rest && ctx.core.player(owner).alive {
                let (rem, _) = guzheng_discarded_lists(ctx.core, t);
                let mut gained = Vec::new();
                for c in rem {
                    if ctx.core.discard.contains(&c) {
                        ctx.core.discard.retain(|&dc| dc != c);
                        ctx.core.player_mut(owner).hand.push(c);
                        gained.push(c);
                    }
                }
                if !gained.is_empty() {
                    ctx.out.push(LogEvent::CardsDrawn {
                        player: owner,
                        cards: gained.clone(),
                    });
                    ctx.emit(EventData::CardsDrawn {
                        player: owner,
                        cards: gained,
                    });
                }
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_gzyinghun(m: &mut HostMachine, ctx: &mut Ctx<'_>, owner: PlayerId) -> Flow {
    match m.stage {
        HostStage::Act => wait_ask(m, ctx, HostStage::TargetChoice, ASK_GZYINGHUN_TARGET),
        HostStage::TargetChoice => {
            clear_ask(ctx, owner);
            let Some(t) = answered_players(ctx)
                .into_iter()
                .next()
                .filter(|&p| p != owner && ctx.core.player(p).alive)
            else {
                return Flow::Done;
            };
            m.target = Some(t);
            let x = (ctx.core.player(owner).max_hp as i32 - ctx.core.player(owner).hp).max(1) as u32;
            if x == 1 {
                ctx.log(LogEvent::SkillTriggered {
                    skill: "gzyinghun",
                    owner,
                    presentation: None,
                });
                m.cards = vec![1];
                m.stage = HostStage::DrawSettle;
                ctx.start_draw(t, 1);
                Flow::Yield
            } else {
                wait_ask(m, ctx, HostStage::SelectNature, ASK_GZYINGHUN_MODE)
            }
        }
        HostStage::SelectNature => {
            clear_ask(ctx, owner);
            let Some(mode) = answered_tag(ctx) else {
                return Flow::Done;
            };
            let Some(t) = m.target.filter(|&p| ctx.core.player(p).alive) else {
                return Flow::Done;
            };
            let x = (ctx.core.player(owner).max_hp as i32 - ctx.core.player(owner).hp).max(1) as u32;
            let (draw_n, disc_n) = if mode == 0 { (x, 1) } else { (1, x) };
            ctx.log(LogEvent::SkillTriggered {
                skill: "gzyinghun",
                owner,
                presentation: None,
            });
            m.cards = vec![disc_n];
            m.stage = HostStage::DrawSettle;
            ctx.start_draw(t, draw_n);
            Flow::Yield
        }
        HostStage::DrawSettle => {
            let Some(t) = m.target.filter(|&p| ctx.core.player(p).alive) else {
                return Flow::Done;
            };
            let disc_n = m.cards.first().copied().unwrap_or(1) as usize;
            let pool = he_cards(ctx.core, t);
            if pool.is_empty() {
                return Flow::Done;
            }
            if pool.len() <= disc_n {
                discard_player_cards(ctx, t, &pool);
                return Flow::Done;
            }
            m.card = Some(disc_n as u32);
            wait_ask(m, ctx, HostStage::SelectCard, ASK_GZYINGHUN_DISCARD)
        }
        HostStage::SelectCard => {
            clear_ask(ctx, owner);
            if let Some(t) = m.target.filter(|&p| ctx.core.player(p).alive) {
                let picks: Vec<_> = answered_cards(ctx)
                    .into_iter()
                    .filter(|c| he_cards(ctx.core, t).contains(c))
                    .collect();
                if !picks.is_empty() {
                    discard_player_cards(ctx, t, &picks);
                }
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_chunlao(m: &mut HostMachine, ctx: &mut Ctx<'_>, owner: PlayerId) -> Flow {
    match m.stage {
        HostStage::Act => wait_ask(m, ctx, HostStage::SelectCard, ASK_CHUNLAO_STORE),
        HostStage::SelectCard => {
            clear_ask(ctx, owner);
            let picks: Vec<_> = answered_cards(ctx)
                .into_iter()
                .filter(|&c| {
                    ctx.core.player(owner).hand.contains(&c)
                        && ctx.catalog.resolved_card_kind(ctx.core, c).is_sha_like()
                })
                .collect();
            if picks.is_empty() {
                return Flow::Done;
            }
            ctx.log(LogEvent::SkillTriggered {
                skill: "chunlao",
                owner,
                presentation: None,
            });
            for &c in &picks {
                ctx.take_hand_card(owner, c);
            }
            ctx.core
                .vm_aux
                .piles
                .entry((owner, "chunlao".to_string()))
                .or_default()
                .extend(picks.iter().copied());
            ctx.note_cards_lost(owner, &picks);
            let origins = vec![LostOrigin::Hand; picks.len()];
            ctx.emit(EventData::CardsLost {
                player: owner,
                cards: picks,
                origins,
            });
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_chunlao_save(m: &mut HostMachine, ctx: &mut Ctx<'_>, owner: PlayerId) -> Flow {
    match m.stage {
        HostStage::Act => wait_ask(m, ctx, HostStage::SelectCard, ASK_CHUNLAO_SPEND),
        HostStage::SelectCard => {
            clear_ask(ctx, owner);
            let Some(c) = answered_cards(ctx).into_iter().next() else {
                return Flow::Done;
            };
            let removed = ctx
                .core
                .vm_aux
                .piles
                .get_mut(&(owner, "chunlao".to_string()))
                .is_some_and(|p| {
                    let before = p.len();
                    p.retain(|&pc| pc != c);
                    p.len() < before
                });
            if !removed {
                return Flow::Done;
            }
            if ctx
                .core
                .vm_aux
                .piles
                .get(&(owner, "chunlao".to_string()))
                .is_some_and(Vec::is_empty)
            {
                ctx.core.vm_aux.piles.remove(&(owner, "chunlao".to_string()));
            }
            ctx.note_cards_lost(owner, &[c]);
            ctx.to_discard(c);
            ctx.out.push(LogEvent::CardDiscarded {
                player: owner,
                cards: vec![c],
            });
            ctx.log(LogEvent::SkillTriggered {
                skill: "chunlao",
                owner,
                presentation: None,
            });
            set_flag(ctx.core, owner, "hs_yuelu_saved", 1);
            let dying = m.target.unwrap_or(owner);
            if ctx.core.player(dying).alive {
                ctx.emit(EventData::CardUsed(CardUseData {
                    card: None,
                    player: dying,
                    targets: vec![dying],
                    nature: DamageNature::Normal,
                    use_as: Some(CardKind::Jiu),
                    cancelled: false,
                }));
                let fid = ctx.top_frame_id();
                ctx.core
                    .vm_aux
                    .flags
                    .insert((dying, format!("dying_save_frame:{fid}")), u32::from(dying) + 1);
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

fn step_longnu(m: &mut HostMachine, ctx: &mut Ctx<'_>, owner: PlayerId) -> Flow {
    match m.stage {
        HostStage::Act => {
            ctx.log(LogEvent::SkillTriggered {
                skill: "nzry_longnu",
                owner,
                presentation: None,
            });
            if flag(ctx.core, owner, "l11_nzry_longnu_state") == 0 {
                set_flag(ctx.core, owner, "l11_nzry_longnu_state", 1);
                emit_lose_hp(ctx, owner, 1);
                m.stage = HostStage::DrawSettle;
                Flow::Yield
            } else {
                set_flag(ctx.core, owner, "l11_nzry_longnu_state", 0);
                ctx.lose_max_hp(owner, 1);
                if ctx.core.player(owner).alive {
                    ctx.start_draw(owner, 1);
                    set_flag(ctx.core, owner, "nzry_longnu_thunder_turn", 1);
                }
                Flow::Done
            }
        }
        HostStage::DrawSettle => {
            if ctx.core.player(owner).alive {
                ctx.start_draw(owner, 1);
                set_flag(ctx.core, owner, "nzry_longnu_fire_turn", 1);
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}
