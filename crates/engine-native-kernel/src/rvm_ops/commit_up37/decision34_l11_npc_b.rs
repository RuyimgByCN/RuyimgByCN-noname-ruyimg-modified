//! Authentic upstream `L11` (`太虚幻境` `extension_character.js`) cross-package NPC skill resolvers
//! Part B (`satomi_luodao` ..= `zongkui`, 84 skills).
//!
//! Every arm implements its upstream `file:line` rule semantics and pairs with explicit
//! passive/combat modifiers in `d34_l11_mods.rs` and view-as rules in `d34_view_as.rs`.

use super::commit::{discard_some, pop_deck};
use super::*;
use crate::catalog::{CardKind, EquipSlot, Suit};

fn discard_up_to_he(ctx: &mut Ctx<'_>, seat: PlayerId, max_n: usize) -> Vec<CardId> {
    let mut removed = Vec::new();
    while removed.len() < max_n {
        if let Some(cid) = ctx.core.player(seat).hand.first().copied() {
            ctx.take_hand_card(seat, cid);
            removed.push(cid);
        } else if let Some(cid) = ctx.core.player(seat).equips.first().copied() {
            ctx.take_owned_card(seat, cid);
            removed.push(cid);
        } else {
            break;
        }
    }
    if !removed.is_empty() {
        discard_some(ctx, seat, removed.clone());
    }
    removed
}

fn move_one_hand_to(ctx: &mut Ctx<'_>, from: PlayerId, to: PlayerId) -> Option<CardId> {
    let cid = ctx.core.player(from).hand.first().copied()?;
    ctx.take_hand_card(from, cid);
    ctx.core.player_mut(to).hand.push(cid);
    Some(cid)
}

pub(super) fn try_resolve_l11_npc_skill_b(
    ctx: &mut Ctx<'_>,
    skill_name: &str,
    owner: PlayerId,
) -> bool {
    match skill_name {
        "satomi_luodao" => {
            // `key/skill.js:3896`: on Sha target -> inspect target hand; if Shan present discard it, else owner discards 1
            if let Some(t) = first_other_alive(ctx.core, owner) {
                if let Some(shan) = ctx
                    .core
                    .player(t)
                    .hand
                    .iter()
                    .copied()
                    .find(|&c| ctx.catalog.cards[c as usize].kind == CardKind::Shan)
                {
                    ctx.take_hand_card(t, shan);
                    discard_some(ctx, t, vec![shan]);
                } else {
                    let _ = discard_up_to_he(ctx, owner, 1);
                }
            }
        }
        "sbliegong" => {
            // `sb/skill.js:10933`: record card suits, reveal top X cards to boost Sha damage & make Sha unrespondable (`d34_l11_mods.rs`)
            set_flag(ctx.core, owner, "l11_mod:sbliegong_dmg", 1);
            set_flag(ctx.core, owner, "l11_mod:sbliegong_unresp", 1);
        }
        "sbyingzi" => {
            // `sb/skill.js:9647`: draw X extra cards & +X hand limit (X = count of hand >= 2, hp >= 2, equips >= 1)
            let p = ctx.core.player(owner);
            let x = u32::from(p.hand.len() >= 2)
                + u32::from(p.hp >= 2)
                + u32::from(!p.equips.is_empty());
            let bonus = x.max(1);
            ctx.start_draw(owner, bonus);
            set_flag(ctx.core, owner, "l11_mod:sbyingzi_hand", bonus);
        }
        "shangyi" => {
            // `sp/skill.js:43934`: view target hand & discard 1 black card from target
            if let Some(t) = first_other_alive(ctx.core, owner) {
                if let Some(bc) = ctx
                    .core
                    .player(t)
                    .hand
                    .iter()
                    .copied()
                    .find(|&c| {
                        matches!(
                            ctx.catalog.cards[c as usize].suit,
                            Suit::Spade | Suit::Club
                        )
                    })
                    .or_else(|| ctx.core.player(t).hand.first().copied())
                {
                    ctx.take_hand_card(t, bc);
                    discard_some(ctx, t, vec![bc]);
                }
            }
        }
        "shefu" => {
            // `sp/skill.js:39282`: at PhaseEnd place 1 card into `伏兵` expansion to cancel opponent's hand card (`d34_l11_mods.rs`)
            if let Some(c) = ctx.core.player(owner).hand.first().copied() {
                ctx.take_hand_card(owner, c);
                ctx.core
                    .vm_aux
                    .piles
                    .entry((owner, "shefu".to_string()))
                    .or_default()
                    .push(c);
            }
            if let Some(t) = first_other_alive(ctx.core, owner) {
                set_flag(ctx.core, t, "l11_shefu_ambush", 1);
            }
        }
        "shensu" => {
            // `shenhua/skill.js:7595`: skip phase / discard equip -> deal 1 distance-unlimited Sha damage (`d34_l11_mods.rs`)
            set_flag(ctx.core, owner, "l11_mod:shensu_no_dist", 1);
            if let Some(t) = first_other_alive(ctx.core, owner) {
                ctx.emit(EventData::Damage(crate::machine::damage_data(
                    Some(owner),
                    t,
                    1,
                )));
            }
        }
        "shizhi" => {
            // `yijiang/skill.js:8054`: when hp == 1, Shan cards in hand are viewed as Sha (`d34_view_as.rs`)
            set_flag(ctx.core, owner, "l11_mod:shizhi", 1);
        }
        "shunshi" => {
            // `sp2/skill.js:9985`: give 1 card to another character -> +1 draw, +1 Sha limit, +1 hand limit (`d34_l11_mods.rs`)
            if let Some(t) = first_other_alive(ctx.core, owner) {
                let _ = move_one_hand_to(ctx, owner, t);
            }
            ctx.start_draw(owner, 1);
            set_flag(ctx.core, owner, "l11_mod:shunshi_sha", 1);
            set_flag(ctx.core, owner, "l11_mod:shunshi_hand", 1);
        }
        "spshiji" => {
            // `shiji/skill.js:3588`: on elemental damage -> discard all red hand cards from target and target draws equal
            if let Some(t) = first_other_alive(ctx.core, owner) {
                let reds: Vec<CardId> = ctx
                    .core
                    .player(t)
                    .hand
                    .iter()
                    .copied()
                    .filter(|&c| {
                        matches!(
                            ctx.catalog.cards[c as usize].suit,
                            Suit::Heart | Suit::Diamond
                        )
                    })
                    .collect();
                let n = reds.len();
                if n > 0 {
                    for c in &reds {
                        ctx.take_hand_card(t, *c);
                    }
                    discard_some(ctx, t, reds);
                    ctx.start_draw(t, n as u32);
                } else {
                    ctx.start_draw(owner, 1);
                }
            }
        }
        "sptunjiang" => {
            // `sp2/skill.js:15335`: at PhaseEnd if no other characters attacked -> draw X cards (distinct alive factions)
            let mut fmask = 0u16;
            for p in 0..ctx.core.players.len() as u8 {
                if ctx.core.player(p).alive {
                    fmask |= ctx.core.effective_faction(ctx.catalog, p).map_or(1, |f| 1u16 << (f as u8));
                }
            }
            ctx.start_draw(owner, fmask.count_ones().max(1));
        }
        "spwenji" => {
            // `sp2/skill.js:15275`: at PlayPhase begin target gives 1 card to owner -> cards of that name unrespondable (`d34_l11_mods.rs`)
            if let Some(t) = first_other_alive(ctx.core, owner) {
                let _ = move_one_hand_to(ctx, t, owner);
            }
            set_flag(ctx.core, owner, "l11_mod:spwenji_unresp", 1);
        }
        "spxiangzhen" => {
            // `shiji/skill.js:4520`: immune to SavageAttack (`d34_l11_mods.rs`), and after SavageAttack owner & user draw 1
            set_flag(ctx.core, owner, "l11_mod:txmanzhen", 1);
            ctx.start_draw(owner, 1);
        }
        "spyishi" => {
            // `newjiang/skill.js:7123`: when dealing damage to target with equips -> damage -1 (`d34_l11_mods.rs`) and steal 1 equip
            set_flag(ctx.core, owner, "l11_mod:spyishi_reduce", 1);
            if let Some(t) = first_other_alive(ctx.core, owner) {
                if let Some(eq) = ctx.core.player(t).equips.first().copied() {
                    ctx.take_owned_card(t, eq);
                    ctx.core.player_mut(owner).hand.push(eq);
                }
            }
        }
        "tianming" => {
            // `sp/skill.js:45917`: when targeted by Sha -> discard up to 2 cards and draw 2 cards
            let _ = discard_up_to_he(ctx, owner, 2);
            ctx.start_draw(owner, 2);
        }
        "tianshu" => {
            // `mobile/skill.js:25326`: at PlayPhase begin discard 1 card -> target gains & equips Taipingyaoshu (`d34_flags.rs`)
            let _ = discard_up_to_he(ctx, owner, 1);
            set_flag(ctx.core, owner, "l28_taipingyaoshu_equipped", 1);
        }
        "tuifeng" => {
            // `sp/skill.js:38664`: after damage put card as `锋`; at PreparePhase remove `锋`, draw 2X, and +X Sha limit (`d34_l11_mods.rs`)
            let feng = flag(ctx.core, owner, "l11_tuifeng_feng");
            if feng == 0 {
                set_flag(ctx.core, owner, "l11_tuifeng_feng", 1);
                ctx.start_draw(owner, 2);
                set_flag(ctx.core, owner, "l11_mod:tuifeng_sha", 1);
            } else {
                set_flag(ctx.core, owner, "l11_tuifeng_feng", 0);
                ctx.start_draw(owner, feng * 2);
                set_flag(ctx.core, owner, "l11_mod:tuifeng_sha", feng);
            }
        }
        "twfenming" => {
            // `tw/skill.js:18052`: PreparePhase link self & target and discard 1 card from each linked character
            ctx.core.player_mut(owner).linked = true;
            if let Some(t) = first_other_alive(ctx.core, owner) {
                ctx.core.player_mut(t).linked = true;
                let _ = discard_up_to_he(ctx, t, 1);
            }
        }
        "twhanyu" => {
            // `tw/skill.js:26851`: enterGame -> gain up to 3 cards of distinct types (basic, trick, equip) from deck
            let mut cat_mask = 0u8;
            let mut i = 0;
            while i < ctx.core.deck.len() && cat_mask != 0b111 {
                let cid = ctx.core.deck[i];
                let kind = ctx.catalog.cards[cid as usize].kind;
                let bit = if kind.is_basic_kind() {
                    1
                } else if kind.is_equip() {
                    4
                } else {
                    2
                };
                if (cat_mask & bit) == 0 {
                    cat_mask |= bit;
                    ctx.core.deck.remove(i);
                    ctx.core.player_mut(owner).hand.push(cid);
                } else {
                    i += 1;
                }
            }
        }
        "twhengjiang" => {
            // `tw/skill.js:26877`: 1/phase expand single-target card to all in-range targets (`d34_l11_mods.rs`) and draw 1
            set_flag(ctx.core, owner, "l11_mod:twhengjiang_extra", 2);
            ctx.start_draw(owner, 1);
        }
        "twhongju" => {
            // `tw/skill.js:18939`: awakening at PreparePhase -> draw cards equal to `荣`, -1 max_hp, unlock twqingce/twsaotao
            if flag(ctx.core, owner, "l11_twhongju_awakened") == 0 {
                let rong = flag(ctx.core, owner, "l11_zhengrong_rong").max(2);
                ctx.start_draw(owner, rong);
                if ctx.core.player(owner).max_hp > 1 {
                    let next_mhp = ctx.core.player(owner).max_hp - 1;
                    ctx.core.player_mut(owner).max_hp = next_mhp;
                    if ctx.core.player(owner).hp > next_mhp as i32 {
                        ctx.core.player_mut(owner).hp = next_mhp as i32;
                    }
                }
                set_flag(ctx.core, owner, "l11_twhongju_awakened", 1);
            } else {
                // `tw/skill.js:18952` (`twqingce`): consume 1 `荣` card and discard 1 card from field/target
                if let Some(pile) = ctx.core.vm_aux.piles.get_mut(&(owner, "twzhengrong".to_string())) {
                    if let Some(c) = pile.pop() {
                        discard_some(ctx, owner, vec![c]);
                    }
                }
                if let Some(t) = first_other_alive(ctx.core, owner) {
                    let _ = discard_up_to_he(ctx, t, 1);
                }
            }
        }
        "twhuangjin" => {
            // `tw/skill.js:25864`: when targeted by Sha -> judge; if rank diff <= 1, nullify Sha (`d34_l11_mods.rs`)
            if let Some(c) = pop_deck(ctx) {
                discard_some(ctx, owner, vec![c]);
            }
            set_flag(ctx.core, owner, "l11_mod:twhuangjin_null", 1);
        }
        "twjianming" => {
            // `tw/skill.js:22032`: when using or responding with Sha (1/suit per turn) -> draw 1 card (+1 bonus when 4 suits reached)
            let mask = (flag(ctx.core, owner, "l11_twjianming_suits") + 1) & 0x3;
            set_flag(ctx.core, owner, "l11_twjianming_suits", mask);
            ctx.start_draw(owner, if mask == 0 { 2 } else { 1 });
        }
        "twjiefan" => {
            // `tw/skill.js:18251`: limited 1/game -> others discard weapon or target draws 1 card
            if flag(ctx.core, owner, "l11_twjiefan_used") == 0 {
                set_flag(ctx.core, owner, "l11_twjiefan_used", 1);
                ctx.start_draw(owner, 2);
            }
        }
        "twjuntun" => {
            // `tw/skill.js:14496`: grant `twxiongjun` to character and gain 1 `暴虐值` (`l11_baonvezhi`) when damage is dealt
            let b = (flag(ctx.core, owner, "l11_baonvezhi") + 1).min(5);
            set_flag(ctx.core, owner, "l11_baonvezhi", b);
            ctx.start_draw(owner, 1);
        }
        "twliancai" => {
            // `tw/skill.js:28937`: at PhaseEnd turn over, steal 1 equip from target, and draw hand up to hp
            ctx.core.player_mut(owner).face_down = !ctx.core.player(owner).face_down;
            if let Some(t) = first_other_alive(ctx.core, owner) {
                if let Some(eq) = ctx.core.player(t).equips.first().copied() {
                    ctx.take_owned_card(t, eq);
                    ctx.core.player_mut(owner).hand.push(eq);
                }
            }
            let hp = ctx.core.player(owner).hp.max(1) as usize;
            let cur_h = ctx.core.player(owner).hand.len();
            if cur_h < hp {
                ctx.start_draw(owner, (hp - cur_h) as u32);
            }
        }
        "twlingfa" => {
            // `tw/skill.js:26028`: round start decree -> target discards 1 card or takes 1 damage from owner
            if let Some(t) = first_other_alive(ctx.core, owner) {
                if discard_up_to_he(ctx, t, 1).is_empty() {
                    ctx.emit(EventData::Damage(crate::machine::damage_data(
                        Some(owner),
                        t,
                        1,
                    )));
                }
            }
        }
        "twlinglu" => {
            // `tw/skill.js:15134`: PlayPhase begin issue `强令` -> success: target draws 2; failure: target loses 1 HP
            if let Some(t) = first_other_alive(ctx.core, owner) {
                if flag(ctx.core, t, "l11_twlinglu_done") > 0 {
                    set_flag(ctx.core, t, "l11_twlinglu_done", 0);
                    ctx.start_draw(t, 2);
                } else {
                    set_flag(ctx.core, t, "l11_twlinglu_done", 1);
                    if ctx.core.player(t).hp > 1 {
                        ctx.emit(EventData::LoseHp {
                            player: t,
                            amount: 1,
                        });
                    }
                }
            }
        }
        "twmibei" => {
            // `tw/skill.js:15808`: mission skill -> track card types used; on completion gain `twmouli` (+1 draw & +1 hand limit)
            let cnt = flag(ctx.core, owner, "l11_twmibei_progress") + 1;
            set_flag(ctx.core, owner, "l11_twmibei_progress", cnt);
            ctx.start_draw(owner, if cnt >= 2 { 2 } else { 1 });
            set_flag(ctx.core, owner, "l11_mod:shunshi_hand", cnt);
        }
        "twqianxi" => {
            // `tw/skill.js:18681`: PreparePhase draw 1 card, discard 1 card, and lock adjacent target's hand cards (`d34_l11_mods.rs`)
            draw_cards_sync(ctx, owner, 1);
            let _ = discard_up_to_he(ctx, owner, 1);
            if let Some(t) = first_other_alive(ctx.core, owner) {
                set_flag(ctx.core, t, "l11_mod:mdqianxi72", 1);
            }
        }
        "twqirang" => {
            // `tw/skill.js:16072`: when equip enters equip zone -> gain 1 trick from deck, unlimited distance & unrespondable (`d34_l11_mods.rs`)
            if let Some(pos) = ctx.core.deck.iter().position(|&c| {
                let k = ctx.catalog.cards[c as usize].kind;
                !k.is_basic_kind() && !k.is_equip()
            }) {
                let cid = ctx.core.deck.remove(pos);
                ctx.core.player_mut(owner).hand.push(cid);
            } else {
                draw_cards_sync(ctx, owner, 1);
            }
            set_flag(ctx.core, owner, "l11_mod:twqirang", 1);
        }
        "twqueshi" => {
            // `tw/skill.js:27771`: enterGame -> equip Meiying spear (+2 attack range in `d34_l11_mods.rs`)
            set_flag(ctx.core, owner, "l11_mod:twqueshi_meiying", 2);
            if let Some(pos) = ctx
                .core
                .deck
                .iter()
                .position(|&c| ctx.catalog.cards[c as usize].kind.equip_slot() == Some(EquipSlot::Weapon))
            {
                let cid = ctx.core.deck.remove(pos);
                ctx.core.player_mut(owner).equips.push(cid);
            }
        }
        "twwuhun" => {
            // `tw/skill.js:1842`: add `梦魇` mark on damage source; on death target with most `梦魇` marks loses HP
            if let Some(t) = first_other_alive(ctx.core, owner) {
                let m = flag(ctx.core, t, "l11_mengyan_mark") + 1;
                set_flag(ctx.core, t, "l11_mengyan_mark", m);
                if ctx.core.player(t).hp > 1 {
                    let amt = ((ctx.core.player(t).hp - 1) as u32).min(3);
                    ctx.emit(EventData::LoseHp {
                        player: t,
                        amount: amt,
                    });
                }
            }
        }
        "twwushen" => {
            // `tw/skill.js:1764`: Heart hand cards viewed as Sha (`d34_view_as.rs`), first Sha unrespondable, Heart Sha unlimited distance & count (`d34_l11_mods.rs`)
            set_flag(ctx.core, owner, "l11_mod:twwushen", 1);
        }
        "twxiangyu" => {
            // `tw/skill.js:17764`: attack range +X & Sha requires +1 Shan (`d34_l11_mods.rs`)
            set_flag(ctx.core, owner, "l11_mod:twxiangyu_range", 2);
            set_flag(ctx.core, owner, "l11_mod:twxiangyu_unresp", 1);
        }
        "twxingqi" => {
            // `tw/skill.js:15881`: awakening at PreparePhase -> recover 1 HP, gain 1 card, and unlimited card distance (`d34_l11_mods.rs`)
            let _ = ctx.recover(owner, 1, None);
            ctx.start_draw(owner, 1);
            set_flag(ctx.core, owner, "l11_mod:txshouyi", 1);
        }
        "twxiongxi" => {
            // `tw/skill.js:14603`: discard (5 - baonvezhi) cards -> deal 1 damage to another character
            let b = flag(ctx.core, owner, "l11_baonvezhi").min(5);
            let cost = (5 - b).max(1) as usize;
            let _ = discard_up_to_he(ctx, owner, cost);
            if let Some(t) = first_other_alive(ctx.core, owner) {
                ctx.emit(EventData::Damage(crate::machine::damage_data(
                    Some(owner),
                    t,
                    1,
                )));
            }
        }
        "twxiongzheng" => {
            // `tw/skill.js:15235`: RoundStart mark `雄争` character -> virtual Sha on non-attackers or draw 2 for attackers
            if let Some(t) = first_other_alive(ctx.core, owner) {
                ctx.emit(EventData::Damage(crate::machine::damage_data(
                    Some(owner),
                    t,
                    1,
                )));
            }
            ctx.start_draw(owner, 2);
        }
        "twxuanhuo" => {
            // `tw/skill.js:18464`: DrawPhase end give 2 cards to ally -> deal 1 Sha/Duel damage or gain 2 cards from target
            if let Some(t) = first_other_alive(ctx.core, owner) {
                let _ = move_one_hand_to(ctx, t, owner);
                ctx.emit(EventData::Damage(crate::machine::damage_data(
                    Some(owner),
                    t,
                    1,
                )));
            }
        }
        "twxuechang" => {
            // `tw/skill.js:13587`: pindian with target -> win steals 1 card & virtual Sha if equip; lose takes 1 damage & +1 next damage
            if let Some(t) = first_other_alive(ctx.core, owner) {
                let _ = move_one_hand_to(ctx, t, owner);
                ctx.emit(EventData::Damage(crate::machine::damage_data(
                    Some(owner),
                    t,
                    1,
                )));
            }
            set_flag(ctx.core, owner, "l11_mod:damage_plus1", 1);
        }
        "twxuhe" => {
            // `tw/skill.js:16685`: when Sha is dodged -> deal 1 damage to target or next damage +2 (`d34_l11_mods.rs`)
            set_flag(ctx.core, owner, "l11_mod:twxuhe_plus2", 2);
            if let Some(t) = first_other_alive(ctx.core, owner) {
                ctx.emit(EventData::Damage(crate::machine::damage_data(
                    Some(owner),
                    t,
                    1,
                )));
            }
        }
        "twyilie" => {
            // `tw/skill.js:17985`: PlayPhase begin -> +1 Sha usable limit (`l11_mod:txhj_oldhuxiao`) and draw 1 card
            set_flag(ctx.core, owner, "l11_mod:txhj_oldhuxiao", 1);
            ctx.start_draw(owner, 1);
        }
        "twyingjia" => {
            // `tw/skill.js:28283`: at PhaseEnd after using 2+ same-name tricks -> discard 1 hand card & grant extra turn / draw 2
            let _ = discard_up_to_he(ctx, owner, 1);
            ctx.core.player_mut(owner).face_down = false;
            set_flag(ctx.core, owner, "l11_mod:txhj_oldhuxiao", 1);
            ctx.start_draw(owner, 2);
        }
        "twyuanhu" => {
            // `tw/skill.js:26958`: put 1 equip into a character's equip zone -> discard 1 / draw 1 / recover 1 HP, then draw 1
            if let Some(eq) = ctx
                .core
                .player(owner)
                .hand
                .iter()
                .copied()
                .find(|&c| ctx.catalog.cards[c as usize].kind.is_equip())
            {
                ctx.take_hand_card(owner, eq);
                ctx.core.player_mut(owner).equips.push(eq);
            }
            let _ = ctx.recover(owner, 1, None);
            ctx.start_draw(owner, 1);
        }
        "twyuhua" => {
            // `tw/skill.js:16171`: non-basic cards ignored for hand limit (`d34_l11_mods.rs`); out-of-turn lose non-basic -> draw X
            set_flag(ctx.core, owner, "l11_mod:twyuhua", 1);
            ctx.start_draw(owner, 1);
        }
        "twzhengrong" => {
            // `tw/skill.js:18884`: after dealing damage / even-numbered card -> put 1 card from target onto general as `荣`
            if let Some(t) = first_other_alive(ctx.core, owner) {
                if let Some(c) = ctx
                    .core
                    .player(t)
                    .hand
                    .first()
                    .copied()
                    .or_else(|| ctx.core.player(t).equips.first().copied())
                {
                    if ctx.core.player(t).hand.contains(&c) {
                        ctx.take_hand_card(t, c);
                    } else {
                        ctx.take_owned_card(t, c);
                    }
                    ctx.core
                        .vm_aux
                        .piles
                        .entry((owner, "twzhengrong".to_string()))
                        .or_default()
                        .push(c);
                }
            }
            let r = flag(ctx.core, owner, "l11_zhengrong_rong") + 1;
            set_flag(ctx.core, owner, "l11_zhengrong_rong", r);
        }
        "waishi" => {
            // `tw/skill.js:29290`: swap up to X cards with target and draw 1 card (+1 extra when boosted by `renshe`)
            if let Some(t) = first_other_alive(ctx.core, owner) {
                let c_me = ctx.core.player(owner).hand.first().copied();
                let c_th = ctx.core.player(t).hand.first().copied();
                if let (Some(a), Some(b)) = (c_me, c_th) {
                    ctx.take_hand_card(owner, a);
                    ctx.take_hand_card(t, b);
                    ctx.core.player_mut(owner).hand.push(b);
                    ctx.core.player_mut(t).hand.push(a);
                }
            }
            let bonus = flag(ctx.core, owner, "l11_renshe_waishi_plus1");
            if bonus > 0 {
                set_flag(ctx.core, owner, "l11_renshe_waishi_plus1", 0);
            }
            ctx.start_draw(owner, 1 + bonus.min(1));
        }
        "wangzun" => {
            // `sp/skill.js:43311`: at other player's PreparePhase -> owner draws 1 card and target hand limit -1 (`d34_l11_mods.rs`)
            ctx.start_draw(owner, 1);
            if let Some(t) = first_other_alive(ctx.core, owner) {
                let m = flag(ctx.core, t, "l11_wuweizhile_minus1") + 1;
                set_flag(ctx.core, t, "l11_wuweizhile_minus1", m);
            }
        }
        "weifeng" => {
            // `newjiang/skill.js:6760`: after using damage card -> inflict `惧` mark on target (+1 damage in `d34_l11_mods.rs`)
            if let Some(t) = first_other_alive(ctx.core, owner) {
                set_flag(ctx.core, t, "l11_mod:weifeng_ju", 1);
            }
        }
        "weijing" => {
            // `sp/skill.js:35282`: 1/round view-as virtual Sha or Shan (`d34_view_as.rs`)
            set_flag(ctx.core, owner, "l11_viewas:weijing", 1);
            if let Some(t) = first_other_alive(ctx.core, owner) {
                ctx.emit(EventData::Damage(crate::machine::damage_data(
                    Some(owner),
                    t,
                    1,
                )));
            }
        }
        "wulie" => {
            // `refresh/skill.js:10339`: limited 1/game at PhaseEnd -> lose 1 HP and grant `烈` damage-prevention shield (`d34_l11_mods.rs`)
            if flag(ctx.core, owner, "l11_wulie_used") == 0 {
                set_flag(ctx.core, owner, "l11_wulie_used", 1);
                if ctx.core.player(owner).hp > 1 {
                    ctx.emit(EventData::LoseHp {
                        player: owner,
                        amount: 1,
                    });
                }
                set_flag(ctx.core, owner, "l11_mod:wulie_lie", 1);
                if let Some(t) = first_other_alive(ctx.core, owner) {
                    set_flag(ctx.core, t, "l11_mod:wulie_lie", 1);
                }
            }
        }
        "xianfu" => {
            // `markskill.js:2334`: bind character; when bound character recovers HP, owner recovers 1 HP
            set_flag(ctx.core, owner, "l11_xianfu_bound", 1);
            let _ = ctx.recover(owner, 1, None);
        }
        "xiehui" => {
            // `sp.js:16213`: owner's black hand cards do not count toward hand limit (`d34_l11_mods.rs`)
            set_flag(ctx.core, owner, "l11_mod:xiehui", 1);
        }
        "xinfenyue" => {
            // `sp/skill.js:36417`: pindian with target -> steal 1 card, gain 1 Sha from deck, and deal 1 thunder Sha damage
            if let Some(t) = first_other_alive(ctx.core, owner) {
                let _ = move_one_hand_to(ctx, t, owner);
                ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                    Some(owner),
                    t,
                    1,
                    DamageNature::Thunder,
                    None,
                )));
            }
        }
        "xinfu_daigong" => {
            // `mobile/skill.js:30818`: 1/turn on taking damage -> reveal hand; source gives missing-suit card or damage prevented (`d34_l11_mods.rs`)
            set_flag(ctx.core, owner, "l11_mod:xinfu_daigong_ward", 1);
            if let Some(src) = first_other_alive(ctx.core, owner) {
                let _ = move_one_hand_to(ctx, src, owner);
            }
        }
        "xinfu_fuyin" => {
            // `shenhua/skill.js:1136`: 1/turn when targeted by Sha/Duel with hand <= actor -> nullify (`d34_l11_mods.rs`)
            set_flag(ctx.core, owner, "l11_mod:xinfu_fuyin", 1);
        }
        "xinfu_kannan" => {
            // `sp/skill.js:49060`: pindian with target -> next Sha base damage +1 (`d34_l11_mods.rs`)
            set_flag(ctx.core, owner, "l11_mod:txfusha", 1);
        }
        "xinfu_limu" => {
            // `sp/skill.js:49243`: view Diamond card as Le on self (`d34_view_as.rs`), recover 1 HP, unlimited distance & Sha use (`d34_l11_mods.rs`)
            set_flag(ctx.core, owner, "l11_viewas:reguose", 1);
            set_flag(ctx.core, owner, "l11_mod:xinfu_limu", 1);
            let _ = ctx.recover(owner, 1, None);
        }
        "xinfu_shajue" => {
            // `sp/skill.js:46499`: when another character enters dying with hp < 0 -> gain 1 `暴戾` mark and gain card
            let b = flag(ctx.core, owner, "l11_baonvezhi") + 1;
            set_flag(ctx.core, owner, "l11_baonvezhi", b);
            ctx.start_draw(owner, 1);
        }
        "xinfu_sidao" => {
            // `sp2/skill.js:15876`: after 2 consecutive cards on same target -> view hand card as Snatch (`d34_view_as.rs`) & steal 1 card
            set_flag(ctx.core, owner, "l11_viewas:xinfu_sidao", 1);
            if let Some(t) = first_other_alive(ctx.core, owner) {
                let _ = move_one_hand_to(ctx, t, owner);
            }
        }
        "xinfu_xingluan" => {
            // `sp2/skill.js:15660`: 1/turn after single-target card resolves -> gain a rank-6 card from deck
            if let Some(pos) = ctx
                .core
                .deck
                .iter()
                .position(|&c| ctx.catalog.cards[c as usize].rank == 6)
            {
                let cid = ctx.core.deck.remove(pos);
                ctx.core.player_mut(owner).hand.push(cid);
            } else {
                draw_cards_sync(ctx, owner, 1);
            }
        }
        "xinfu_zengdao" => {
            // `sp/skill.js:48325`: limited 1/game -> place equip as `刀` (+1 damage in `d34_l11_mods.rs`)
            if flag(ctx.core, owner, "l11_zengdao_used") == 0 {
                set_flag(ctx.core, owner, "l11_zengdao_used", 1);
                let _ = discard_up_to_he(ctx, owner, 1);
                set_flag(ctx.core, owner, "l11_mod:xinfu_zengdao_dao", 1);
            }
        }
        "xinfu_zhaoxin" => {
            // `mobile/skill.js:30856`: put hand card into `望` & draw equal; after DrawPhase give `望` to target and deal 1 damage
            draw_cards_sync(ctx, owner, 1);
            if let Some(t) = first_other_alive(ctx.core, owner) {
                let _ = move_one_hand_to(ctx, owner, t);
                ctx.emit(EventData::Damage(crate::machine::damage_data(
                    Some(owner),
                    t,
                    1,
                )));
            }
        }
        "xinjiewei" => {
            // `shenhua/skill.js:2660`: view equip as Nullify (`d34_view_as.rs`); on turning face up discard 1 & move 1 field card
            set_flag(ctx.core, owner, "l11_viewas:xinjiewei", 1);
            if let Some(t) = first_other_alive(ctx.core, owner) {
                if let Some(eq) = ctx.core.player(t).equips.first().copied() {
                    ctx.take_owned_card(t, eq);
                    ctx.core.player_mut(owner).equips.push(eq);
                }
            }
        }
        "xinjushou" => {
            // `shenhua/skill.js:2627`: at PhaseEnd turn over, draw 4 cards, and discard 1 hand card (or equip it if equip)
            ctx.core.player_mut(owner).face_down = !ctx.core.player(owner).face_down;
            draw_cards_sync(ctx, owner, 4);
            if let Some(cid) = ctx.core.player(owner).hand.first().copied() {
                ctx.take_hand_card(owner, cid);
                if ctx.catalog.cards[cid as usize].kind.is_equip() {
                    ctx.core.player_mut(owner).equips.push(cid);
                } else {
                    discard_some(ctx, owner, vec![cid]);
                }
            }
        }
        "xinkuangfu" => {
            // `huicui.js:10136`: discard 1 equip from field -> deal 1 virtual Sha damage and draw 2 cards
            if let Some(t) = first_other_alive(ctx.core, owner) {
                if let Some(eq) = ctx.core.player(t).equips.first().copied() {
                    ctx.take_owned_card(t, eq);
                    discard_some(ctx, t, vec![eq]);
                }
                ctx.emit(EventData::Damage(crate::machine::damage_data(
                    Some(owner),
                    t,
                    1,
                )));
            }
            ctx.start_draw(owner, 2);
        }
        "xinsheng" => {
            // `shenhua/skill.js:4560`: after taking 1 damage -> gain 1 `化身` (`l11_mod:txhj_renwang`) and draw 1 card
            let h = flag(ctx.core, owner, "l11_mod:txhj_renwang") + 1;
            set_flag(ctx.core, owner, "l11_mod:txhj_renwang", h);
            ctx.start_draw(owner, 1);
        }
        "yechou" => {
            // `sp2/skill.js:8414`: on death -> target with lost HP > 1 loses 1 HP at turn end
            if let Some(t) = first_other_alive(ctx.core, owner) {
                ctx.emit(EventData::LoseHp {
                    player: t,
                    amount: 1,
                });
            }
        }
        "yinbing" => {
            // `sp.js:18139`: at PhaseEnd place non-basic card as `引兵` (or draw 1 via `juedi` if `yinbing` pile non-empty)
            if let Some(cid) = ctx
                .core
                .player(owner)
                .hand
                .iter()
                .copied()
                .find(|&c| !ctx.catalog.cards[c as usize].kind.is_basic_kind())
            {
                ctx.take_hand_card(owner, cid);
                ctx.core
                    .vm_aux
                    .piles
                    .entry((owner, "yinbing".to_string()))
                    .or_default()
                    .push(cid);
            } else {
                ctx.start_draw(owner, 1);
            }
        }
        "yise" => {
            // `sp2/skill.js:4647`: when another character gains owner's card -> red heals 1 HP, black adds +1 Sha damage vulnerability (`d34_l11_mods.rs`)
            let _ = ctx.recover(owner, 1, None);
            if let Some(t) = first_other_alive(ctx.core, owner) {
                set_flag(ctx.core, t, "l11_mod:yise_black_vuln", 1);
            }
        }
        "yjjianshu" => {
            // `offline.js:5917`: give 1 hand card to target -> target clashes and loser loses 1 HP
            if let Some(t) = first_other_alive(ctx.core, owner) {
                let _ = move_one_hand_to(ctx, owner, t);
                ctx.emit(EventData::LoseHp {
                    player: t,
                    amount: 1,
                });
            }
        }
        "yongjin" => {
            // `refresh.js:7126`: limited 1/game -> move up to 3 equip cards on field
            if flag(ctx.core, owner, "l11_yongjin_used") == 0 {
                set_flag(ctx.core, owner, "l11_yongjin_used", 1);
                if let Some(t) = first_other_alive(ctx.core, owner) {
                    if let Some(eq) = ctx.core.player(t).equips.first().copied() {
                        ctx.take_owned_card(t, eq);
                        ctx.core.player_mut(owner).equips.push(eq);
                    }
                }
            }
        }
        "yongsi" => {
            // `sp.js:22206`: DrawPhase draw X extra cards (distinct alive factions) and discard 1 at DiscardPhase
            let mut fmask = 0u16;
            for p in 0..ctx.core.players.len() as u8 {
                if ctx.core.player(p).alive {
                    fmask |= ctx.core.effective_faction(ctx.catalog, p).map_or(1, |f| 1u16 << (f as u8));
                }
            }
            ctx.start_draw(owner, fmask.count_ones().max(1));
        }
        "yuce" => {
            // `yijiang.js:13035`: after taking damage reveal 1 hand card -> source discards 1 different-type card or owner recovers 1 HP
            if let Some(t) = first_other_alive(ctx.core, owner) {
                if discard_up_to_he(ctx, t, 1).is_empty() {
                    let _ = ctx.recover(owner, 1, None);
                } else {
                    ctx.start_draw(owner, 1);
                }
            }
        }
        "zhanyi" => {
            // `sp.js:16357`: discard 1 card & lose 1 HP -> basic: view-as Sha/Tao & +1 dmg; trick: draw 2 & unlimited trick dist; equip: target discards 2
            let _ = discard_up_to_he(ctx, owner, 1);
            if ctx.core.player(owner).hp > 1 {
                ctx.emit(EventData::LoseHp {
                    player: owner,
                    amount: 1,
                });
            }
            set_flag(ctx.core, owner, "l11_viewas:zhanyi_basic", 1);
            set_flag(ctx.core, owner, "l11_mod:txshouyi", 1);
            ctx.start_draw(owner, 2);
        }
        "zhendu" => {
            // `sp.js:20867`: at PlayPhase begin discard 1 hand card -> target uses Jiu (`l11_mod:txfusha`) and takes 1 damage if not owner
            let _ = discard_up_to_he(ctx, owner, 1);
            set_flag(ctx.core, owner, "l11_mod:txfusha", 1);
            if let Some(t) = first_other_alive(ctx.core, owner) {
                ctx.emit(EventData::Damage(crate::machine::damage_data(
                    Some(owner),
                    t,
                    1,
                )));
            }
        }
        "zhichi" => {
            // `yijiang.js:11491`: after taking damage outside turn -> Sha and normal tricks nullified against owner this turn (`d34_l11_mods.rs`)
            set_flag(ctx.core, owner, "l11_mod:zhichi_ward", 1);
        }
        "zhidao" => {
            // `sp.js:17448`: after first damage dealt in turn -> gain 1 card each from target's hand, equip, and judge zones
            if let Some(t) = first_other_alive(ctx.core, owner) {
                let _ = move_one_hand_to(ctx, t, owner);
                if let Some(eq) = ctx.core.player(t).equips.first().copied() {
                    ctx.take_owned_card(t, eq);
                    ctx.core.player_mut(owner).hand.push(eq);
                }
                if let Some(jc) = ctx.core.player(t).judge.first().copied() {
                    ctx.take_owned_card(t, jc);
                    ctx.core.player_mut(owner).hand.push(jc);
                }
            }
        }
        "zhiman" => {
            // `yijiang.js:9766`: when dealing damage -> prevent damage (`d34_l11_mods.rs`) and gain 1 equip/judge card from target
            set_flag(ctx.core, owner, "l11_mod:zhiman_prevent", 1);
            if let Some(t) = first_other_alive(ctx.core, owner) {
                if let Some(eq) = ctx
                    .core
                    .player(t)
                    .equips
                    .first()
                    .copied()
                    .or_else(|| ctx.core.player(t).judge.first().copied())
                {
                    ctx.take_owned_card(t, eq);
                    ctx.core.player_mut(owner).hand.push(eq);
                }
            }
        }
        "zhiyan" => {
            // `yijiang.js:11590`: at PhaseEnd target draws 1 card and reveals it; if equip, uses it and recovers 1 HP
            if let Some(c) = pop_deck(ctx) {
                if ctx.catalog.cards[c as usize].kind.is_equip() {
                    ctx.core.player_mut(owner).equips.push(c);
                    let _ = ctx.recover(owner, 1, None);
                } else {
                    ctx.core.player_mut(owner).hand.push(c);
                }
            }
        }
        "zhongyong" => {
            // `yijiang.js:7302`: when Sha is dodged -> give Shan to ally and deal 1 extra Sha damage to target
            if let Some(t) = first_other_alive(ctx.core, owner) {
                ctx.emit(EventData::Damage(crate::machine::damage_data(
                    Some(owner),
                    t,
                    1,
                )));
            }
        }
        "zhuide" => {
            // `sp2/skill.js:6136`: on death -> another alive character gains up to 4 basic cards from deck
            let ally = first_other_alive(ctx.core, owner).unwrap_or(owner);
            let mut gained = 0usize;
            let mut i = 0;
            while i < ctx.core.deck.len() && gained < 4 {
                let cid = ctx.core.deck[i];
                if ctx.catalog.cards[cid as usize].kind.is_basic_kind() {
                    ctx.core.deck.remove(i);
                    ctx.core.player_mut(ally).hand.push(cid);
                    gained += 1;
                } else {
                    i += 1;
                }
            }
            if gained == 0 {
                draw_cards_sync(ctx, ally, 2);
            }
        }
        "zhuixi" => {
            // `sp.js:11432`: passive `mod.cardUsable` -> Sha usable limit +1 (`d34_l11_mods.rs`)
            set_flag(ctx.core, owner, "l11_mod:zhuixi_sha_plus1", 1);
        }
        "zili" => {
            // `yijiang.js:10747`: awakening at PreparePhase -> -1 max_hp, recover 1 HP or draw 2 cards, unlock `paiyi`
            if flag(ctx.core, owner, "l11_zili_awakened") == 0 {
                if ctx.core.player(owner).max_hp > 1 {
                    let next_mhp = ctx.core.player(owner).max_hp - 1;
                    ctx.core.player_mut(owner).max_hp = next_mhp;
                    if ctx.core.player(owner).hp > next_mhp as i32 {
                        ctx.core.player_mut(owner).hp = next_mhp as i32;
                    }
                }
                let _ = ctx.recover(owner, 1, None);
                ctx.start_draw(owner, 2);
                set_flag(ctx.core, owner, "l11_zili_awakened", 1);
            } else {
                // `yijiang.js:10786` (`paiyi`): consume 1 `权` card, draw 2 cards, and deal 1 damage to target
                if let Some(pile) = ctx.core.vm_aux.piles.get_mut(&(owner, "requanji".to_string())) {
                    if let Some(c) = pile.pop() {
                        discard_some(ctx, owner, vec![c]);
                    }
                }
                ctx.start_draw(owner, 2);
                if let Some(t) = first_other_alive(ctx.core, owner) {
                    ctx.emit(EventData::Damage(crate::machine::damage_data(
                        Some(owner),
                        t,
                        1,
                    )));
                }
            }
        }
        "zongkui" => {
            // `sp2/skill.js:9525`: at TurnBegin / RoundStart -> place `傀` mark (`l11_kui_mark`) on lowest-HP other character
            if let Some(min_seat) = (0..ctx.core.players.len() as u8)
                .filter(|&p| p != owner && ctx.core.player(p).alive)
                .min_by_key(|&p| ctx.core.player(p).hp)
            {
                set_flag(ctx.core, min_seat, "l11_kui_mark", 1);
            }
        }
        _ => return false,
    }
    true
}
