use super::*;

pub(super) fn push_event_data(h: &mut HashSink, data: &EventData) {
    match data {
        EventData::OrdinaryJudge { owner, invocation } => {
            push_u8(h, 26);
            push_u8(h, *owner);
            push_u32(h, *invocation);
        }
        EventData::SkillInvocation(d) => {
            push_u8(h, 24);
            push_u8(h, d.owner);
            push_u16(h, d.def);
            push_u32(h, d.window);
            push_u8(h, d.source.map(|p| p + 1).unwrap_or(0));
            push_u8(h, d.target);
            push_u8(h, d.phase.tag());
            push_u8(h, u8::from(d.cancelled));
            push_u32(h, d.jiding_card.unwrap_or(u32::MAX));
            push_u64(h, d.jiding_changes.len() as u64);
            for c in &d.jiding_changes {
                push_u32(h, c.card);
                push_u16(h, c.def);
                push_u8(h, c.actor);
                push_u8(h, c.before_player);
                push_u8(h, c.before_target);
                match &c.cost {
                    engine_native_kernel::event::JidingCardCost::Zhenlie { lose_hp } => {
                        push_u8(h, 0);
                        push_u32(h, *lose_hp);
                    }
                    engine_native_kernel::event::JidingCardCost::TongJi { card, loss } => {
                        push_u8(h, 1);
                        push_u32(h, *card);
                        push_u32(h, loss.unwrap_or(u32::MAX));
                    }
                    engine_native_kernel::event::JidingCardCost::Yihua { cards, losses } => {
                        push_u8(h, 2);
                        push_cards(h, cards);
                        push_u64(h, losses.len() as u64);
                        for child in losses {
                            push_u32(h, *child);
                        }
                    }
                }
            }
            push_u8(h, d.judgement.map(|s| s as u8 + 1).unwrap_or(0));
            push_u8(h, u8::from(d.deferred_log));
            push_u8(h, u8::from(d.effect_applied));
            if let Some(p) = &d.judge_receipt {
                push_u32(h, p.child);
                push_u8(h, p.target);
                push_u8(
                    h,
                    match p.purpose {
                        engine_native_kernel::event::JudgePurpose::Yuance(o) => o.tag(),
                        engine_native_kernel::event::JudgePurpose::Jidian => 2,
                    },
                );
            } else {
                push_u32(h, u32::MAX);
            }
            push_u8(h, u8::from(d.payment.is_some()));
            if let Some(p) = &d.payment {
                push_cards(h, &p.selected);
                push_u32(h, p.log_begin);
                push_u8(h, u8::from(p.committed));
                push_u64(h, p.loss_children.len() as u64);
                for id in &p.loss_children {
                    push_u32(h, *id)
                }
                push_u32(h, p.after_log.unwrap_or(u32::MAX));
            }
        }
        EventData::SkillLog { owner, def, begin } => {
            push_u8(h, 25);
            push_u8(h, *owner);
            push_u16(h, *def);
            push_u8(h, u8::from(*begin));
        }
        EventData::Damage(d) => {
            push_u8(h, 1);
            push_u8(h, d.source.map(|s| s + 1).unwrap_or(0));
            push_u8(h, d.target);
            push_i32(h, d.amount);
            h.buf.extend_from_slice(&d.settlement.canonical_bytes());
            push_u32(h, d.card.unwrap_or(u32::MAX));
            push_u8(h, u8::from(d.from_chain));
            push_players(h, &d.redirect_chain);
            push_u8(h, d.prevented as u8);
            push_u8(h, d.nature as u8);
        }
        EventData::DamageWindow { root, window } => {
            push_u8(h, 23);
            push_u32(h, *root);
            push_u8(h, window.tag());
        }
        EventData::Recover {
            player,
            amount,
            by_card,
        } => {
            push_u8(h, 2);
            push_u8(h, *player);
            push_u32(h, *amount);
            push_u32(h, by_card.unwrap_or(u32::MAX));
        }
        EventData::CardsDrawn { player, cards } => {
            push_u8(h, 3);
            push_u8(h, *player);
            push_cards(h, cards);
        }
        EventData::CardsMoved { cards, from, to } => {
            push_u8(h, 4);
            push_cards(h, cards);
            push_u8(h, *from);
            push_u8(h, *to);
        }
        EventData::CardUsed(u) => {
            push_u8(h, 5);
            push_u32(h, u.card.unwrap_or(u32::MAX));
            push_u8(h, u.player);
            push_players(h, &u.targets);
            push_u8(h, u.nature as u8);
        }
        EventData::CardResponded { card, player } => {
            push_u8(h, 6);
            push_u32(h, *card);
            push_u8(h, *player);
        }
        EventData::Dying { player, source } => {
            push_u8(h, 7);
            push_u8(h, *player);
            push_u8(h, source.map(|s| s + 1).unwrap_or(0));
        }
        EventData::Death { player, killer } => {
            push_u8(h, 8);
            push_u8(h, *player);
            push_u8(h, killer.map(|k| k + 1).unwrap_or(0));
        }
        EventData::Phase {
            player,
            phase,
            begin,
        } => {
            push_u8(h, 9);
            push_u8(h, *player);
            push_u8(h, *phase as u8);
            push_u8(h, *begin as u8);
        }
        EventData::TurnEnded { player } => {
            push_u8(h, 10);
            push_u8(h, *player);
        }
        EventData::TurnBegin { player } => {
            // R05（规则 333）：回合开始窗，判别值表尾追加 27。
            push_u8(h, 27);
            push_u8(h, *player);
        }
        EventData::DrawPhase {
            player,
            progress,
            pending,
            num_fixed,
            actual_gain,
            outcome,
        } => {
            // R05（规则 333）：抽牌族判别值表尾追加 28–31；D 载荷全量入域。
            push_u8(h, 28);
            push_u8(h, *player);
            push_u8(h, progress.tag());
            push_i32(h, *pending);
            push_u8(h, *num_fixed as u8);
            push_u32(h, *actual_gain);
            push_u8(h, outcome.tag());
        }
        EventData::DrawWindow { root, window } => {
            push_u8(h, 29);
            push_u32(h, *root);
            push_u8(h, window.tag());
        }
        EventData::Draw {
            player,
            requested,
            held_cards,
            terminal,
        } => {
            push_u8(h, 30);
            push_u8(h, *player);
            push_u32(h, *requested);
            push_u32(h, held_cards.len() as u32);
            for c in held_cards {
                push_u32(h, *c);
            }
            push_u8(h, terminal.tag());
        }
        EventData::DrawGain {
            draw,
            recipient,
            eligible,
            progress,
            history_recorded,
            cancelled,
        } => {
            push_u8(h, 31);
            push_u32(h, *draw);
            push_u8(h, *recipient);
            push_u32(h, eligible.len() as u32);
            for c in eligible {
                push_u32(h, *c);
            }
            push_u8(h, progress.tag());
            push_u8(h, *history_recorded as u8);
            push_u8(h, *cancelled as u8);
        }
        EventData::TurnEndWindow { player, after } => {
            // R05（K21）：判别值 32。
            push_u8(h, 32);
            push_u8(h, *player);
            push_u8(h, *after as u8);
        }
        EventData::TurnOverBefore { player, cancelled } => {
            push_u8(h, 20);
            push_u8(h, *player);
            push_u8(h, *cancelled as u8);
        }
        EventData::TurnOvered { player, face_down } => {
            push_u8(h, 21);
            push_u8(h, *player);
            push_u8(h, *face_down as u8);
        }
        EventData::LoseHp { player, amount } => {
            push_u8(h, 22);
            push_u8(h, *player);
            push_u32(h, *amount);
        }
        EventData::ShaDodged {
            card,
            source,
            target,
        } => {
            // swd 批 141（判别 16）
            push_u8(h, 16);
            push_u32(h, card.unwrap_or(u32::MAX));
            push_u8(h, *source);
            push_u8(h, *target);
        }
        EventData::DyingSaved { player, source } => {
            push_u8(h, 23);
            push_u8(h, *player);
            push_u8(h, source.map(|k| k + 1).unwrap_or(0));
        }
        EventData::CardsGiven {
            giver,
            recipient,
            cards,
        } => {
            // NC04 未映射事件批（判别 24）
            push_u8(h, 24);
            push_u8(h, *giver);
            push_u8(h, *recipient);
            push_u32(h, cards.len() as u32);
            for c in cards {
                push_u32(h, *c);
            }
        }
        EventData::SkillToggled {
            player,
            skill,
            enabled,
            temporary,
        } => {
            push_u8(h, 11);
            push_u8(h, *player);
            push_u16(h, *skill);
            push_u8(h, *enabled as u8);
            push_u8(h, *temporary as u8);
        }
        EventData::Equipped {
            player,
            card,
            replaced,
        } => {
            push_u8(h, 12);
            push_u8(h, *player);
            push_u32(h, *card);
            push_u32(h, replaced.unwrap_or(u32::MAX));
        }
        EventData::Linked { player, linked } => {
            push_u8(h, 13);
            push_u8(h, *player);
            push_u8(h, *linked as u8);
        }
        EventData::CardsDiscarded { player, cards } => {
            push_u8(h, 14);
            push_u8(h, *player);
            push_cards(h, cards);
        }
        EventData::CardsLost {
            player,
            cards,
            origins,
        } => {
            // swd 批 125（判别 15）
            push_u8(h, 15);
            push_u8(h, *player);
            push_cards(h, cards);
            for o in origins {
                push_u8(
                    h,
                    match o {
                        crate::event::LostOrigin::Hand => 0,
                        crate::event::LostOrigin::Equip => 1,
                        crate::event::LostOrigin::Judge => 2,
                    },
                );
            }
        }
    }
}
