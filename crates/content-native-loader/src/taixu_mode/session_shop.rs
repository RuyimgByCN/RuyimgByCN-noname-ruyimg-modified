//! L11 太虚幻境商店库存生成、阶梯涨价（含「商贾之道」减免）与购买结账（`session_shop.rs`）。

use super::catalog::{L11_TAIXU_BUFF_STORE, L11_TAIXU_CARD_RANK};
use super::session::TaixuModeSession;
use super::session_types::{TaixuPendingInteraction, TaixuShopCategory, TaixuShopStock, TaixuSplitMix64};
use super::skill_rank::L11_TAIXU_SKILL_RANK;
use crate::taixu::TaixuError;

impl TaixuModeSession {
    /// 商店价格（`lookStore`：`base_value + 200 * max(purchased - dec, 0)`，`商贾之道` `dec = 1`）。
    pub fn shop_item_price(&self, shop_id: &str, cat: TaixuShopCategory, item_id: &str) -> Option<i32> {
        let stock = self.stores.get(shop_id)?;
        let dec = u32::from(self.buffs.iter().any(|b| b == "buff_txhj_juguzhidao"));
        let (base, purchased) = match cat {
            TaixuShopCategory::Buff => {
                let b = L11_TAIXU_BUFF_STORE.iter().find(|b| b.id == item_id)?;
                (b.value as i32, stock.buff_num)
            }
            TaixuShopCategory::Card | TaixuShopCategory::Equip => {
                let c = L11_TAIXU_CARD_RANK.iter().find(|c| c.card_id == item_id)?;
                let n = if cat == TaixuShopCategory::Card {
                    stock.card_num
                } else {
                    stock.equip_num
                };
                (c.value as i32, n)
            }
            TaixuShopCategory::Skill => {
                let s = L11_TAIXU_SKILL_RANK.iter().find(|s| s.skill_id == item_id)?;
                (s.value as i32, stock.skill_num)
            }
        };
        Some(base + (purchased.saturating_sub(dec) as i32) * 200)
    }

    /// 在当前打开的商店购买一件商品。
    pub fn buy_shop_item(&mut self, cat: TaixuShopCategory, item_idx: usize) -> Result<i32, TaixuError> {
        let Some(TaixuPendingInteraction::Shop { shop_id, .. }) = self.pending.clone() else {
            return Err(TaixuError::UnknownCommand("no pending shop".to_string()));
        };
        let item_id = {
            let stock = self
                .stores
                .get(&shop_id)
                .ok_or_else(|| TaixuError::UnknownCommand(format!("missing shop {shop_id}")))?;
            match cat {
                TaixuShopCategory::Buff => stock.buffs.get(item_idx).cloned(),
                TaixuShopCategory::Card => stock.cards.get(item_idx).cloned(),
                TaixuShopCategory::Equip => stock.equips.get(item_idx).cloned(),
                TaixuShopCategory::Skill => stock.skills.get(item_idx).cloned(),
            }
            .ok_or_else(|| TaixuError::UnknownCommand(format!("shop idx {item_idx} oob")))?
        };
        let price = self
            .shop_item_price(&shop_id, cat, &item_id)
            .ok_or_else(|| TaixuError::UnknownCommand(format!("price missing for {item_id}")))?;
        if self.coin < price {
            return Err(TaixuError::InsufficientGold {
                have: self.coin.max(0) as u32,
                need: price.max(0) as u32,
            });
        }
        self.coin -= price;
        let stock = self.stores.get_mut(&shop_id).unwrap();
        match cat {
            TaixuShopCategory::Buff => {
                stock.buff_num += 1;
                stock.buffs.remove(item_idx);
                if !self.buffs.contains(&item_id) {
                    self.buffs.push(item_id.clone());
                }
            }
            TaixuShopCategory::Card => {
                stock.card_num += 1;
                stock.cards.remove(item_idx);
                self.cards.push(item_id.clone());
            }
            TaixuShopCategory::Equip => {
                stock.equip_num += 1;
                stock.equips.remove(item_idx);
                self.acquire_equip(&item_id);
            }
            TaixuShopCategory::Skill => {
                stock.skill_num += 1;
                stock.skills.remove(item_idx);
                self.acquire_skill(&item_id);
            }
        }
        *self.collect_cards.entry(item_id).or_insert(0) += 1;
        Ok(price)
    }

    pub fn leave_shop(&mut self) -> Result<(), TaixuError> {
        let Some(TaixuPendingInteraction::Shop { slot_idx, shop_id }) = self.pending.take() else {
            return Err(TaixuError::UnknownCommand("no pending shop".to_string()));
        };
        if !self.seen_events.contains(&shop_id) {
            self.seen_events.push(shop_id);
        }
        self.consume_slot(slot_idx);
        Ok(())
    }

    pub(super) fn ensure_shop_stock(&mut self, shop_id: &str, level: u8) {
        if self.stores.contains_key(shop_id) {
            return;
        }
        let mut cards_pool: Vec<&'static str> = Vec::new();
        let mut equip_pool: Vec<&'static str> = Vec::new();
        for c in &L11_TAIXU_CARD_RANK {
            let match_rank = match level {
                1 => c.rank < 3,
                2 => c.rank < 4,
                3 | 4 => c.rank > 1,
                _ => c.rank <= 5,
            };
            if match_rank {
                if c.category.is_equip() {
                    equip_pool.push(c.card_id);
                } else {
                    cards_pool.push(c.card_id);
                }
            }
        }
        let mut skill_pool: Vec<&'static str> = Vec::new();
        for s in &L11_TAIXU_SKILL_RANK {
            if let Some(ref flt) = self.available_skills_filter {
                if !flt.iter().any(|x| x == s.skill_id) {
                    continue;
                }
            }
            if self.use_skills.iter().any(|x| x == s.skill_id) {
                continue;
            }
            let match_rank = match level {
                1 => s.rank < 3,
                2 => s.rank < 4,
                3 | 5 => s.rank > 1,
                _ => s.rank > 2,
            };
            if match_rank {
                skill_pool.push(s.skill_id);
            }
        }
        let mut buff_pool: Vec<&'static str> = Vec::new();
        for b in &L11_TAIXU_BUFF_STORE {
            if !b.store || self.buffs.iter().any(|x| x == b.id) {
                continue;
            }
            let match_rank = match level {
                1 => b.rank <= 3,
                2 | 5 => b.rank <= 4,
                _ => b.rank > 1,
            };
            if match_rank {
                buff_pool.push(b.id);
            }
        }

        let (nc, ne, nb, ns) = match level {
            1 => (4, 4, 0, 0),
            2 => (0, 0, 4, 4),
            3 => (3, 3, 3, 3),
            4 => (6, 6, 6, 6),
            _ => (2, 5, 3, 5),
        };
        let draw_n = |rng: &mut TaixuSplitMix64, pool: &mut Vec<&'static str>, cnt: usize| {
            let mut out = Vec::new();
            while out.len() < cnt && !pool.is_empty() {
                let idx = rng.gen_range(pool.len());
                out.push(pool.swap_remove(idx).to_string());
            }
            out
        };
        let cards = draw_n(&mut self.rng, &mut cards_pool, nc);
        let equips = draw_n(&mut self.rng, &mut equip_pool, ne);
        let buffs = draw_n(&mut self.rng, &mut buff_pool, nb);
        let skills = draw_n(&mut self.rng, &mut skill_pool, ns);
        self.stores.insert(
            shop_id.to_string(),
            TaixuShopStock {
                level,
                card_num: 0,
                buff_num: 0,
                skill_num: 0,
                equip_num: 0,
                buffs,
                cards,
                equips,
                skills,
            },
        );
    }
}
