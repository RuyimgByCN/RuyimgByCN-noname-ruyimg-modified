use super::*;

// ---- 事件数据 ----

pub(super) fn enc_event_data(w: &mut W, v: &EventData) {
    match v {
        EventData::Damage(d) => {
            w.u8(22);
            enc_opt_pid(w, &d.source);
            enc_pid(w, &d.target);
            w.i32(d.amount);
            enc_damage_nature(w, &d.nature);
            w.boolean(d.prevented);
            w.len(d.redirect_chain.len());
            for p in &d.redirect_chain {
                w.u8(*p);
            }
            enc_opt_u32(w, d.card);
            w.boolean(d.from_chain);
            super::damage::enc_settlement(w, &d.settlement);
        }
        EventData::DamageWindow { root, window } => {
            w.u8(21);
            w.u32(*root);
            w.u8(window.tag());
        }
        EventData::Recover {
            player,
            amount,
            by_card,
        } => {
            w.u8(1);
            enc_pid(w, player);
            w.u32(*amount);
            enc_opt_u32(w, *by_card);
        }
        EventData::CardsDrawn { player, cards } => {
            w.u8(2);
            enc_pid(w, player);
            w.len(cards.len());
            for c in cards {
                w.u32(*c);
            }
        }
        EventData::CardsMoved { cards, from, to } => {
            w.u8(3);
            w.len(cards.len());
            for c in cards {
                w.u32(*c);
            }
            enc_pid(w, from);
            enc_pid(w, to);
        }
        EventData::CardUsed(u) => {
            w.u8(4);
            w.u32(u.card);
            enc_pid(w, &u.player);
            w.len(u.targets.len());
            for t in &u.targets {
                w.u8(*t);
            }
            enc_damage_nature(w, &u.nature);
            enc_opt_u32(w, u.use_as.map(|k| card_kind_to_num(&k)));
            w.boolean(u.cancelled);
        }
        EventData::CardResponded { card, player } => {
            w.u8(5);
            w.u32(*card);
            enc_pid(w, player);
        }
        EventData::Dying { player, source } => {
            w.u8(6);
            enc_pid(w, player);
            enc_opt_pid(w, source);
        }
        EventData::Death { player, killer } => {
            w.u8(7);
            enc_pid(w, player);
            enc_opt_pid(w, killer);
        }
        EventData::Phase {
            player,
            phase,
            begin,
        } => {
            w.u8(8);
            enc_pid(w, player);
            enc_phase(w, phase);
            w.boolean(*begin);
        }
        EventData::TurnEnded { player } => {
            w.u8(9);
            enc_pid(w, player);
        }
        EventData::TurnOverBefore { player, cancelled } => {
            // E03 翻面原语（tag 14，表尾追加）
            w.u8(14);
            enc_pid(w, player);
            w.boolean(*cancelled);
        }
        EventData::TurnOvered { player, face_down } => {
            // E03 翻面完成（tag 15，表尾追加）
            w.u8(15);
            enc_pid(w, player);
            w.boolean(*face_down);
        }
        EventData::SkillToggled {
            player,
            skill,
            enabled,
            temporary,
        } => {
            w.u8(10);
            enc_pid(w, player);
            enc_skill(w, skill);
            w.boolean(*enabled);
            w.boolean(*temporary);
        }
        EventData::Equipped {
            player,
            card,
            replaced,
        } => {
            w.u8(11);
            enc_pid(w, player);
            w.u32(*card);
            enc_opt_u32(w, *replaced);
        }
        EventData::Linked { player, linked } => {
            w.u8(12);
            enc_pid(w, player);
            w.boolean(*linked);
        }
        EventData::CardsDiscarded { player, cards } => {
            w.u8(13);
            enc_pid(w, player);
            w.len(cards.len());
            for c in cards {
                w.u32(*c);
            }
        }
        EventData::LoseHp { player, amount } => {
            // 逐技对照批三（tag 16，表尾追加）
            w.u8(16);
            enc_pid(w, player);
            w.u32(*amount);
        }
        EventData::CardsLost {
            player,
            cards,
            origins,
        } => {
            // swd 批 125（判别 18，表尾追加）
            w.u8(18);
            enc_pid(w, player);
            w.len(cards.len());
            for c in cards {
                w.u32(*c);
            }
            for o in origins {
                w.u8(match o {
                    crate::event::LostOrigin::Hand => 0,
                    crate::event::LostOrigin::Equip => 1,
                    crate::event::LostOrigin::Judge => 2,
                });
            }
        }
        EventData::ShaDodged {
            card,
            source,
            target,
        } => {
            // swd 批 141（判别 19，表尾追加）
            w.u8(19);
            w.u32(*card);
            enc_pid(w, source);
            enc_pid(w, target);
        }
        EventData::DyingSaved { player, source } => {
            // 濒死救回（tag 17，表尾追加；焚逝挂点规则 v157）
            w.u8(17);
            enc_pid(w, player);
            enc_opt_pid(w, source);
        }
        EventData::CardsGiven {
            giver,
            recipient,
            cards,
        } => {
            // 给牌触发帧（NC04 未映射事件批，tag 20 表尾追加）
            w.u8(20);
            enc_pid(w, giver);
            enc_pid(w, recipient);
            w.len(cards.len());
            for c in cards {
                w.u32(*c);
            }
        }
    }
}

pub(super) fn dec_event_data(r: &mut R<'_>) -> Result<EventData, ArchiveError> {
    Ok(match r.u8()? {
        tag @ (0 | 22) => {
            let source = dec_opt_pid(r)?;
            let target = dec_pid(r)?;
            let amount = r.i32()?;
            let nature = dec_damage_nature(r)?;
            let prevented = r.boolean()?;
            let chain = r.vec(|r| r.u8())?;
            let card = dec_opt_u32(r)?;
            let from_chain = r.boolean()?;
            let settlement = if tag == 22 {
                super::damage::dec_settlement(r)?
            } else {
                Default::default()
            };
            EventData::Damage(DamageData {
                settlement,
                source,
                target,
                amount,
                nature,
                prevented,
                redirect_chain: chain,
                card,
                from_chain,
            })
        }
        21 => {
            let root = r.u32()?;
            let tag = r.u8()?;
            let window = crate::event::DamageWindowKind::from_tag(tag)
                .ok_or(ArchiveError::BadTag("damage_window", tag))?;
            EventData::DamageWindow { root, window }
        }
        1 => {
            let player = dec_pid(r)?;
            let amount = r.u32()?;
            let by_card = dec_opt_u32(r)?;
            EventData::Recover {
                player,
                amount,
                by_card,
            }
        }
        2 => {
            let player = dec_pid(r)?;
            let cards = r.vec(|r| r.u32())?;
            EventData::CardsDrawn { player, cards }
        }
        3 => {
            let cards = r.vec(|r| r.u32())?;
            let from = dec_pid(r)?;
            let to = dec_pid(r)?;
            EventData::CardsMoved { cards, from, to }
        }
        4 => {
            let card = r.u32()?;
            let player = dec_pid(r)?;
            let targets = r.vec(|r| r.u8())?;
            let nature = dec_damage_nature(r)?;
            let use_as = match dec_opt_u32(r)? {
                Some(n) => Some(card_kind_from_num(n)?),
                None => None,
            };
            let cancelled = r.boolean()?;
            EventData::CardUsed(CardUseData {
                card,
                player,
                targets,
                nature,
                use_as,
                cancelled,
            })
        }
        5 => {
            let card = r.u32()?;
            let player = dec_pid(r)?;
            EventData::CardResponded { card, player }
        }
        6 => {
            let player = dec_pid(r)?;
            let source = dec_opt_pid(r)?;
            EventData::Dying { player, source }
        }
        7 => {
            let player = dec_pid(r)?;
            let killer = dec_opt_pid(r)?;
            EventData::Death { player, killer }
        }
        8 => {
            let player = dec_pid(r)?;
            let phase = dec_phase(r)?;
            let begin = r.boolean()?;
            EventData::Phase {
                player,
                phase,
                begin,
            }
        }
        9 => {
            let player = dec_pid(r)?;
            EventData::TurnEnded { player }
        }
        14 => {
            let player = dec_pid(r)?;
            let cancelled = r.boolean()?;
            EventData::TurnOverBefore { player, cancelled }
        }
        15 => {
            let player = dec_pid(r)?;
            let face_down = r.boolean()?;
            EventData::TurnOvered { player, face_down }
        }
        16 => {
            let player = dec_pid(r)?;
            let amount = r.u32()?;
            EventData::LoseHp { player, amount }
        }
        17 => {
            let player = dec_pid(r)?;
            let source = dec_opt_pid(r)?;
            EventData::DyingSaved { player, source }
        }
        10 => {
            let player = dec_pid(r)?;
            let skill = dec_skill(r)?;
            let enabled = r.boolean()?;
            let temporary = r.boolean()?;
            EventData::SkillToggled {
                player,
                skill,
                enabled,
                temporary,
            }
        }
        11 => {
            let player = dec_pid(r)?;
            let card = r.u32()?;
            let replaced = dec_opt_u32(r)?;
            EventData::Equipped {
                player,
                card,
                replaced,
            }
        }
        12 => {
            let player = dec_pid(r)?;
            let linked = r.boolean()?;
            EventData::Linked { player, linked }
        }
        13 => {
            let player = dec_pid(r)?;
            let cards = r.vec(|r| r.u32())?;
            EventData::CardsDiscarded { player, cards }
        }
        18 => {
            let player = dec_pid(r)?;
            let cards = r.vec(|r| r.u32())?;
            let n = cards.len();
            let mut origins = Vec::with_capacity(n);
            for _ in 0..n {
                origins.push(match r.u8()? {
                    0 => crate::event::LostOrigin::Hand,
                    1 => crate::event::LostOrigin::Equip,
                    2 => crate::event::LostOrigin::Judge,
                    t => return Err(ArchiveError::BadTag("lost_origin", t)),
                });
            }
            EventData::CardsLost {
                player,
                cards,
                origins,
            }
        }
        19 => {
            let card = r.u32()?;
            let source = dec_pid(r)?;
            let target = dec_pid(r)?;
            EventData::ShaDodged {
                card,
                source,
                target,
            }
        }
        20 => {
            let giver = dec_pid(r)?;
            let recipient = dec_pid(r)?;
            let cards = r.vec(|r| r.u32())?;
            EventData::CardsGiven {
                giver,
                recipient,
                cards,
            }
        }
        t => return Err(ArchiveError::BadTag("event_data", t)),
    })
}

// ---- 机器快照 ----

pub(super) fn enc_judgement_progress(w: &mut W, c: &JudgementProgressSnap) {
    w.u32(c.trick);
    w.u32(c.judge_card);
    enc_pids(w, &c.rewriters);
    w.boolean(c.asked);
    enc_pids(w, &c.tian_du);
    w.boolean(c.gained);
    w.boolean(c.in_rewrite);
    w.boolean(c.finalized);
    w.boolean(c.judge_held);
    // v75（swd 卡牌面窗口机批）：昊天塔插窗两字段（尾部追加）
    enc_opt_pid(w, &c.haotian_holder);
    match c.haotian_other {
        None => w.u8(0),
        Some(x) => {
            w.u8(1);
            w.u32(x);
        }
    }
    // v89（T11 Boss 诸葛亮族批 116）：改命 7 张候选（尾部追加）
    match &c.gaiming_cards {
        None => w.u8(0),
        Some(cards) => {
            w.u8(1);
            w.len(cards.len());
            for c in cards {
                w.u32(*c);
            }
        }
    }
}

pub(super) fn dec_judgement_progress(r: &mut R<'_>) -> Result<JudgementProgressSnap, ArchiveError> {
    let trick = r.u32()?;
    let judge_card = r.u32()?;
    let rewriters = dec_pids(r)?;
    let asked = r.boolean()?;
    let tian_du = dec_pids(r)?;
    let gained = r.boolean()?;
    let in_rewrite = r.boolean()?;
    let finalized = r.boolean()?;
    let judge_held = r.boolean()?;
    // v75（swd 卡牌面窗口机批）：昊天塔插窗两字段（尾部追加）
    let haotian_holder = dec_opt_pid(r)?;
    let haotian_other = match r.u8()? {
        0 => None,
        1 => Some(r.u32()?),
        _ => return Err(ArchiveError::UnknownCardKind(255)),
    };
    // v89：改命候选
    let gaiming_cards = match r.u8()? {
        0 => None,
        1 => {
            let len = r.len()?;
            let mut cards = Vec::with_capacity(len);
            for _ in 0..len {
                cards.push(r.u32()?);
            }
            Some(cards)
        }
        _ => return Err(ArchiveError::UnknownCardKind(255)),
    };
    Ok(JudgementProgressSnap {
        trick,
        judge_card,
        rewriters,
        asked,
        tian_du,
        gained,
        in_rewrite,
        finalized,
        judge_held,
        haotian_holder,
        haotian_other,
        gaiming_cards,
    })
}
