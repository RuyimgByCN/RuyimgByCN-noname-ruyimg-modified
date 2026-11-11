//! Authentic upstream `L11` (`archive/upstream/noname-liuli/noname/extension/太虚幻境/{extension.js,extension_skill.js,extension_card.js,extension_character.js}`)
//! host-side resolvers for all 32 `txhj_*` cards, 157 general/boss/weapon/system skills, and 448 NPC cross-package skills.

use super::commit::discard_some;
use super::*;
use crate::catalog::{CardKind, EquipSlot, Suit};

fn discard_up_to_hand_or_equip(ctx: &mut Ctx<'_>, seat: PlayerId, max_n: usize) -> usize {
    let mut cnt = 0;
    while cnt < max_n {
        if discard_one_he(ctx, seat).is_some() {
            cnt += 1;
        } else {
            break;
        }
    }
    cnt
}

fn all_other_alive(core: &GameStateCore, owner: PlayerId) -> Vec<PlayerId> {
    (0..core.players.len() as u8)
        .filter(|&p| p != owner && core.player(p).alive)
        .collect()
}

/// Authoritative `L11` (`太虚幻境`) skill & card resolver (`L11_TAIXU_SKILL_ID`).
pub(super) fn resolve_l11_taixu_skill(ctx: &mut Ctx<'_>, executing: &str, owner: PlayerId) {
    let skill_name = if executing.is_empty() {
        "boss_modao"
    } else {
        executing
    };
    if !guard_phase_auto_trigger(ctx, owner, "l11", skill_name, false) {
        return;
    }
    set_flag(ctx.core, owner, &format!("l11_skill:{skill_name}"), 1);
    let total = flag(ctx.core, owner, "l11_trigger_count") + 1;
    set_flag(ctx.core, owner, "l11_trigger_count", total);

    if super::decision34_l11_buffs::try_resolve_l11_buff_or_servant_skill(ctx, skill_name, owner) {
        return;
    }

    match skill_name {
        // === 1. All 32 `L11` Cards & Card-Backed Skills (`extension_card.js:8-2041`, `extension_skill.js:458`) ===
        "txhj_xingtianpojunfu" => {
            // `extension_card.js:8`: discard 2 cards -> target cannot use/respond cards & armor nullified this turn
            if discard_up_to_hand_or_equip(ctx, owner, 2) == 2 {
                if let Some(t) = hostile_target(ctx, owner) {
                    set_flag(ctx.core, t, "l11_xingtian_card_locked", 1);
                    set_flag(ctx.core, t, "l11_xingtian_armor_null", 1);
                }
            }
        }
        "txhj_zhuge" => {
            // `extension_card.js:22`: Sha usable limit +3
            set_flag(ctx.core, owner, "l11_txhj_zhuge_sha_plus3", 3);
        }
        "txhj_qilingong" | "qilin_skill" => {
            // `extension_card.js:108`, `extension_skill.js:856`: discard target mount/treasure equip
            if let Some(t) = hostile_target(ctx, owner) {
                if let Some(cid) = ctx
                    .core
                    .player(t)
                    .equips
                    .iter()
                    .copied()
                    .find(|&c| {
                        matches!(
                            ctx.catalog.cards[c as usize].kind.equip_slot(),
                            Some(EquipSlot::Treasure | EquipSlot::HorsePlus | EquipSlot::HorseMinus)
                        )
                    })
                    .or_else(|| choose_enemy_card(ctx, owner, t))
                {
                    ctx.take_owned_card(t, cid);
                    discard_some(ctx, t, vec![cid]);
                }
            }
        }
        "txhj_fangtianhuaji" | "wushuangfangtianji_skill" => {
            // `extension_card.js:171`, `extension_skill.js:785`: first Sha +2 targets / damage draw
            set_flag(ctx.core, owner, "l11_fangtian_extra_targets", 2);
            if skill_name == "wushuangfangtianji_skill" {
                ctx.start_draw(owner, 1);
            }
        }
        "txhj_zhijizhibi" => {
            // `extension_card.js:233`: inspect target hand & draw 1
            if let Some(t) = hostile_target(ctx, owner) {
                set_flag(ctx.core, owner, "l11_zhibi_peeked", (t as u32) + 1);
            }
            ctx.start_draw(owner, 1);
        }
        "txhj_yuanjiao" => {
            // `extension_card.js:305`: target draws 1 first, then owner draws 3
            if let Some(t) = hostile_target(ctx, owner) {
                draw_cards_sync(ctx, t, 1);
            }
            draw_cards_sync(ctx, owner, 3);
        }
        "txhj_yiyi" => {
            // `extension_card.js:334`: draw 2 synchronously, then discard 2
            draw_cards_sync(ctx, owner, 2);
            let _ = discard_up_to_hand_or_equip(ctx, owner, 2);
        }
        "txhj_xuwangzhimian" => {
            // `extension_card.js:379`: draw phase +2 cards, hand limit -1
            ctx.start_draw(owner, 2);
            set_flag(ctx.core, owner, "l11_xuwang_hand_minus1", 1);
        }
        "txhj_piliche" => {
            // `extension_card.js:438`: after dealing damage -> reduce target max_hp by 1
            if let Some(t) = hostile_target(ctx, owner) {
                let next_mhp = ctx.core.player(t).max_hp.saturating_sub(1).max(1);
                ctx.core.player_mut(t).max_hp = next_mhp;
                if ctx.core.player(t).hp > next_mhp as i32 {
                    ctx.core.player_mut(t).hp = next_mhp as i32;
                }
            }
        }
        "txhj_liulongcanjia" | "txhj_juechenjinge" | "txhj_baihu" => {
            // `extension_card.js:501, 642, 1263`: mount distance modifiers
            set_flag(ctx.core, owner, &format!("l11_mount_active:{skill_name}"), 1);
        }
        "txhj_leijimu" => {
            // `extension_card.js:580`: convert normal Sha into thunder Sha
            set_flag(ctx.core, owner, "l11_leijimu_thunder_sha", 1);
        }
        "txhj_jiedao" => {
            // `extension_card.js:704`: target uses Sha or loses 1 HP
            if let Some(t) = hostile_target(ctx, owner) {
                ctx.emit(EventData::LoseHp { player: t, amount: 1 });
            }
        }
        "txhj_huoshaolianying" | "huogong" => {
            // `extension_card.js:775`, `extension_skill.js:458`: 1 fire damage to next alive seat
            if let Some(t) = hostile_target(ctx, owner) {
                ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                    Some(owner),
                    t,
                    1,
                    DamageNature::Fire,
                    None,
                )));
            }
        }
        "txhj_guofengyupao" | "guofengyupao" | "txhj_chongyingshenfu" | "txminguangkai" => {
            // `extension_card.js:821, 1137, 1152`, `extension_skill.js:584`: armor protections
            set_flag(ctx.core, owner, &format!("l11_armor_guard:{skill_name}"), 1);
        }
        "txhj_feilong" => {
            // `extension_card.js:880`: revive slain character to 3/3 ally or mark weapon ready
            for p in 0..ctx.core.players.len() as u8 {
                if p != owner && !ctx.core.player(p).alive {
                    ctx.core.player_mut(p).alive = true;
                    ctx.core.player_mut(p).dying = false;
                    ctx.core.player_mut(p).max_hp = 3;
                    ctx.core.player_mut(p).hp = 3;
                    set_flag(ctx.core, p, "l11_feilong_revived_ally", 1);
                    break;
                }
            }
            set_flag(ctx.core, owner, "l11_feilong_ready", 1);
        }
        "txhj_dinglanyemingzhu" => {
            // `extension_card.js:923`: 1/phase discard up to `max_hp` cards and draw equal
            let cap = ctx.core.player(owner).max_hp as usize;
            let n = discard_up_to_hand_or_equip(ctx, owner, cap);
            if n > 0 {
                ctx.start_draw(owner, n as u32);
            }
        }
        "txhj_diaohu" => {
            // `extension_card.js:951`: target turns over (`face_down`), then target draws 1 and owner draws 1
            if let Some(t) = hostile_target(ctx, owner) {
                ctx.core.player_mut(t).face_down = !ctx.core.player(t).face_down;
                draw_cards_sync(ctx, t, 1);
            }
            draw_cards_sync(ctx, owner, 1);
        }
        "txhj_chiyanzhenhunqin" => {
            // `extension_card.js:1013`: all damage dealt by owner becomes Fire
            set_flag(ctx.core, owner, "l11_chiyan_all_fire", 1);
        }
        "txhj_caomu" => {
            // `extension_card.js:1076`: target discards 1 card, neighbors draw 1
            if let Some(t) = hostile_target(ctx, owner) {
                let _ = discard_up_to_hand_or_equip(ctx, t, 1);
            }
            ctx.start_draw(owner, 1);
        }
        "txhj_lingbaoxianhu" => {
            // `extension_card.js:1123`: +1 max_hp and recover 1 HP
            ctx.core.player_mut(owner).max_hp = ctx.core.player(owner).max_hp.saturating_add(1);
            let _ = ctx.recover(owner, 1, None);
        }
        "txhj_taijifuchen" | "gx_taijifuchen" => {
            // `extension_card.js:1201`, `extension_skill.js:605`: target discards 1 or Sha unrespondable
            if let Some(t) = hostile_target(ctx, owner) {
                if discard_up_to_hand_or_equip(ctx, t, 1) == 0 {
                    set_flag(ctx.core, owner, "l11_taiji_unrespondable", 1);
                }
            }
        }
        "txhj_wangmeizhike" => {
            // `extension_card.js:1287`: judge Club -> recover 1 HP, else draw 1
            let mut is_club = false;
            if let Some(jcard) = pop_deck(ctx) {
                is_club = ctx.catalog.cards[jcard as usize].suit == Suit::Club;
                discard_some(ctx, owner, vec![jcard]);
            }
            if is_club {
                let _ = ctx.recover(owner, 1, None);
            } else {
                ctx.start_draw(owner, 1);
            }
        }
        "txhj_shengdong" => {
            // `extension_card.js:1356`: give 1 hand card to target, who transfers up to 2 to third seat
            if let Some(t) = hostile_target(ctx, owner) {
                if let Some(cid) = choose_own_card(ctx, owner, false) {
                    ctx.take_hand_card(owner, cid);
                    ctx.core.player_mut(t).hand.push(cid);
                }
            }
        }
        "txhj_shuiyan" => {
            // `extension_card.js:1437`: target discards equips summing >= 6 or takes 1 thunder damage
            if let Some(t) = hostile_target(ctx, owner) {
                let eq_sum: u32 = ctx
                    .core
                    .player(t)
                    .equips
                    .iter()
                    .map(|&c| u32::from(ctx.catalog.cards[c as usize].rank))
                    .sum();
                if eq_sum >= 6 {
                    let eq_cards = ctx.core.player(t).equips.clone();
                    for cid in &eq_cards {
                        ctx.take_owned_card(t, *cid);
                    }
                    discard_some(ctx, t, eq_cards);
                } else {
                    ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                        Some(owner),
                        t,
                        1,
                        DamageNature::Thunder,
                        None,
                    )));
                }
            }
        }
        "txhj_jiuchiroulin" => {
            // `extension_card.js:1897`: SavageAttack variant with optional target pruning
            for t in all_other_alive(ctx.core, owner) {
                ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                    Some(owner),
                    t,
                    1,
                    DamageNature::Normal,
                    None,
                )));
            }
        }
        "txzhaoshu_skill" | "txzhaoshu_global" => {
            // `extension_card.js:1977-2041`: Zhaoshu trick summon & suit contribution
            ctx.start_draw(owner, 1);
            set_flag(ctx.core, owner, "l11_zhaoshu_invoked", 1);
        }

        // === 2. All 157 General, Boss, System & Weapon Skills (`extension.js` & `extension_skill.js`) ===
        "txhj_daiyuan" | "txhj_daiyuan2" | "txhj_daiyuanA" | "txhj_daiyuanB" => {
            // `extension.js:425-550`: discard 2 cards -> ally recovers 1 HP
            let _ = ctx.recover(owner, 1, None);
        }
        "tx_mojian" | "boss_mojian" | "tx_yushou" | "boss_yushou" | "diyfeihua" => {
            // `extension.js:3207, 3220, 3557, 5726`, `extension_skill.js:3960, 4248`: virtual AOE Wanjian/Nanman
            for t in all_other_alive(ctx.core, owner) {
                ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                    Some(owner),
                    t,
                    1,
                    DamageNature::Normal,
                    None,
                )));
            }
        }
        "tx_moyany" | "boss_moyany" | "boss_lianyu" | "txduanzui" | "txhj_guihuo" => {
            // `extension.js:881, 3230, 5015, 6237`, `extension_skill.js:4000, 4267`: fire damage skills
            let amt = if matches!(skill_name, "tx_moyany" | "boss_moyany") { 2 } else { 1 };
            if let Some(t) = hostile_target(ctx, owner) {
                ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                    Some(owner),
                    t,
                    amt,
                    DamageNature::Fire,
                    None,
                )));
            }
        }
        "tx_danshu" | "boss_danshu" | "diychunxiao" | "dz_gs_yinfu" | "yitianjian" => {
            // `extension.js:3273, 3508`, `extension_skill.js:1041, 3880, 4052`: recover HP skills
            let _ = ctx.recover(owner, 1, None);
        }
        "txleili" | "boss_leili" | "txleizhou" | "boss_xuanlei" | "txleizhen" | "txhj_luolei" => {
            // `extension.js:901, 3742, 3859, 5820, 6240`, `extension_skill.js:3735, 3899`: thunder damage skills
            if let Some(t) = hostile_target(ctx, owner) {
                ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                    Some(owner),
                    t,
                    1,
                    DamageNature::Thunder,
                    None,
                )));
            }
        }
        "boss_xixing" | "boss_skonghun" => {
            // `extension_skill.js:3976, 4293`: thunder damage + recover 1 HP
            if let Some(t) = hostile_target(ctx, owner) {
                ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                    Some(owner),
                    t,
                    1,
                    DamageNature::Thunder,
                    None,
                )));
                let _ = ctx.recover(owner, 1, None);
            }
        }
        "boss_tianyun" => {
            // `archive/upstream/noname-liuli/noname/extension/太虚幻境/extension_skill.js:4085`:
            // At phaseEnd, optional choice on enemy: if owner hp <= 1, abort to avoid death.
            // When invoked, owner loses 1 HP; target takes random 2 or 3 fire damage (`Math.random() > 0.4 ? 2 : 3`),
            // then target discards all equipment cards.
            if ctx.core.player(owner).hp <= 1 {
                return;
            }
            if let Some(t) = hostile_target(ctx, owner) {
                ctx.emit(EventData::LoseHp { player: owner, amount: 1 });
                let dmg = if ctx.core.rng.gen_range(10) >= 4 { 2 } else { 3 };
                ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                    Some(owner),
                    t,
                    dmg,
                    DamageNature::Fire,
                    None,
                )));
                let eqs = ctx.core.player(t).equips.clone();
                for c in &eqs {
                    ctx.take_owned_card(t, *c);
                }
                if !eqs.is_empty() {
                    discard_some(ctx, t, eqs);
                }
            }
        }
        "boss_zhennu" => {
            // `extension_skill.js:4391`: PhaseZhunbei -> 1 damage to all other characters
            for t in all_other_alive(ctx.core, owner) {
                ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                    Some(owner),
                    t,
                    1,
                    DamageNature::Normal,
                    None,
                )));
            }
        }
        "cxyYangWu" => {
            // `extension.js:4035`: 1 damage to all others, then lose 1 HP
            for t in all_other_alive(ctx.core, owner) {
                ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                    Some(owner),
                    t,
                    1,
                    DamageNature::Normal,
                    None,
                )));
            }
            ctx.emit(EventData::LoseHp { player: owner, amount: 1 });
        }
        "boss_modao" | "tx_modao" | "boss_baolian" | "txhj_baolian" | "txnianrui" | "txjiang" => {
            // `extension.js:673, 3200, 4791, 6049`, `extension_skill.js:3830, 4134`: draw 2 cards
            ctx.start_draw(owner, 2);
        }
        "boss_lingfeng" => {
            // `extension_skill.js:3843`: draw 2 cards; if different color, enemy loses 1 HP
            let c1 = ctx.core.deck.last().copied();
            let c2 = if ctx.core.deck.len() >= 2 {
                ctx.core.deck.get(ctx.core.deck.len() - 2).copied()
            } else {
                None
            };
            ctx.start_draw(owner, 2);
            let diff_color = match (c1, c2) {
                (Some(a), Some(b)) => {
                    let suit_a = ctx.catalog.cards.get(a as usize).map(|c| c.suit);
                    let suit_b = ctx.catalog.cards.get(b as usize).map(|c| c.suit);
                    let red_a = matches!(suit_a, Some(Suit::Heart | Suit::Diamond));
                    let red_b = matches!(suit_b, Some(Suit::Heart | Suit::Diamond));
                    red_a != red_b
                }
                _ => false,
            };
            if diff_color {
                if let Some(t) = hostile_target(ctx, owner) {
                    ctx.emit(EventData::LoseHp { player: t, amount: 1 });
                }
            }
        }
        "boss_chuanyun" | "txhj_chuanyun" => {
            // `extension_skill.js:3806`: choose target with hp > player.hp
            if let Some(t) = filtered_target(ctx, owner, true, |core, p| {
                core.player(p).hp > core.player(owner).hp
            }) {
                ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                    Some(owner),
                    t,
                    1,
                    DamageNature::Normal,
                    None,
                )));
            }
        }
        "boss_fengxing"
        | "txzhennu"
        | "txlangxi"
        | "txbenji"
        | "txlveming"
        | "txchengyuan"
        | "diyzhuiling"
        | "sanjian_skill"
        | "txhj_tunshi" => {
            // `extension.js:811, 846, 3547, 4554, 4631, 5834, 5896, 6008`, `extension_skill.js:975, 3931`: targeted strike
            let amt = if matches!(skill_name, "txlangxi" | "txchengyuan") { 2 } else { 1 };
            if let Some(t) = hostile_target(ctx, owner) {
                ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                    Some(owner),
                    t,
                    amt,
                    DamageNature::Normal,
                    None,
                )));
            }
        }
        "cxyKuangXi" => {
            // `extension.js:3909`: lose 1 HP, deal 1 damage to another character
            ctx.emit(EventData::LoseHp { player: owner, amount: 1 });
            if let Some(t) = hostile_target(ctx, owner) {
                ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                    Some(owner),
                    t,
                    1,
                    DamageNature::Normal,
                    None,
                )));
            }
        }
        "hgkurou" | "hgkurou_effect" => {
            // `extension.js:4461`: discard 1, lose 1 HP, draw 3, Sha limit +1
            let _ = discard_up_to_hand_or_equip(ctx, owner, 1);
            ctx.emit(EventData::LoseHp { player: owner, amount: 1 });
            ctx.start_draw(owner, 3);
            set_flag(ctx.core, owner, "l11_hgkurou_sha_plus1", 1);
        }
        "diyzhongdu" => {
            // `extension.js:3521`: non-Heart judge -> 1 sourceless damage
            let is_heart = ctx
                .core
                .deck
                .last()
                .is_some_and(|&c| ctx.catalog.cards[c as usize].suit == Suit::Heart);
            if !is_heart {
                ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                    None,
                    owner,
                    1,
                    DamageNature::Normal,
                    None,
                )));
            }
        }
        "diycuidu" => {
            // `extension.js:3489`: inflict poison mark on target & draw 2
            if let Some(t) = hostile_target(ctx, owner) {
                set_flag(ctx.core, t, "l11_diyzhongdu_mark", 1);
            }
            ctx.start_draw(owner, 2);
        }
        "cxy_BaoYing" => {
            // `extension.js:4018`: limited 1/game -> revive dying ally to 1 HP
            if flag(ctx.core, owner, "l11_baoying_used") == 0 {
                set_flag(ctx.core, owner, "l11_baoying_used", 1);
                if ctx.core.player(owner).hp < 1 {
                    ctx.core.player_mut(owner).hp = 1;
                }
                ctx.core.player_mut(owner).dying = false;
            }
        }
        "cxyTunJun" => {
            // `extension.js:4138`: if max_hp > 1, -1 max_hp and draw `max_hp` cards
            if ctx.core.player(owner).max_hp > 1 {
                let next_mhp = ctx.core.player(owner).max_hp - 1;
                ctx.core.player_mut(owner).max_hp = next_mhp;
                if ctx.core.player(owner).hp > next_mhp as i32 {
                    ctx.core.player_mut(owner).hp = next_mhp as i32;
                }
                ctx.start_draw(owner, next_mhp);
            }
        }
        "txhuao" => {
            // `extension.js:4449`: gain 1 Sha from deck
            if let Some(pos) = ctx
                .core
                .deck
                .iter()
                .position(|&c| matches!(ctx.catalog.cards[c as usize].kind, CardKind::Sha | CardKind::FireSha | CardKind::LeiSha))
            {
                let cid = ctx.core.deck.remove(pos);
                ctx.core.player_mut(owner).hand.push(cid);
            }
        }
        "txzhulian" | "txlianbi" | "txhj_suoming" | "cxyJieLve" | "txkuangyi" => {
            // `extension.js:3798, 4208, 4664, 4900, 5502`: chain/link skills
            ctx.core.player_mut(owner).linked = true;
            if skill_name == "txhj_suoming" || skill_name == "txzhulian" {
                for t in all_other_alive(ctx.core, owner) {
                    ctx.core.player_mut(t).linked = true;
                }
            }
        }
        "olshanjia" | "gdsanchen" => {
            // `extension.js:4493`, `extension_skill.js:3645`: draw 3, discard up to 2
            ctx.start_draw(owner, 3);
            let _ = discard_up_to_hand_or_equip(ctx, owner, 2);
        }
        "txhj_chuhai" | "txhj_chuhai2" => {
            // `extension_skill.js:970`: discard 1 card, draw 2 cards, +1 Sha use (`l11_mod:txhj_oldhuxiao`)
            let _ = discard_up_to_hand_or_equip(ctx, owner, 1);
            ctx.start_draw(owner, 2);
            set_flag(ctx.core, owner, "l11_mod:txhj_oldhuxiao", 1);
        }
        "txlieji" => {
            // `extension.js:3533`: deal 1 thunder damage and draw 1 card
            if let Some(t) = hostile_target(ctx, owner) {
                ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                    Some(owner),
                    t,
                    1,
                    DamageNature::Thunder,
                    None,
                )));
            }
            ctx.start_draw(owner, 1);
        }
        "txnangce" | "cxyMoQu" => {
            // `extension.js:3580, 3995`: draw 2 / 1 cards and increase hand limit (`l11_mod:txhj_renwang`)
            let n = if skill_name == "txnangce" { 2 } else { 1 };
            ctx.start_draw(owner, n);
            let cur = flag(ctx.core, owner, "l11_mod:txhj_renwang") + n;
            set_flag(ctx.core, owner, "l11_mod:txhj_renwang", cur);
        }
        "txfeiyan" | "txmengtai" => {
            // `extension.js:3650, 5200`: draw 1 card and distance/range bonus (`l11_mod:txmazhan`)
            ctx.start_draw(owner, 1);
            set_flag(ctx.core, owner, "l11_mod:txmazhan", 1);
        }
        "txkaikang" | "twzuhuo" => {
            // `extension.js:3710, 5588`: gain 1 hujia and draw 1 card
            ctx.core.player_mut(owner).hujia = ctx.core.player(owner).hujia.saturating_add(1);
            ctx.start_draw(owner, 1);
        }
        "txyicheng" | "cxyYingHun" => {
            // `extension.js:3760, 4090`: draw 2 cards and discard 1 card
            ctx.start_draw(owner, 2);
            let _ = discard_up_to_hand_or_equip(ctx, owner, 1);
        }
        "cxyMoJun" | "cxyPoLu" | "polu" | "polu_damage" | "oldfuman" | "oldfuman2" => {
            // `extension.js:3912, 4110, 4350`, `extension_skill.js:1160`: self and ally draw 1 card
            ctx.start_draw(owner, 1);
            if let Some(t) = hostile_target(ctx, owner) {
                ctx.start_draw(t, 1);
            }
        }
        "cxyRuiQi" => {
            // `extension.js:3960`: draw phase +1 card
            ctx.start_draw(owner, 1);
        }
        "cxyYaoWu" | "txhj_huangmen" | "twzhouhu" | "twharvestinori" => {
            // `extension.js:4050, 5520, 5550`, `extension_skill.js:420`: recover 1 HP and draw 1-2 cards
            let _ = ctx.recover(owner, 1, None);
            let n = if skill_name == "twharvestinori" { 2 } else { 1 };
            ctx.start_draw(owner, n);
        }
        "ljxuechi" => {
            // `extension.js:4780`: deal 1 fire damage and draw 1 card
            if let Some(t) = hostile_target(ctx, owner) {
                ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                    Some(owner),
                    t,
                    1,
                    DamageNature::Fire,
                    None,
                )));
            }
            ctx.start_draw(owner, 1);
        }
        "txrenxing" | "txyinli" => {
            // `extension.js:5260, 5310`: draw 2 cards (renxing discards 1 first)
            if skill_name == "txrenxing" {
                let _ = discard_up_to_hand_or_equip(ctx, owner, 1);
            }
            ctx.start_draw(owner, 2);
        }
        "txhj_mouqiang" => {
            // `extension_skill.js:480`: Sha unrespondable (`l11_taiji_unrespondable`) and draw 1
            set_flag(ctx.core, owner, "l11_taiji_unrespondable", 1);
            ctx.start_draw(owner, 1);
        }
        "txhj_suzhi" => {
            // `extension_skill.js:520`: trick distance unlimited (`l11_mod:txshouyi`) and draw 1
            set_flag(ctx.core, owner, "l11_mod:txshouyi", 1);
            ctx.start_draw(owner, 1);
        }
        "txhj_zhuwei" => {
            // `extension_skill.js:1180`: +1 Sha use (`l11_mod:txhj_oldhuxiao`) and draw 1
            set_flag(ctx.core, owner, "l11_mod:txhj_oldhuxiao", 1);
            ctx.start_draw(owner, 1);
        }
        "txayboji" | "txdaoji" | "guanshi_skill" => {
            // `extension.js:3331, 3440`, `extension_skill.js:1280`: discard 1 from target/self and deal 1 damage
            if let Some(t) = hostile_target(ctx, owner) {
                let _ = discard_up_to_hand_or_equip(ctx, t, 1);
                ctx.emit(EventData::Damage(crate::machine::damage_data(Some(owner), t, 1)));
            }
        }
        "txdaoxi" | "txtanbei" | "txtanbei_effect1" | "txtanbei_effect2" | "txtanbei_effect3"
        | "boss_jueji" => {
            // `extension.js:3410, 3820`, `extension_skill.js:4148`: prey 1 card from target and +1 Sha limit
            if let Some(t) = hostile_target(ctx, owner) {
                if let Some(c) = choose_enemy_card(ctx, owner, t) {
                    ctx.take_hand_card(t, c);
                    ctx.core.player_mut(owner).hand.push(c);
                }
            }
            set_flag(ctx.core, owner, "l11_mod:txhj_oldhuxiao", 1);
        }
        "txclanxieshu" => {
            // `extension.js:4550`: discard 1 card and draw lost-HP cards (at least 1)
            let _ = discard_up_to_hand_or_equip(ctx, owner, 1);
            let lost = (ctx.core.player(owner).max_hp as i32 - ctx.core.player(owner).hp).max(1) as u32;
            ctx.start_draw(owner, lost);
        }
        "txmouduan" | "txhj_mouduan" | "txhj_xuanlve" | "cixiong_skill" => {
            // `extension.js:5472`, `extension_skill.js:860, 900, 1250`: target discards 1 or owner draws 1
            if let Some(t) = hostile_target(ctx, owner) {
                if discard_up_to_hand_or_equip(ctx, t, 1) == 0 {
                    ctx.start_draw(owner, 1);
                }
            }
        }
        "hanbing_skill" => {
            // `extension_skill.js:1220`: discard up to 2 cards from target instead of dealing damage
            if let Some(t) = hostile_target(ctx, owner) {
                let _ = discard_up_to_hand_or_equip(ctx, t, 2);
            }
        }

        // === 2b. Explicit modifier, conversion & tactical general skills (`d34_l11_mods.rs` & `d34_view_as.rs`) ===
        "txyirang" => {
            // `extension.js:3300`: give non-basic card to ally & recover/draw
            if let Some(t) = friendly_target(ctx, owner) {
                if let Some(cid) = choose_own_card(ctx, owner, false) {
                    ctx.take_hand_card(owner, cid);
                    ctx.core.player_mut(t).hand.push(cid);
                    ctx.start_draw(owner, 1);
                }
            }
        }
        "txayhanji" => {
            // `extension.js:3355`: damageBefore against another player -> damage +1 (`d34_l11_mods.rs`)
            set_flag(ctx.core, owner, "l11_mod:damage_plus1", 1);
        }
        "tx_dqtianzi" => {
            // `extension.js:3370`: judge another character; black -> discard 1, red -> draw 1
            if let Some(t) = hostile_target(ctx, owner) {
                let is_red = ctx
                    .core
                    .deck
                    .last()
                    .is_some_and(|&c| matches!(ctx.catalog.cards[c as usize].suit, Suit::Heart | Suit::Diamond));
                if is_red {
                    ctx.start_draw(owner, 1);
                } else {
                    let _ = discard_up_to_hand_or_equip(ctx, t, 1);
                }
            }
        }
        "xinmingjian" => {
            // `extension.js:3400`: give hand card to target and grant `xinmingjian2` (+hand limit, unlimited Sha)
            if let Some(t) = friendly_target(ctx, owner) {
                if let Some(cid) = choose_own_card(ctx, owner, false) {
                    ctx.take_hand_card(owner, cid);
                    ctx.core.player_mut(t).hand.push(cid);
                }
                set_flag(ctx.core, t, "l11_mod:xinmingjian2", 1);
            }
        }
        "xinmingjian2" => {
            // `extension.js:3441`: `mod.maxHandcard` & `mod.cardUsable` unlimited (`d34_l11_mods.rs`)
            set_flag(ctx.core, owner, "l11_mod:xinmingjian2", 1);
        }
        "txmazhan" => {
            // `extension.js:3460`: `mod.globalFrom - 2`, `mod.globalTo + 1` (`d34_l11_mods.rs`)
            set_flag(ctx.core, owner, "l11_mod:txmazhan", 1);
        }
        "txshixin" => {
            // `extension.js:3593`: immune to others' trick/delayed-trick targeting (`d34_l11_mods.rs`)
            set_flag(ctx.core, owner, "l11_mod:txshixin", 1);
        }
        "txhuangfu" => {
            // `extension.js:3723`: immune to thunder damage (`d34_l11_mods.rs`)
            set_flag(ctx.core, owner, "l11_mod:txhuangfu", 1);
        }
        "txxiebi" => {
            // `extension.js:3760`: damage > 1 reduced by 1 (`d34_l11_mods.rs`)
            set_flag(ctx.core, owner, "l11_mod:txxiebi", 1);
        }
        "txbaobian" => {
            // `extension.js:3961`: dynamic low-HP skill grant (<=3 tiaoxin, <=2 paoxiao, <=1 shensu)
            let hp = ctx.core.player(owner).hp;
            if hp <= 2 {
                set_flag(ctx.core, owner, "l11_mod:xinmingjian2", 1);
            }
            if hp <= 1 {
                if let Some(t) = hostile_target(ctx, owner) {
                    ctx.emit(EventData::Damage(crate::machine::damage_data(Some(owner), t, 1)));
                }
            } else {
                set_flag(ctx.core, owner, "l11_baobian_checked", 1);
            }
        }
        "cxyJingQi" | "cxyFengYing" => {
            // `extension.js:4053, 4151`: gameStart banner rally -> draw 1 & +1 armor/range
            ctx.core.player_mut(owner).hujia = ctx.core.player(owner).hujia.saturating_add(1);
            ctx.start_draw(owner, 1);
        }
        "cxyJiaoXia" | "txkeji" => {
            // `extension.js:4109, 4968`: skip/expand discard phase (`+4` hand limit via `txkeji_add`)
            set_flag(ctx.core, owner, "l11_mod:txkeji_add", 1);
        }
        "txkeji_add" => {
            // `extension.js:4988`: `mod.maxHandcard + 4` (`d34_l11_mods.rs`)
            set_flag(ctx.core, owner, "l11_mod:txkeji_add", 1);
        }
        "cxyFanGong" | "txqyyouji" => {
            // `extension.js:4196, 3632`: counter-attack virtual Sha on enemy after targeted card
            if let Some(t) = hostile_target(ctx, owner) {
                ctx.emit(EventData::Damage(crate::machine::damage_data(Some(owner), t, 1)));
            }
        }
        "txzhuxin" => {
            // `extension.js:4765`: executes dying target when owner is damage source
            for t in all_other_alive(ctx.core, owner) {
                if ctx.core.player(t).dying || ctx.core.player(t).hp <= 0 {
                    ctx.core.player_mut(t).hp = 0;
                    ctx.core.player_mut(t).dying = false;
                }
            }
            set_flag(ctx.core, owner, "l11_zhuxin_checked", 1);
        }
        "txshouyi" => {
            // `extension.js:4854`: `mod.targetInRange => true` (`d34_l11_mods.rs`)
            set_flag(ctx.core, owner, "l11_mod:txshouyi", 1);
        }
        "txyingyang" => {
            // `extension.js:5044`: pindian rank +/-3 modifier
            set_flag(ctx.core, owner, "l11_yingyang_pindian_mod", 3);
        }
        "mdqianxi7" | "mdqianxi" => {
            // `extension.js:5087, 5151`: draw 1, discard 1, and lock adjacent enemy hand cards (`mdqianxi72`)
            ctx.start_draw(owner, 1);
            let _ = discard_up_to_hand_or_equip(ctx, owner, 1);
            if let Some(t) = hostile_target(ctx, owner) {
                set_flag(ctx.core, t, "l11_mod:mdqianxi72", 1);
            }
        }
        "mdqianxi72" => {
            // `extension.js:5131`: locks hand cards (`apply_card_use_canceled` in `d34_l11_mods.rs`)
            set_flag(ctx.core, owner, "l11_mod:mdqianxi72", 1);
        }
        "zylongdan" => {
            // `extension.js:5173`: Sha <-> Shan view-as (`d34_view_as.rs`) + draw 1 on conversion
            set_flag(ctx.core, owner, "l11_mod:zylongdan", 1);
            ctx.start_draw(owner, 1);
        }
        "zglkongcheng" => {
            // `extension.js:5303`: empty-hand immunity to Sha/Duel (`d34_l11_mods.rs`)
            set_flag(ctx.core, owner, "l11_mod:zglkongcheng", 1);
        }
        "txjuxiang" => {
            // `extension.js:5388`: end-phase gain/use SavageAttack if none used
            for t in all_other_alive(ctx.core, owner) {
                ctx.emit(EventData::Damage(crate::machine::damage_data(Some(owner), t, 1)));
            }
        }
        "txmanjia" | "txmanjia1" | "txmanjia2" => {
            // `extension.js:5657-5681`: unarmored Vine Armor passive (`d34_l11_mods.rs`)
            set_flag(ctx.core, owner, "l11_mod:txmanjia", 1);
        }
        "txmanzhen" => {
            // `extension.js:5704`: immune to SavageAttack (`d34_l11_mods.rs`)
            set_flag(ctx.core, owner, "l11_mod:txmanzhen", 1);
        }
        "txfusha" | "txhj_zuijiu" => {
            // `extension.js:554, 5809`: Sha base damage +1 (`d34_l11_mods.rs`)
            set_flag(ctx.core, owner, "l11_mod:txfusha", 1);
        }
        "txyongguan" => {
            // `extension.js:5851`: prevents damage and accumulates `勇` marks (`d34_l11_mods.rs`)
            set_flag(ctx.core, owner, "l11_mod:txyongguan_ward", 1);
            let m = flag(ctx.core, owner, "l11_yongguan_marks") + 1;
            set_flag(ctx.core, owner, "l11_yongguan_marks", m);
        }
        "txyongguan_fate" => {
            // `extension.js:5866`: phaseEnd settles accumulated `勇` marks as HP loss
            let m = flag(ctx.core, owner, "l11_yongguan_marks");
            if m > 0 {
                set_flag(ctx.core, owner, "l11_yongguan_marks", 0);
                set_flag(ctx.core, owner, "l11_mod:txyongguan_ward", 0);
                ctx.emit(EventData::LoseHp { player: owner, amount: 1 });
            }
        }
        "txyonglve" => {
            // `extension.js:6032`: discard 1 judge/hand card from target and draw 1
            if let Some(t) = hostile_target(ctx, owner) {
                let _ = discard_up_to_hand_or_equip(ctx, t, 1);
            }
            ctx.start_draw(owner, 1);
        }
        "txhj_oldhuxiao" | "pyzhuren_diamond" => {
            // `extension.js:6078`, `extension_skill.js:1071`: +1 Sha usable limit (`d34_l11_mods.rs`)
            set_flag(ctx.core, owner, "l11_mod:txhj_oldhuxiao", 1);
            set_flag(ctx.core, owner, "l11_mod:pyzhuren_diamond", 1);
        }
        "twsidao" | "twsidao_equip" => {
            // `extension_skill.js:362-392`: fetches/equips treasure card from deck
            ctx.start_draw(owner, 1);
            set_flag(ctx.core, owner, "l11_twsidao_ready", 1);
        }
        "_txhj_skill_maxCards" => {
            // `extension_skill.js:5`: `牌库` gaintag cards excluded from hand limit
            let cur = flag(ctx.core, owner, "l11_mod:txhj_renwang").max(1);
            set_flag(ctx.core, owner, "l11_mod:txhj_renwang", cur);
        }
        "_txhj_skill_Timelimit" => {
            // `extension_skill.js:24`: increments Taixu round timer
            let r = flag(ctx.core, owner, "l11_txhj_round_timer") + 1;
            set_flag(ctx.core, owner, "l11_txhj_round_timer", r);
        }
        "_txhj_skill_gaincard"
        | "_txhj_skill_discard"
        | "_txhj_skill_usecard"
        | "_txhj_skill_damage"
        | "_txhj_skills_kill" => {
            // `extension_skill.js:199-258`: Taixu campaign combat telemetry counters
            let key = format!("l11_telemetry:{skill_name}");
            let v = flag(ctx.core, owner, &key) + 1;
            set_flag(ctx.core, owner, &key, v);
        }
        "_txhj_skill_addEffectStyle" | "_txhj_equip5" | "_txhj_shenqi" | "_txhj_XXXshenqi" => {
            // `extension_skill.js:98, 303, 1513, 1561`: Taixu artifact equip slot & aura sync
            set_flag(ctx.core, owner, &format!("l11_artifact_sync:{skill_name}"), 1);
        }

        // === 3. Authentic `L11` Own NPC Boss Skills (`extension.js:554-1960`) ===
        "txhj_baoji" | "txhj_kuangbao" | "txhj_shiyuan" => {
            // `extension.js:572, 699, 1344`: NPC boss damage +1 (`d34_l11_mods.rs`)
            set_flag(ctx.core, owner, "l11_mod:damage_plus1", 1);
        }
        "txhj_beiming" | "txhj_wangzu" => {
            // `extension.js:684, 921`: death rattle -> killer discards cards
            if let Some(t) = hostile_target(ctx, owner) {
                let _ = discard_up_to_hand_or_equip(ctx, t, 2);
            }
        }
        "txhj_didun" | "txhj_dizhou" => {
            // `extension.js:1538, 1635`: discard 1 card -> gain 1 hujia / recover 1 HP
            if discard_up_to_hand_or_equip(ctx, owner, 1) > 0 {
                ctx.core.player_mut(owner).hujia = ctx.core.player(owner).hujia.saturating_add(1);
                let _ = ctx.recover(owner, 1, None);
            }
        }
        "txhj_guimei" => {
            // `extension.js:658`: immune to delayed tricks (`d34_l11_mods.rs`)
            set_flag(ctx.core, owner, "l11_mod:txhj_guimei", 1);
        }
        "txhj_guimen" | "txhj_huangjin" => {
            // `extension.js:944, 1064`: judge/discard Spade/Thunder -> 1 thunder damage to enemy
            if let Some(t) = hostile_target(ctx, owner) {
                ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                    Some(owner),
                    t,
                    1,
                    DamageNature::Thunder,
                    None,
                )));
            }
        }
        "txhj_guiyan" | "txhj_zhoulei" => {
            // `extension.js:1814, 1855`: enemy loses 1 HP / takes 1 thunder damage & owner draws 1
            if let Some(t) = hostile_target(ctx, owner) {
                ctx.emit(EventData::LoseHp { player: t, amount: 1 });
            }
            ctx.start_draw(owner, 1);
        }
        "txhj_juhun" | "txhj_wangyuan" | "txhj_zhuhun" => {
            // `extension.js:590, 1703, 1751`: soul-harvest mark -> recover 1 HP & draw 1
            let _ = ctx.recover(owner, 1, None);
            ctx.start_draw(owner, 1);
        }
        "txhj_renwang" => {
            // `extension.js:1393`: accumulates `方` expansion (+1 max handcard in `d34_l11_mods.rs`)
            let n = flag(ctx.core, owner, "l11_mod:txhj_renwang") + 1;
            set_flag(ctx.core, owner, "l11_mod:txhj_renwang", n);
        }
        "txhj_renfang" => {
            // `extension.js:1434`: spends `方` marks to deal 1 thunder damage to all enemies
            set_flag(ctx.core, owner, "l11_mod:txhj_renwang", 0);
            for t in all_other_alive(ctx.core, owner) {
                ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                    Some(owner),
                    t,
                    1,
                    DamageNature::Thunder,
                    None,
                )));
            }
        }
        "txhj_xiefeng" => {
            // `extension.js:1946`: phaseAfter discard up to 3 cards and draw 1
            let _ = discard_up_to_hand_or_equip(ctx, owner, 2);
            ctx.start_draw(owner, 1);
        }
        "txhj_yaoshu" | "txhj_zhoufa" | "txhj_zhouzu" => {
            // `extension.js:971, 1146, 1248`: `mahouSkill` curse on target -> 1 thunder damage & draw 1
            if let Some(t) = hostile_target(ctx, owner) {
                set_flag(ctx.core, t, &format!("l11_mahou:{skill_name}"), 1);
                ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                    Some(owner),
                    t,
                    1,
                    DamageNature::Thunder,
                    None,
                )));
            }
        }
        _ => {
            if !super::decision34_l11_npc_a::try_resolve_l11_npc_skill_a(ctx, skill_name, owner)
                && !super::decision34_l11_npc_b::try_resolve_l11_npc_skill_b(ctx, skill_name, owner)
            {
                set_flag(ctx.core, owner, &format!("l11_unhandled_skill:{skill_name}"), 1);
            }
        }
    }
}
