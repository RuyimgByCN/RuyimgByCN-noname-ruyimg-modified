//! Authentic upstream `L11` (`太虚幻境` `extension_character.js`) cross-package NPC skill resolvers
//! Part A (`aocai` ..= `rongbei`, 80 skills).
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

pub(super) fn try_resolve_l11_npc_skill_a(
    ctx: &mut Ctx<'_>,
    skill_name: &str,
    owner: PlayerId,
) -> bool {
    match skill_name {
        "aocai" => {
            // `sp.js:21826`: out-of-turn inspect top 2 (or 4 if hand empty) deck cards and gain a basic card
            let look_n = if ctx.core.player(owner).hand.is_empty() { 4 } else { 2 };
            let deck_len = ctx.core.deck.len();
            let start = deck_len.saturating_sub(look_n);
            if let Some(rel) = ctx.core.deck[start..]
                .iter()
                .rposition(|&c| ctx.catalog.cards[c as usize].kind.is_basic_kind())
            {
                let cid = ctx.core.deck.remove(start + rel);
                ctx.core.player_mut(owner).hand.push(cid);
            } else {
                draw_cards_sync(ctx, owner, 1);
            }
        }
        "baiyi" => {
            // `mobile.js:10960`: limited 1/game when wounded -> swap tactical hand positions of two other characters
            if flag(ctx.core, owner, "l11_baiyi_used") == 0 {
                set_flag(ctx.core, owner, "l11_baiyi_used", 1);
                let others: Vec<PlayerId> = (0..ctx.core.players.len() as u8)
                    .filter(|&p| p != owner && ctx.core.player(p).alive)
                    .collect();
                if others.len() >= 2 {
                    let (a, b) = (others[0], others[1]);
                    let ha = ctx.core.player(a).hand.clone();
                    let hb = ctx.core.player(b).hand.clone();
                    ctx.core.player_mut(a).hand = hb;
                    ctx.core.player_mut(b).hand = ha;
                } else if let Some(&t) = others.first() {
                    let _ = move_one_hand_to(ctx, t, owner);
                }
            }
        }
        "benyu" => {
            // `sp.js:17346`: after damage -> draw up to source hand count (cap 5), or discard & deal 1 damage to source
            if let Some(src) = first_other_alive(ctx.core, owner) {
                let my_h = ctx.core.player(owner).hand.len();
                let src_h = ctx.core.player(src).hand.len().min(5);
                if my_h < src_h {
                    draw_cards_sync(ctx, owner, (src_h - my_h) as u32);
                } else if !discard_up_to_he(ctx, owner, 1).is_empty() {
                    ctx.emit(EventData::Damage(crate::machine::damage_data(
                        Some(owner),
                        src,
                        1,
                    )));
                } else {
                    draw_cards_sync(ctx, owner, 1);
                }
            }
        }
        "bmcanshi" => {
            // `sp2.js:9417`: cancel single-target card from `傀`-marked actor or add `傀`-marked extra targets (`d34_l11_mods.rs`)
            set_flag(ctx.core, owner, "l11_mod:bmcanshi", 1);
            if let Some(t) = first_other_alive(ctx.core, owner) {
                set_flag(ctx.core, t, "l11_kui_mark", 1);
            }
        }
        "buqu" => {
            // `shenhua.js:7047`: on dying, place top card as `创` (`l11_mod:buqu_chuang` sets hand limit) and recover to 1 HP
            if let Some(c) = pop_deck(ctx) {
                ctx.core
                    .vm_aux
                    .piles
                    .entry((owner, "buqu".to_string()))
                    .or_default()
                    .push(c);
            }
            let chuang = ctx
                .core
                .vm_aux
                .piles
                .get(&(owner, "buqu".to_string()))
                .map_or(1, |v| v.len().max(1) as u32);
            set_flag(ctx.core, owner, "l11_mod:buqu_chuang", chuang);
            if ctx.core.player(owner).hp < 1 {
                ctx.core.player_mut(owner).hp = 1;
            }
            ctx.core.player_mut(owner).dying = false;
        }
        "chenqing" => {
            // `sp.js:18988`: on dying, another character draws 4, discards 4; if 4 suits or rescued, dying player recovers 1 HP
            let helper = first_other_alive(ctx.core, owner).unwrap_or(owner);
            draw_cards_sync(ctx, helper, 4);
            let _ = discard_up_to_he(ctx, helper, 4);
            let _ = ctx.recover(owner, 1, Some(u32::from(helper)));
            ctx.core.player_mut(owner).dying = false;
        }
        "chouce" => {
            // `sp.js:14136`: after taking damage, judge top card -> black: discard 1 from target; red: draw 1 (or 2 if xianfu)
            let is_red = pop_deck(ctx).is_some_and(|c| {
                let red = matches!(
                    ctx.catalog.cards[c as usize].suit,
                    Suit::Heart | Suit::Diamond
                );
                discard_some(ctx, owner, vec![c]);
                red
            });
            if is_red {
                let bonus = if flag(ctx.core, owner, "l11_xianfu_bound") > 0 {
                    2
                } else {
                    1
                };
                ctx.start_draw(owner, bonus);
            } else if let Some(t) = first_other_alive(ctx.core, owner) {
                if discard_up_to_he(ctx, t, 1).is_empty() {
                    ctx.start_draw(owner, 1);
                }
            }
        }
        "dclbjiuxian" => {
            // `collab.js:2492`: view multi-target trick as `Jiu` (`d34_view_as.rs`) & unlimited Jiu (`l11_mod:txfusha`)
            set_flag(ctx.core, owner, "l11_viewas:dclbjiuxian", 1);
            set_flag(ctx.core, owner, "l11_mod:txfusha", 1);
        }
        "dcmoukui" => {
            // `sp2.js:1638`: on Sha target -> draw 1 card and discard 1 card from target
            ctx.start_draw(owner, 1);
            if let Some(t) = first_other_alive(ctx.core, owner) {
                let _ = discard_up_to_he(ctx, t, 1);
            }
        }
        "dcnutao" => {
            // `collab.js:3112`: trick targeting others deals 1 thunder damage; thunder damage in PlayPhase gives +1 Sha limit
            if let Some(t) = first_other_alive(ctx.core, owner) {
                ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                    Some(owner),
                    t,
                    1,
                    DamageNature::Thunder,
                    None,
                )));
            }
            let cur = flag(ctx.core, owner, "l11_mod:dcnutao_sha_plus") + 1;
            set_flag(ctx.core, owner, "l11_mod:dcnutao_sha_plus", cur);
        }
        "diancai" => {
            // `xianding/skill.js:28675`: at phase end after losing cards -> draw hand up to max_hp
            let mhp = ctx.core.player(owner).max_hp as usize;
            let cur_h = ctx.core.player(owner).hand.len();
            let diff = mhp.saturating_sub(cur_h).max(1) as u32;
            ctx.start_draw(owner, diff);
        }
        "dualside" => {
            // `game.js:34262`: dual-sided character flips to back/front form on turnOver or dying, restoring HP & drawing 2
            ctx.core.player_mut(owner).face_down = !ctx.core.player(owner).face_down;
            let mhp = ctx.core.player(owner).max_hp as i32;
            ctx.core.player_mut(owner).hp = mhp;
            ctx.core.player_mut(owner).dying = false;
            draw_cards_sync(ctx, owner, 2);
        }
        "duwu" => {
            // `sp.js:22122`: discard cards to deal 1 damage to target in range; if target enters dying, lose 1 HP
            let _ = discard_up_to_he(ctx, owner, 1);
            if let Some(t) = first_other_alive(ctx.core, owner) {
                ctx.emit(EventData::Damage(crate::machine::damage_data(
                    Some(owner),
                    t,
                    1,
                )));
                if ctx.core.player(t).hp <= 1 && ctx.core.player(owner).hp > 1 {
                    ctx.emit(EventData::LoseHp {
                        player: owner,
                        amount: 1,
                    });
                }
            }
        }
        "fenji" => {
            // `shenhua.js:7094`: when a character loses hand card -> lose 1 HP and target draws 2 cards
            if ctx.core.player(owner).hp > 1 {
                ctx.emit(EventData::LoseHp {
                    player: owner,
                    amount: 1,
                });
            }
            ctx.start_draw(owner, 2);
        }
        "fenxun" => {
            // `sp.js:15556`: discard 1 card -> distance to target treated as 1 this turn (`d34_l11_mods.rs`)
            let _ = discard_up_to_he(ctx, owner, 1);
            set_flag(ctx.core, owner, "l11_mod:fenxun_dist1", 1);
        }
        "gaoyuan" => {
            // `mobile.js:11997`: when targeted by Sha, discard 1 card to deflect Sha (`d34_l11_mods.rs`)
            if !discard_up_to_he(ctx, owner, 1).is_empty() {
                set_flag(ctx.core, owner, "l11_mod:gaoyuan_deflect", 1);
            }
        }
        "gnsheque" => {
            // `mobile.js:16944`: at other player's prepare phase if they have equips -> ignore armor & deal 1 Sha damage
            set_flag(ctx.core, owner, "l11_mod:gnsheque_pierce", 1);
            if let Some(t) = first_other_alive(ctx.core, owner) {
                ctx.emit(EventData::Damage(crate::machine::damage_data(
                    Some(owner),
                    t,
                    1,
                )));
            }
        }
        "guanzong" => {
            // `mobile.js:6700`: 1/phase cause one other character to deal 1 damage to another character
            let others: Vec<PlayerId> = (0..ctx.core.players.len() as u8)
                .filter(|&p| p != owner && ctx.core.player(p).alive)
                .collect();
            if let Some(&target) = others.last() {
                let src = others.first().copied().unwrap_or(owner);
                ctx.emit(EventData::Damage(crate::machine::damage_data(
                    Some(src),
                    target,
                    1,
                )));
            }
        }
        "guju" => {
            // `sp2.js:9487`: when a `傀`-marked character takes damage -> draw 1 card and increment `l11_mod:guju_extra`
            let n = flag(ctx.core, owner, "l11_mod:guju_extra") + 1;
            set_flag(ctx.core, owner, "l11_mod:guju_extra", n);
            ctx.start_draw(owner, 1);
        }
        "gushe" => {
            // `sp.js:17027`: pindian up to 3 targets -> loser discards 1 or owner draws 1, add `饶舌` mark
            if let Some(t) = first_other_alive(ctx.core, owner) {
                if discard_up_to_he(ctx, t, 1).is_empty() {
                    ctx.start_draw(owner, 1);
                }
            }
            let m = flag(ctx.core, owner, "l11_gushe_raoshe") + 1;
            set_flag(ctx.core, owner, "l11_gushe_raoshe", m);
        }
        "jici" => {
            // `sp.js:17100`: boosts `gushe` pindian card rank by `饶舌` count and grants +1 gushe / 1 draw
            let raoshe = flag(ctx.core, owner, "l11_gushe_raoshe").max(1);
            set_flag(ctx.core, owner, "l11_yingyang_pindian_mod", raoshe);
            ctx.start_draw(owner, 1);
        }
        "jieyuan" => {
            // `sp.js:18564`: discard black hand card for damage +1 (`l11_mod:jieyuan_more`) or red for damage -1 (`l11_mod:jieyuan_less`)
            if let Some(&cid) = discard_up_to_he(ctx, owner, 1).first() {
                let is_black = matches!(
                    ctx.catalog.cards[cid as usize].suit,
                    Suit::Spade | Suit::Club
                );
                if is_black {
                    set_flag(ctx.core, owner, "l11_mod:jieyuan_more", 1);
                    set_flag(ctx.core, owner, "l11_mod:jieyuan_less", 0);
                } else {
                    set_flag(ctx.core, owner, "l11_mod:jieyuan_less", 1);
                    set_flag(ctx.core, owner, "l11_mod:jieyuan_more", 0);
                }
            }
        }
        "jigong" => {
            // `yijiang.js:7487`: PlayPhase begin draw 2 cards; hand limit adjusted by damage dealt (`l11_mod:jigong_hand`)
            ctx.start_draw(owner, 2);
            set_flag(ctx.core, owner, "l11_mod:jigong_hand", 2);
        }
        "jili" => {
            // `sp.js:17418`: when distance-1 character is targeted by red basic/trick -> owner also joins (draws/recovers)
            if ctx.core.player(owner).hp < ctx.core.player(owner).max_hp as i32 {
                let _ = ctx.recover(owner, 1, None);
            } else {
                ctx.start_draw(owner, 1);
            }
        }
        "jinglve" => {
            // `mobile.js:11007`: inspect target hand & mark 1 card as `死士` (`l11_jinglve_sishi` cancels target card use)
            if let Some(t) = first_other_alive(ctx.core, owner) {
                set_flag(ctx.core, t, "l11_jinglve_sishi", 1);
                let _ = move_one_hand_to(ctx, t, owner);
            }
        }
        "jsrgshichong" => {
            // `jsrg.js:3056`: zhuanhuanji on single-target card -> yin: steal 1 hand card; yang: target gives 1 hand card
            let yang = flag(ctx.core, owner, "l11_shichong_yang");
            set_flag(
                ctx.core,
                owner,
                "l11_shichong_yang",
                if yang == 0 { 1 } else { 0 },
            );
            if let Some(t) = first_other_alive(ctx.core, owner) {
                if move_one_hand_to(ctx, t, owner).is_none() {
                    ctx.start_draw(owner, 1);
                }
            }
        }
        "jsrgzhenqiao" => {
            // `jsrg.js:3836`: attack range +1 (`d34_l11_mods.rs`); if weapon slot empty, Sha deals extra settlement
            set_flag(ctx.core, owner, "l11_mod:jsrgzhenqiao_range", 1);
            let has_wep = ctx
                .core
                .player(owner)
                .equips
                .iter()
                .any(|&c| ctx.catalog.cards[c as usize].kind.equip_slot() == Some(EquipSlot::Weapon));
            if !has_wep {
                if let Some(t) = first_other_alive(ctx.core, owner) {
                    ctx.emit(EventData::Damage(crate::machine::damage_data(
                        Some(owner),
                        t,
                        1,
                    )));
                }
            }
        }
        "jujian" => {
            // `yijiang.js:11872`: discard up to 3 cards -> another character draws equal cards; if 3 same type, recover 1 HP
            let n = discard_up_to_he(ctx, owner, 2).len().max(1) as u32;
            let ally = first_other_alive(ctx.core, owner).unwrap_or(owner);
            ctx.start_draw(ally, n);
            if n >= 2 {
                let _ = ctx.recover(owner, 1, None);
            }
        }
        "kyou_zhidian" => {
            // `diy.js:3461`: view trick as distance-unlimited Sha (`d34_view_as.rs`) + 4 Sha buffs (`d34_l11_mods.rs`)
            set_flag(ctx.core, owner, "l11_viewas:kyou_zhidian", 1);
            set_flag(ctx.core, owner, "l11_mod:kyou_zhidian_unresp", 1);
            set_flag(ctx.core, owner, "l11_mod:kyou_zhidian_pierce", 1);
            set_flag(ctx.core, owner, "l11_mod:kyou_zhidian_dmg", 1);
            set_flag(ctx.core, owner, "l11_mod:kyou_zhidian_unlimited", 1);
        }
        "leiji" => {
            // `shenhua.js:7205`: on using/responding Shan -> judge target; Spade deals 2 thunder damage
            if let Some(c) = pop_deck(ctx) {
                discard_some(ctx, owner, vec![c]);
            }
            if let Some(t) = first_other_alive(ctx.core, owner) {
                ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                    Some(owner),
                    t,
                    2,
                    DamageNature::Thunder,
                    None,
                )));
            }
        }
        "liangzhu" => {
            // `sp.js:20532`: when a character recovers HP in PlayPhase -> owner draws 1 or target draws 2
            ctx.start_draw(owner, 1);
            if let Some(t) = first_other_alive(ctx.core, owner) {
                ctx.start_draw(t, 1);
            }
        }
        "lianzhu" => {
            // `sp.js:16157`: show & give 1 card to target; if black, target discards 2 cards or owner draws 2 cards
            if let Some(t) = first_other_alive(ctx.core, owner) {
                let _ = move_one_hand_to(ctx, owner, t);
                if discard_up_to_he(ctx, t, 2).len() < 2 {
                    ctx.start_draw(owner, 2);
                }
            }
        }
        "lihun" => {
            // `sp.js:22495`: discard 1 card, turn over (`face_down`), and take all hand cards from male target
            let _ = discard_up_to_he(ctx, owner, 1);
            ctx.core.player_mut(owner).face_down = !ctx.core.player(owner).face_down;
            if let Some(t) = first_other_alive(ctx.core, owner) {
                let thand = ctx.core.player(t).hand.clone();
                for c in thand {
                    ctx.take_hand_card(t, c);
                    ctx.core.player_mut(owner).hand.push(c);
                }
            }
        }
        "lirang" => {
            // `sp.js:20639`: when owner's cards are discarded -> give up to 2 discarded cards to another character
            if let Some(t) = first_other_alive(ctx.core, owner) {
                if let Some(c) = ctx.core.discard.pop() {
                    ctx.core.player_mut(t).hand.push(c);
                } else {
                    ctx.start_draw(t, 1);
                }
            }
        }
        "lixia" => {
            // `sp.js:19457`: others calculate distance to owner -1 (`d34_l11_mods.rs`), and at phase end draw 1 card
            set_flag(ctx.core, owner, "l11_mod:lixia_dist_to_minus1", 1);
            ctx.start_draw(owner, 1);
        }
        "liyu" => {
            // `refresh.js:12527`: after Sha deals damage -> gain 1 card from target and deal 1 Duel damage
            if let Some(t) = first_other_alive(ctx.core, owner) {
                let _ = move_one_hand_to(ctx, t, owner);
                ctx.emit(EventData::Damage(crate::machine::damage_data(
                    Some(owner),
                    t,
                    1,
                )));
            }
        }
        "mingjian" => {
            // `yijiang.js:8886`: give all hand cards to target -> target gets +1 hand limit & +1 Sha limit (`d34_l11_mods.rs`)
            if let Some(t) = first_other_alive(ctx.core, owner) {
                let my_h = ctx.core.player(owner).hand.clone();
                for c in my_h {
                    ctx.take_hand_card(owner, c);
                    ctx.core.player_mut(t).hand.push(c);
                }
                set_flag(ctx.core, t, "l11_mod:mingjian_buff", 1);
            }
            set_flag(ctx.core, owner, "l11_mod:mingjian_buff", 1);
        }
        "mizhao" => {
            // `sp.js:22674`: give all hand cards to target -> target clashes and deals 1 virtual Sha damage
            if let Some(t) = first_other_alive(ctx.core, owner) {
                let my_h = ctx.core.player(owner).hand.clone();
                for c in my_h {
                    ctx.take_hand_card(owner, c);
                    ctx.core.player_mut(t).hand.push(c);
                }
                ctx.emit(EventData::Damage(crate::machine::damage_data(
                    Some(owner),
                    t,
                    1,
                )));
            }
        }
        "mjmouzhi" => {
            // `shiji.js:4013`: prevents damage if damage card suit matches previous damage suit (`d34_l11_mods.rs`)
            set_flag(ctx.core, owner, "l11_mod:mjmouzhi_suit", 1);
        }
        "mouzhu" => {
            // `sp.js:12799`: target gives 1 hand card to owner; if owner has more cards, target clashes/takes 1 Duel damage
            if let Some(t) = first_other_alive(ctx.core, owner) {
                let _ = move_one_hand_to(ctx, t, owner);
                if ctx.core.player(owner).hand.len() > ctx.core.player(t).hand.len() {
                    ctx.emit(EventData::Damage(crate::machine::damage_data(
                        Some(owner),
                        t,
                        1,
                    )));
                }
            }
        }
        "mozhi" => {
            // `sp.js:19068`: at PhaseEnd view hand card as first/second basic or trick used this turn (`d34_view_as.rs`)
            set_flag(ctx.core, owner, "l11_viewas:mozhi", 1);
            ctx.start_draw(owner, 1);
        }
        "new_reqingnang" => {
            // `refresh.js:11844`: discard 1 hand card -> wounded character recovers 1 HP
            let _ = discard_up_to_he(ctx, owner, 1);
            let _ = ctx.recover(owner, 1, None);
        }
        "new_yijue" => {
            // `refresh.js:11585`: discard 1 card & inspect target hand: black locks cards/armor & +1 Heart Sha damage; red gains card & heals
            let _ = discard_up_to_he(ctx, owner, 1);
            if let Some(t) = first_other_alive(ctx.core, owner) {
                set_flag(ctx.core, t, "l11_xingtian_card_locked", 1);
                set_flag(ctx.core, t, "l11_xingtian_armor_null", 1);
                set_flag(ctx.core, owner, "l11_mod:new_yijue_heart_plus1", 1);
                let _ = move_one_hand_to(ctx, t, owner);
            }
        }
        "nsxianhai" => {
            // `diy.js:12236`: 1/round lose 1 max_hp -> target discards all Shan cards from hand
            if ctx.core.player(owner).max_hp > 1 {
                let next_mhp = ctx.core.player(owner).max_hp - 1;
                ctx.core.player_mut(owner).max_hp = next_mhp;
                if ctx.core.player(owner).hp > next_mhp as i32 {
                    ctx.core.player_mut(owner).hp = next_mhp as i32;
                }
            }
            if let Some(t) = first_other_alive(ctx.core, owner) {
                let shans: Vec<CardId> = ctx
                    .core
                    .player(t)
                    .hand
                    .iter()
                    .copied()
                    .filter(|&c| ctx.catalog.cards[c as usize].kind == CardKind::Shan)
                    .collect();
                if !shans.is_empty() {
                    for c in &shans {
                        ctx.take_hand_card(t, *c);
                    }
                    discard_some(ctx, t, shans);
                } else {
                    let _ = discard_up_to_he(ctx, t, 1);
                }
            }
        }
        "nzry_chenglve" => {
            // `shenhua.js:2142`: zhuanhuanji (yin: draw 1 discard 2; yang: draw 2 discard 1) + unlimited distance & Sha use (`d34_l11_mods.rs`)
            let yang = flag(ctx.core, owner, "l11_chenglve_yang");
            if yang == 0 {
                set_flag(ctx.core, owner, "l11_chenglve_yang", 1);
                draw_cards_sync(ctx, owner, 1);
                let _ = discard_up_to_he(ctx, owner, 2);
            } else {
                set_flag(ctx.core, owner, "l11_chenglve_yang", 0);
                draw_cards_sync(ctx, owner, 2);
                let _ = discard_up_to_he(ctx, owner, 1);
            }
            set_flag(ctx.core, owner, "l11_mod:nzry_chenglve", 1);
        }
        "nzry_feijun" => {
            // `shenhua.js:1702`: discard 1 card -> gain 1 card from target with more hand cards or force equip discard
            let _ = discard_up_to_he(ctx, owner, 1);
            if let Some(t) = first_other_alive(ctx.core, owner) {
                if move_one_hand_to(ctx, t, owner).is_none() {
                    let _ = discard_up_to_he(ctx, t, 1);
                }
            }
        }
        "nzry_kuizhu" => {
            // `shenhua.js:1931`: after discard phase -> up to X characters draw 1 card each, or deal 1 damage to target
            ctx.start_draw(owner, 1);
            if let Some(t) = first_other_alive(ctx.core, owner) {
                ctx.emit(EventData::Damage(crate::machine::damage_data(
                    Some(owner),
                    t,
                    1,
                )));
            }
        }
        "nzry_shicai" => {
            // `shenhua.js:2208`: after using a card of a new type this turn -> put card on deck top and draw 1 card
            if let Some(c) = ctx.core.discard.pop() {
                ctx.core.deck.push(c);
            }
            ctx.start_draw(owner, 1);
        }
        "olniepan" => {
            // `refresh.js:9277`: limited 1/game on dying -> discard all cards, reset state, draw 3, recover to 3 HP, gain bazhen/huoji/kanpo
            if flag(ctx.core, owner, "l11_olniepan_used") == 0 {
                set_flag(ctx.core, owner, "l11_olniepan_used", 1);
                let _ = discard_up_to_he(ctx, owner, 16);
                ctx.core.player_mut(owner).face_down = false;
                ctx.core.player_mut(owner).linked = false;
                ctx.core.player_mut(owner).dying = false;
                let target_hp = 3.min(ctx.core.player(owner).max_hp as i32).max(1);
                ctx.core.player_mut(owner).hp = target_hp;
                draw_cards_sync(ctx, owner, 3);
            }
        }
        "olzhuiji" => {
            // `sp.js:7210`: distance to `hp <= owner.hp` is 1 (`d34_l11_mods.rs`); on Sha target force discard 1 or recast equips
            set_flag(ctx.core, owner, "l11_mod:olzhuiji_dist", 1);
            if let Some(t) = first_other_alive(ctx.core, owner) {
                let _ = discard_up_to_he(ctx, t, 1);
            }
        }
        "piaoling" => {
            // `refresh.js:7811`: PhaseEnd judge -> if Heart, give judge card to character (if self, discard 1) or put on deck top
            if let Some(c) = pop_deck(ctx) {
                ctx.core.player_mut(owner).hand.push(c);
                let _ = discard_up_to_he(ctx, owner, 1);
            }
        }
        "qingxian" => {
            // `yijiang.js:3656`: after damage/recover -> target loses 1 HP & equips 1 card from deck, or heals 1 & discards 1 equip
            if let Some(t) = first_other_alive(ctx.core, owner) {
                if discard_up_to_he(ctx, t, 1).is_empty() {
                    ctx.emit(EventData::LoseHp {
                        player: t,
                        amount: 1,
                    });
                }
            }
            let _ = ctx.recover(owner, 1, None);
        }
        "qingzhongx" => {
            // `sp/skill.js:35220`, `markskill.js:18544`: PlayPhase begin draw 2 cards, then swap hand cards with min-hand other player
            draw_cards_sync(ctx, owner, 2);
            if let Some(min_seat) = (0..ctx.core.players.len() as u8)
                .filter(|&p| p != owner && ctx.core.player(p).alive)
                .min_by_key(|&p| ctx.core.player(p).hand.len())
            {
                let my_h = ctx.core.player(owner).hand.clone();
                let th = ctx.core.player(min_seat).hand.clone();
                ctx.core.player_mut(owner).hand = th;
                ctx.core.player_mut(min_seat).hand = my_h;
            }
        }
        "quji" => {
            // `sp.js:21426`: discard X (`lost_hp`) cards -> up to X wounded characters recover 1 HP; if black discarded, lose 1 HP
            let lost = (ctx.core.player(owner).max_hp as i32 - ctx.core.player(owner).hp).max(1) as usize;
            let discarded = discard_up_to_he(ctx, owner, lost);
            let had_black = discarded.iter().any(|&c| {
                matches!(
                    ctx.catalog.cards[c as usize].suit,
                    Suit::Spade | Suit::Club
                )
            });
            let _ = ctx.recover(owner, 1, None);
            if had_black && ctx.core.player(owner).hp > 1 {
                ctx.emit(EventData::LoseHp {
                    player: owner,
                    amount: 1,
                });
            }
        }
        "reduodao" => {
            // `yijiang.js:9145`: when targeted by Sha, discard 1 card -> steal weapon (or equip) from Sha source
            let _ = discard_up_to_he(ctx, owner, 1);
            if let Some(t) = first_other_alive(ctx.core, owner) {
                if let Some(eq) = ctx
                    .core
                    .player(t)
                    .equips
                    .iter()
                    .copied()
                    .find(|&c| ctx.catalog.cards[c as usize].kind.equip_slot() == Some(EquipSlot::Weapon))
                    .or_else(|| ctx.core.player(t).equips.first().copied())
                {
                    ctx.take_owned_card(t, eq);
                    ctx.core.player_mut(owner).hand.push(eq);
                }
            }
        }
        "refenxun" => {
            // `sp.js:11358`: distance to target treated as 1 this turn (`d34_l11_mods.rs`); at PhaseEnd if no damage dealt, discard 1
            set_flag(ctx.core, owner, "l11_mod:fenxun_dist1", 1);
            let _ = discard_up_to_he(ctx, owner, 1);
        }
        "reganlu" => {
            // `mobile.js:7572`: swap all equipment cards in equip zones between owner and another character
            if let Some(t) = first_other_alive(ctx.core, owner) {
                let my_eq = ctx.core.player(owner).equips.clone();
                let t_eq = ctx.core.player(t).equips.clone();
                ctx.core.player_mut(owner).equips = t_eq;
                ctx.core.player_mut(t).equips = my_eq;
            }
        }
        "reguanxing" => {
            // `refresh.js:11932`: PreparePhase inspect & reorder top 5 (or 3) deck cards (prioritizing basic cards on top)
            let dlen = ctx.core.deck.len();
            if dlen >= 2 {
                let start = dlen.saturating_sub(5);
                ctx.core.deck[start..].sort_by_key(|&c| ctx.catalog.cards[c as usize].kind.is_basic_kind());
            }
        }
        "reguose" => {
            // `refresh.js:13736`: view Diamond card as Le (`d34_view_as.rs`) or discard 1 judge card on field, then draw 1 card
            set_flag(ctx.core, owner, "l11_viewas:reguose", 1);
            if let Some(t) = first_other_alive(ctx.core, owner) {
                if let Some(jc) = ctx.core.player(t).judge.first().copied() {
                    ctx.take_owned_card(t, jc);
                    discard_some(ctx, t, vec![jc]);
                }
            }
            ctx.start_draw(owner, 1);
        }
        "rejinjiu" => {
            // `refresh.js:9638`: owner's Jiu viewed as Sha (`d34_view_as.rs`), others cannot use Jiu in owner's turn (`d34_l11_mods.rs`)
            set_flag(ctx.core, owner, "l11_mod:rejinjiu", 1);
        }
        "rejuece" => {
            // `refresh.js:7498`: at PhaseEnd deal 1 damage to another character who lost cards this turn
            if let Some(t) = first_other_alive(ctx.core, owner) {
                ctx.emit(EventData::Damage(crate::machine::damage_data(
                    Some(owner),
                    t,
                    1,
                )));
            }
        }
        "rejunxing" => {
            // `refresh.js:7455`: discard X hand cards -> target turns over & draws X cards (or discards X and loses 1 HP)
            let x = discard_up_to_he(ctx, owner, 1).len().max(1) as u32;
            if let Some(t) = first_other_alive(ctx.core, owner) {
                ctx.core.player_mut(t).face_down = !ctx.core.player(t).face_down;
                ctx.start_draw(t, x);
            }
        }
        "relianying" => {
            // `refresh.js:13010`: when losing last hand card -> up to X characters draw 1 card each
            ctx.start_draw(owner, 1);
            if let Some(t) = first_other_alive(ctx.core, owner) {
                ctx.start_draw(t, 1);
            }
        }
        "reluanji" => {
            // `shenhua.js:3007`: view hand cards as ArcheryAttack (`d34_view_as.rs`) and draw if dodged/no damage
            set_flag(ctx.core, owner, "l11_viewas:reluanji", 1);
            if let Some(t) = first_other_alive(ctx.core, owner) {
                ctx.emit(EventData::Damage(crate::machine::damage_data(
                    Some(owner),
                    t,
                    1,
                )));
            }
            ctx.start_draw(owner, 1);
        }
        "reluoyi" => {
            // `refresh.js:15875`: reveal top 3 cards, gain basic/weapon/Duel, and Sha/Duel damage +1 (`d34_l11_mods.rs`)
            draw_cards_sync(ctx, owner, 1);
            set_flag(ctx.core, owner, "l11_mod:reluoyi_dmg_plus1", 1);
        }
        "renshe" => {
            // `tw/skill.js:29350`: after taking damage -> owner and another character each draw 1 card & +1 waishi use
            ctx.start_draw(owner, 1);
            if let Some(t) = first_other_alive(ctx.core, owner) {
                ctx.start_draw(t, 1);
            }
            set_flag(ctx.core, owner, "l11_renshe_waishi_plus1", 1);
        }
        "repojun" => {
            // `refresh.js:11979`: on Sha target exile target card & Sha damage +1 when hand/equip >= target (`d34_l11_mods.rs`)
            if let Some(t) = first_other_alive(ctx.core, owner) {
                let _ = discard_up_to_he(ctx, t, 1);
            }
            set_flag(ctx.core, owner, "l11_mod:repojun_dmg_plus1", 1);
        }
        "reqizhou" => {
            // `mobile/skill.js:28908`: by equip suit count -> 1+: yingzi draw 1; 2+: qixi dismantle 1; 3+: xuanfeng
            ctx.start_draw(owner, 1);
            if !ctx.core.player(owner).equips.is_empty() {
                if let Some(t) = first_other_alive(ctx.core, owner) {
                    let _ = discard_up_to_he(ctx, t, 1);
                }
            }
        }
        "requanji" => {
            // `mobile/skill.js:23519`: after damage / PlayPhase end -> draw 1 card and add 1 `权` (+1 hand limit in `d34_l11_mods.rs`)
            draw_cards_sync(ctx, owner, 1);
            if let Some(c) = ctx.core.player(owner).hand.first().copied() {
                ctx.take_hand_card(owner, c);
                ctx.core
                    .vm_aux
                    .piles
                    .entry((owner, "requanji".to_string()))
                    .or_default()
                    .push(c);
            }
            let quan = flag(ctx.core, owner, "l11_mod:requanji_quan") + 1;
            set_flag(ctx.core, owner, "l11_mod:requanji_quan", quan);
        }
        "reshuangxiong" => {
            // `refresh.js:13862`: reveal top 2 cards, gain 1, and view different-color hand cards as Duel (`d34_view_as.rs`)
            draw_cards_sync(ctx, owner, 1);
            set_flag(ctx.core, owner, "l11_mod:reshuangxiong_duel", 1);
        }
        "retianxiang" => {
            // `shenhua.js:7913`: discard 1 Heart/hand card to prevent damage (`d34_l11_mods.rs`), redirect 1 damage to target, target draws lost_hp
            let _ = discard_up_to_he(ctx, owner, 1);
            set_flag(ctx.core, owner, "l11_mod:retianxiang_ward", 1);
            if let Some(t) = first_other_alive(ctx.core, owner) {
                ctx.emit(EventData::Damage(crate::machine::damage_data(
                    Some(owner),
                    t,
                    1,
                )));
                let lost =
                    (ctx.core.player(t).max_hp as i32 - ctx.core.player(t).hp).clamp(1, 5) as u32;
                ctx.start_draw(t, lost);
            }
        }
        "retuntian" => {
            // `refresh.js:17516`: out-of-turn lose card -> judge: Heart gains card, else +1 `田` (`l11_mod:retuntian_tian` distance -X)
            let tian = flag(ctx.core, owner, "l11_mod:retuntian_tian") + 1;
            set_flag(ctx.core, owner, "l11_mod:retuntian_tian", tian);
            if let Some(c) = pop_deck(ctx) {
                if ctx.catalog.cards[c as usize].suit == Suit::Heart {
                    ctx.core.player_mut(owner).hand.push(c);
                } else {
                    ctx.core
                        .vm_aux
                        .piles
                        .entry((owner, "retuntian".to_string()))
                        .or_default()
                        .push(c);
                }
            }
        }
        "retuxi" => {
            // `refresh.js:15715`: DrawPhase -> steal 1 hand card from up to 2 other alive characters
            let others: Vec<PlayerId> = (0..ctx.core.players.len() as u8)
                .filter(|&p| p != owner && ctx.core.player(p).alive)
                .take(2)
                .collect();
            for t in others {
                let _ = move_one_hand_to(ctx, t, owner);
            }
        }
        "rewurong" => {
            // `refresh.js:11555`: simultaneous reveal -> if target reveals Shan, owner gains 1 card from target; else deal 1 damage
            if let Some(t) = first_other_alive(ctx.core, owner) {
                let target_has_shan = ctx
                    .core
                    .player(t)
                    .hand
                    .iter()
                    .any(|&c| ctx.catalog.cards[c as usize].kind == CardKind::Shan);
                if target_has_shan {
                    let _ = move_one_hand_to(ctx, t, owner);
                } else {
                    ctx.emit(EventData::Damage(crate::machine::damage_data(
                        Some(owner),
                        t,
                        1,
                    )));
                }
            }
        }
        "rexingluan" => {
            // `sp2/skill.js:9812`: 1/phase after single-target card resolves -> gain 1 rank-6 card from deck
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
        "rexingshang" => {
            // `refresh.js:17643`: when a character dies -> recover 1 HP or gain all of deceased's hand cards
            if ctx.core.player(owner).hp < ctx.core.player(owner).max_hp as i32 {
                let _ = ctx.recover(owner, 1, None);
            } else {
                draw_cards_sync(ctx, owner, 2);
            }
        }
        "reyajiao" => {
            // `refresh.js:16396`: out-of-turn card use/respond -> reveal top card and give to an alive player
            if let Some(c) = pop_deck(ctx) {
                ctx.core.player_mut(owner).hand.push(c);
            }
        }
        "rezaiqi" => {
            // `refresh.js:17932`: PhaseEnd -> up to X characters draw 1 card or let owner recover 1 HP
            if ctx.core.player(owner).hp < ctx.core.player(owner).max_hp as i32 {
                let _ = ctx.recover(owner, 1, None);
            } else {
                ctx.start_draw(owner, 1);
            }
        }
        "rezhijian" => {
            // `refresh.js:17497`: put 1 equip from hand into another player's equip zone (or use equip) and draw 1 card
            if let Some(t) = first_other_alive(ctx.core, owner) {
                if let Some(eq) = ctx
                    .core
                    .player(owner)
                    .hand
                    .iter()
                    .copied()
                    .find(|&c| ctx.catalog.cards[c as usize].kind.is_equip())
                {
                    ctx.take_hand_card(owner, eq);
                    ctx.core.player_mut(t).equips.push(eq);
                }
            }
            ctx.start_draw(owner, 1);
        }
        "rongbei" => {
            // `shiji/skill.js:7148`: limited 1/game -> equip empty slots of target from deck/discard
            if flag(ctx.core, owner, "l11_rongbei_used") == 0 {
                set_flag(ctx.core, owner, "l11_rongbei_used", 1);
                let mut equipped = 0u32;
                while equipped < 2 {
                    if let Some(pos) = ctx
                        .core
                        .deck
                        .iter()
                        .position(|&c| ctx.catalog.cards[c as usize].kind.is_equip())
                    {
                        let cid = ctx.core.deck.remove(pos);
                        ctx.core.player_mut(owner).equips.push(cid);
                        equipped += 1;
                    } else {
                        break;
                    }
                }
                if equipped == 0 {
                    draw_cards_sync(ctx, owner, 2);
                }
            }
        }
        _ => return false,
    }
    true
}
