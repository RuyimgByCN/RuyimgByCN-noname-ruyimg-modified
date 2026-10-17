//! L13 手杀MVP（noname-liuli）。结算统计与结算分组走调用方给出的**权威
//! 事实**（用牌/伤害/获得/失去/回复/死亡）与身份关系表：
//!
//! - MVP 计分（上游 `_shoushajiesuan-effect1..4`）：攻击 / 治疗 / 辅助 /
//!   惩罚四项 + 基础分 100（结算时按阵亡与敌友阵亡数调整），总分
//!   `攻 + 疗 + 辅 + 基 − 罚`；
//! - 战绩统计（上游结算页第二张表）：伤害 / 受伤 / 摸牌 / 出牌 / 杀敌；
//! - 结算分组（上游 `game.over` 内 victory/fail/tie 分类，`jiesuanfix`
//!   开关两条口径）与 MVP 排序（冒泡升序后整体反转）。
//!
//! 边界（登记差异见证据）：不替换内核终局（胜负由内核给出，本模块只
//! 订阅事实并去重入账）、不画界面 / 不播音 / 不检测作弊输入 / 不生成
//! 随机名单、段位与金币不改写；态度与敌友由调用方给出（原生不建 `get.attitude`
//! 模型）。事实按 `seq` 单调去重：同一事实重复投递（录像 / 恢复 / 重放）
//! 只入账一次。

use std::collections::{BTreeMap, BTreeSet};

use serde_json::Value;

/// 事实去重与入账状态。
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MvpScore {
    /// 攻击（effect1：伤害来源 / 伤害牌 / 多目标锦囊不在此项）。
    pub attack: i64,
    /// 治疗（effect3：回复来源）。
    pub heal: i64,
    /// 辅助（effect1 无懈与多目标锦囊 + effect2 获得/失去牌）。
    pub support: i64,
    /// 惩罚（effect4：击杀友方）。
    pub penalty: i64,
    /// 基础分（初始 100；结算时按阵亡/敌友阵亡调整）。
    pub base: i64,
}

impl MvpScore {
    pub fn new() -> Self {
        Self {
            attack: 0,
            heal: 0,
            support: 0,
            penalty: 0,
            base: 100,
        }
    }

    /// MVP 总分（上游 `JieSuanmvpCount`）。
    pub fn total(&self) -> i64 {
        self.attack + self.heal + self.support + self.base - self.penalty
    }
}

/// 战绩统计（上游结算页第二张表的五列）。
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MvpStats {
    pub damage: i64,
    pub damaged: i64,
    pub gain: i64,
    pub play: i64,
    pub kill: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MvpPlayer {
    pub id: u32,
    pub name: String,
    pub identity: String,
    pub alive: bool,
    /// 结算时的敌方集合（上游 `getEnemies()`；由调用方给出）。
    pub enemies: BTreeSet<u32>,
    /// 结算时的友方集合（上游 `getFriends()`；由调用方给出）。
    pub friends: BTreeSet<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MvpConfig {
    pub source: String,
    pub upstream_path: String,
    pub upstream_commit: String,
    pub package_name: String,
    pub package_version: String,
    pub author: String,
    /// `extension_手杀MVP_shoushajiesuan`（主开关；关闭=整块不登记）。
    pub enabled: bool,
    /// `extension_手杀MVP_jiesuanfix`（结算分组新版代码）。
    pub jiesuan_fix: bool,
    /// `extension_手杀MVP_yangshi`（'on'/'off'；样式面登记差异）。
    pub yangshi: String,
    /// `extension_手杀MVP_oneMore`（再来一局按钮；界面面登记差异）。
    pub one_more: bool,
    /// `qingyao_shoushapeiyin`（手杀配音；音频面登记差异）。
    pub style_audio: bool,
    /// 结算面板列名（数据面；DOM 渲染登记差异）。
    pub labels: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MvpError {
    Json(String),
    Missing(&'static str),
    BadField(&'static str),
    UnknownPlayer(u32),
    UnknownFact(String),
    BadState(String),
}

/// 一条权威事实。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MvpFact {
    /// 使用牌（effect1 使用面）。
    UseCard {
        seq: u64,
        player: u32,
        damage_tag: bool,
        wuxie: bool,
        trick: bool,
        to_self: bool,
        /// 上游 `selectTarget`；-1 = 不限/全体。
        select_target: i64,
    },
    /// 造成伤害（effect1 伤害来源面 + 战绩「伤害/受伤」）。
    Damage {
        seq: u64,
        source: u32,
        target: u32,
        num: u32,
        attitude_negative: bool,
        source_is_nei: bool,
    },
    /// 获得牌（effect2 获得面 + 战绩「摸牌」）。
    Gain {
        seq: u64,
        player: u32,
        source: u32,
        cards: u32,
        source_alive: bool,
        source_is_nei: bool,
        player_enemy: bool,
    },
    /// 失去/弃置牌（effect2 弃置面）。
    Discard {
        seq: u64,
        player: u32,
        source: u32,
        cards: u32,
        source_alive: bool,
        source_is_nei: bool,
        player_enemy: bool,
    },
    /// 回复体力（effect3）。
    Recover {
        seq: u64,
        player: u32,
        source: u32,
        num: u32,
        source_alive: bool,
        source_is_nei: bool,
        friend_or_self: bool,
    },
    /// 死亡（effect4 + 战绩「杀敌」）。
    Die {
        seq: u64,
        victim: u32,
        source: u32,
        source_alive: bool,
        source_is_nei: bool,
        victim_is_lord: bool,
        /// 死者视来源为友方（上游 `player.getFriends().contains(source)`）。
        victim_friend: bool,
        /// 死者视来源为敌方（上游 `player.getEnemies().contains(source)`）。
        victim_enemy: bool,
    },
}

impl MvpFact {
    pub fn seq(&self) -> u64 {
        match self {
            MvpFact::UseCard { seq, .. }
            | MvpFact::Damage { seq, .. }
            | MvpFact::Gain { seq, .. }
            | MvpFact::Discard { seq, .. }
            | MvpFact::Recover { seq, .. }
            | MvpFact::Die { seq, .. } => *seq,
        }
    }
}

/// 结算结果：分组 + 排序 + MVP。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MvpSettlement {
    pub victory: Vec<u32>,
    pub fail: Vec<u32>,
    pub tie: Vec<u32>,
    /// 总分降序（并列时按上游「冒泡升序后反转」的逆输入序）。
    pub ranking: Vec<u32>,
    /// 总分最高者（上游 `sorts[0]`）。
    pub mvp: Option<u32>,
    pub scores: BTreeMap<u32, MvpScore>,
}

pub struct MvpSession {
    config: MvpConfig,
    players: Vec<MvpPlayer>,
    scores: BTreeMap<u32, MvpScore>,
    stats: BTreeMap<u32, MvpStats>,
    last_seq: u64,
    applied: u64,
    duplicates: u64,
    gated: u64,
    settled: Option<MvpSettlement>,
}

impl MvpSession {
    pub fn new(config: MvpConfig, players: Vec<MvpPlayer>) -> Self {
        let scores = players.iter().map(|p| (p.id, MvpScore::new())).collect();
        let stats = players.iter().map(|p| (p.id, MvpStats::default())).collect();
        Self {
            config,
            players,
            scores,
            stats,
            last_seq: 0,
            applied: 0,
            duplicates: 0,
            gated: 0,
            settled: None,
        }
    }

    pub fn config(&self) -> &MvpConfig {
        &self.config
    }

    pub fn scores(&self) -> &BTreeMap<u32, MvpScore> {
        &self.scores
    }

    pub fn stats(&self) -> &BTreeMap<u32, MvpStats> {
        &self.stats
    }

    pub fn settled(&self) -> Option<&MvpSettlement> {
        self.settled.as_ref()
    }

    /// 投递一条事实。返回 `Ok(true)` = 本次入账；`Ok(false)` = 主开关关闭
    /// 或 `seq` 不新（录像/恢复/重放重复投递）。
    pub fn apply(&mut self, fact: MvpFact) -> Result<bool, MvpError> {
        if !self.config.enabled {
            self.gated += 1;
            return Ok(false);
        }
        if fact.seq() <= self.last_seq {
            self.duplicates += 1;
            return Ok(false);
        }
        let seq = fact.seq();
        // 序号只在事实**完整入账**后推进：中途报错的事实不消费序号
        //（否则错误事实会把后续合法事实误判为重复）。
        match fact {
            MvpFact::UseCard {
                player,
                damage_tag,
                wuxie,
                trick,
                to_self,
                select_target,
                ..
            } => {
                let multi = select_target == -1 || select_target > 1;
                // 上游 filter：伤害牌 / 无懈 / 非自目标多目标锦囊。
                let triggers = damage_tag || wuxie || (!to_self && trick && multi);
                if triggers {
                    let score = self.score_mut(player)?;
                    if damage_tag {
                        score.attack += 2;
                    }
                    if wuxie {
                        score.support += 2;
                    }
                    if multi && !to_self && trick {
                        score.support += 1;
                    }
                }
                self.stat_mut(player)?.play += 1;
            }
            MvpFact::Damage {
                source,
                target,
                num,
                attitude_negative,
                source_is_nei,
                ..
            } => {
                if num > 0 {
                    self.stat_mut(source)?.damage += num as i64;
                    self.stat_mut(target)?.damaged += num as i64;
                }
                // 上游 filter：来源≠承受者且（内奸或态度为负）。
                if source != target && (source_is_nei || attitude_negative) {
                    let score = self.score_mut(source)?;
                    if attitude_negative || source_is_nei {
                        if num > 5 {
                            score.attack += 15;
                        } else {
                            score.attack += 3 * num as i64;
                        }
                    }
                }
            }
            MvpFact::Gain {
                player,
                source,
                cards,
                source_alive,
                source_is_nei,
                player_enemy,
                ..
            } => {
                if cards > 0 {
                    self.stat_mut(player)?.gain += cards as i64;
                }
                // 上游 filter：来源存在、非本人、存活、有牌，且（内奸或敌方）。
                if source_alive
                    && source != player
                    && cards > 0
                    && (source_is_nei || player_enemy)
                {
                    self.score_mut(player)?.support += cards as i64;
                }
            }
            MvpFact::Discard {
                player,
                source,
                cards,
                source_alive,
                source_is_nei,
                player_enemy,
                ..
            } => {
                if source_alive
                    && source != player
                    && cards > 0
                    && (source_is_nei || player_enemy)
                {
                    self.score_mut(source)?.support += cards as i64;
                }
            }
            MvpFact::Recover {
                player,
                source,
                num,
                source_alive,
                source_is_nei,
                friend_or_self,
                ..
            } => {
                if source_alive && (source_is_nei || friend_or_self || player == source) {
                    let score = self.score_mut(source)?;
                    if num > 5 {
                        score.heal += 10;
                    } else {
                        score.heal += 2 * num as i64;
                    }
                }
            }
            MvpFact::Die {
                victim,
                source,
                source_alive,
                source_is_nei,
                victim_is_lord,
                victim_friend,
                victim_enemy,
                ..
            } => {
                if !self.scores.contains_key(&victim) {
                    return Err(MvpError::UnknownPlayer(victim));
                }
                if source_alive {
                    self.stat_mut(source)?.kill += 1;
                    if victim_friend {
                        let score = self.score_mut(source)?;
                        score.penalty += 5;
                    }
                    if victim_friend && source_is_nei && !victim_is_lord {
                        let score = self.score_mut(source)?;
                        score.penalty -= 5;
                        score.attack += 3;
                    }
                    if victim_enemy {
                        self.score_mut(source)?.attack += 3;
                    }
                }
            }
        }
        self.last_seq = seq;
        self.applied += 1;
        Ok(true)
    }

    fn score_mut(&mut self, id: u32) -> Result<&mut MvpScore, MvpError> {
        self.scores.get_mut(&id).ok_or(MvpError::UnknownPlayer(id))
    }

    fn stat_mut(&mut self, id: u32) -> Result<&mut MvpStats, MvpError> {
        self.stats.get_mut(&id).ok_or(MvpError::UnknownPlayer(id))
    }

    /// 结算：基础分调整 + 分组 + 排序 + MVP。**只结算一次**：重复调用
    /// 返回首次结果（恢复 / 录像不重复入账）。
    pub fn settle(
        &mut self,
        result: &str,
        viewer: u32,
        mode: &str,
    ) -> Result<MvpSettlement, MvpError> {
        if let Some(prev) = &self.settled {
            return Ok(prev.clone());
        }
        if !self.config.enabled {
            let settlement = MvpSettlement {
                victory: Vec::new(),
                fail: Vec::new(),
                tie: Vec::new(),
                ranking: Vec::new(),
                mvp: None,
                scores: self.scores.clone(),
            };
            self.settled = Some(settlement.clone());
            return Ok(settlement);
        }
        // 基础分调整（上游结算段）：阵亡 -20；敌方阵亡 +2；友方阵亡 -2。
        for player in &self.players {
            let mut base = 100;
            if !player.alive {
                base -= 20;
            }
            for enemy in &player.enemies {
                if self.players.iter().any(|p| p.id == *enemy && !p.alive) {
                    base += 2;
                }
            }
            for friend in &player.friends {
                if self.players.iter().any(|p| p.id == *friend && !p.alive) {
                    base -= 2;
                }
            }
            if let Some(score) = self.scores.get_mut(&player.id) {
                score.base = base;
            }
        }
        let (victory, fail, tie) = self.classify(result, viewer, mode)?;
        // 排序：总分升序稳定排序后整体反转（上游冒泡升序 + reverse）；
        // 基准序 = 存活者在前、阵亡者在后（`players.concat(dead)`）。
        let mut ranking: Vec<u32> = self.ordered_ids();
        let totals = |id: u32| self.scores.get(&id).map(|s| s.total()).unwrap_or(0);
        ranking.sort_by_key(|id| totals(*id));
        ranking.reverse();
        let mvp = ranking.first().copied();
        let settlement = MvpSettlement {
            victory,
            fail,
            tie,
            ranking,
            mvp,
            scores: self.scores.clone(),
        };
        self.settled = Some(settlement.clone());
        Ok(settlement)
    }

    /// 上游 `getIdeOf`：主/忠/明忠折叠为 `zhu`；`strict=false` 时内奸按
    /// 座位唯一化（`nei<id>`）。
    fn ide(identity: &str, id: u32, only: bool) -> String {
        let folded = match identity {
            "zhu" | "zhong" | "mingzhong" => "zhu",
            other => other,
        };
        if folded == "nei" && !only {
            format!("nei{id}")
        } else {
            folded.to_string()
        }
    }

    fn identity_of(&self, id: u32) -> String {
        self.players
            .iter()
            .find(|p| p.id == id)
            .map(|p| p.identity.clone())
            .unwrap_or_default()
    }

    fn ordered_ids(&self) -> Vec<u32> {
        let mut out: Vec<u32> = self.players.iter().filter(|p| p.alive).map(|p| p.id).collect();
        out.extend(self.players.iter().filter(|p| !p.alive).map(|p| p.id));
        out
    }

    fn player(&self, id: u32) -> Result<&MvpPlayer, MvpError> {
        self.players
            .iter()
            .find(|p| p.id == id)
            .ok_or(MvpError::UnknownPlayer(id))
    }

    fn is_alive(&self, id: u32) -> bool {
        self.players
            .iter()
            .find(|p| p.id == id)
            .map(|p| p.alive)
            .unwrap_or(false)
    }

    /// 结算分组（上游 `game.over` 内 victory/fail/tie；`jiesuanfix` 两条
    /// 口径）。返回（胜、负、平）三组，组内按玩家输入序。
    fn classify(
        &self,
        result: &str,
        viewer: u32,
        mode: &str,
    ) -> Result<(Vec<u32>, Vec<u32>, Vec<u32>), MvpError> {
        let mut victory = Vec::new();
        let mut fail = Vec::new();
        let mut tie = Vec::new();
        let me = self.identity_of(viewer);
        if me.is_empty() {
            return Err(MvpError::UnknownPlayer(viewer));
        }
        let alive_ids: Vec<u32> = self
            .players
            .iter()
            .filter(|p| p.alive)
            .map(|p| p.id)
            .collect();
        // 上游分组与排序都按「存活者在前、阵亡者在后」遍历。
        let ordered = self.ordered_ids();
        // 存活者原始身份表 + 折叠计数（上游 cunhuoPlayerGroup/Groups）。
        let survivor_identities: Vec<String> = alive_ids
            .iter()
            .map(|id| self.identity_of(*id))
            .collect();
        let mut survivor_groups: BTreeMap<String, i64> = BTreeMap::new();
        for id in &alive_ids {
            let key = Self::ide(&self.identity_of(*id), *id, true);
            // 上游按 `getIdeOf(item)`（内奸唯一键）读计数——内奸计数恒 1。
            let read_key = Self::ide(&self.identity_of(*id), *id, false);
            let count = survivor_groups.get(&read_key).copied().unwrap_or(0);
            survivor_groups.insert(key, count + 1);
        }
        let group_has =
            |roles: &[&str]| survivor_identities.iter().any(|r| roles.contains(&r.as_str()));
        let nei_count = survivor_groups.get("nei").copied().unwrap_or(0);

        if mode == "guozhan" {
            let won = result.contains("战斗胜利") || result.contains("战斗失败");
            for id in &ordered {
                if self.is_alive(*id) {
                    if won {
                        victory.push(*id);
                    } else {
                        tie.push(*id);
                    }
                } else {
                    fail.push(*id);
                }
            }
            return Ok((victory, fail, tie));
        }

        if result.contains("战斗胜利") {
            for id in &ordered {
                let player = self.player(*id)?;
                let same = player.identity == me
                    || (me == "zhu" && matches!(player.identity.as_str(), "zhong" | "mingzhong"))
                    || (matches!(me.as_str(), "zhong" | "mingzhong") && player.identity == "zhu");
                let win = if self.config.jiesuan_fix {
                    Self::ide(&me, viewer, false) == Self::ide(&player.identity, *id, false)
                } else {
                    same && (*id == viewer || me != "nei")
                };
                if win {
                    victory.push(*id);
                } else {
                    fail.push(*id);
                }
            }
            return Ok((victory, fail, tie));
        }

        if result.contains("战斗失败") {
            for id in &ordered {
                let player = self.player(*id)?;
                if self.config.jiesuan_fix {
                    if Self::ide(&me, viewer, false) == Self::ide(&player.identity, player.id, false)
                    {
                        fail.push(player.id);
                        continue;
                    }
                    if Self::classify_loser(&me, viewer, &player.identity, player.id, &group_has, nei_count) {
                        victory.push(player.id);
                    }
                } else if player.alive {
                    let same = player.identity == me
                        || (me == "zhu" && matches!(player.identity.as_str(), "zhong" | "mingzhong"));
                    if same && !(player.id != viewer && player.identity == "nei") {
                        fail.push(player.id);
                    } else if survivor_identities.contains(&player.identity)
                        && !(group_has(&["fan"])
                            && matches!(player.identity.as_str(), "zhong" | "mingzhong" | "nei"))
                    {
                        victory.push(player.id);
                    } else {
                        fail.push(player.id);
                    }
                } else {
                    let same = player.identity == me
                        || (me == "zhu" && matches!(player.identity.as_str(), "zhong" | "mingzhong"));
                    if same && !(player.id != viewer && player.identity == "nei") {
                        fail.push(player.id);
                    } else if (survivor_identities.contains(&player.identity)
                        && !(group_has(&["fan"])
                            && matches!(player.identity.as_str(), "zhong" | "mingzhong" | "nei")))
                        || (group_has(&["zhu"])
                            && matches!(player.identity.as_str(), "zhong" | "mingzhong"))
                    {
                        victory.push(player.id);
                    } else {
                        fail.push(player.id);
                    }
                }
            }
            return Ok((victory, fail, tie));
        }

        // 平局 / 其他结果（上游 else 分支）。
        for id in &ordered {
            let player = self.player(*id)?;
            if survivor_identities.contains(&player.identity)
                || (group_has(&["zhu"])
                    && matches!(player.identity.as_str(), "zhong" | "mingzhong"))
            {
                tie.push(player.id);
            } else {
                fail.push(player.id);
            }
        }
        Ok((victory, fail, tie))
    }

    /// 上游 `jiesuanfix` 失败分支：非同一阵营者的胜负判定。
    fn classify_loser(
        me: &str,
        viewer: u32,
        identity: &str,
        id: u32,
        group_has: &dyn Fn(&[&str]) -> bool,
        nei_count: i64,
    ) -> bool {
        let me_ide = Self::ide(me, viewer, false);
        let item_ide = Self::ide(identity, id, false);
        let item_only = Self::ide(identity, id, true);
        match me_ide.as_str() {
            "zhu" => {
                if group_has(&["fan"]) || group_has(&["zhong"]) || nei_count > 1 {
                    item_ide == "fan"
                } else if nei_count == 1 {
                    item_only == "nei"
                } else {
                    false
                }
            }
            "fan" => {
                if group_has(&["zhu"]) {
                    item_ide == "zhu"
                } else {
                    item_only == "nei"
                }
            }
            _ => {
                if group_has(&["zhu"]) {
                    item_ide == "zhu"
                } else if group_has(&["fan"]) || group_has(&["zhong"]) || nei_count > 1 {
                    item_ide == "fan"
                } else if nei_count == 1 {
                    item_only == "nei"
                } else {
                    false
                }
            }
        }
    }

    /// 状态导出（保存恢复面）：配置不入状态（由调用方持有），只导出
    /// 已入账序号/计数与两项统计。
    pub fn export_state(&self) -> String {
        let scores: Vec<String> = self
            .scores
            .iter()
            .map(|(id, s)| {
                format!(
                    "{{\"id\":{id},\"attack\":{},\"heal\":{},\"support\":{},\"penalty\":{},\"base\":{}}}",
                    s.attack, s.heal, s.support, s.penalty, s.base
                )
            })
            .collect();
        let stats: Vec<String> = self
            .stats
            .iter()
            .map(|(id, s)| {
                format!(
                    "{{\"id\":{id},\"damage\":{},\"damaged\":{},\"gain\":{},\"play\":{},\"kill\":{}}}",
                    s.damage, s.damaged, s.gain, s.play, s.kill
                )
            })
            .collect();
        format!(
            "{{\"lastSeq\":{},\"applied\":{},\"duplicates\":{},\"gated\":{},\"scores\":[{}],\"stats\":[{}]}}",
            self.last_seq,
            self.applied,
            self.duplicates,
            self.gated,
            scores.join(","),
            stats.join(",")
        )
    }

    /// 状态恢复：事实序号高水位随状态延续，重复投递继续只入账一次。
    pub fn restore_state(config: MvpConfig, players: Vec<MvpPlayer>, state: &str) -> Result<Self, MvpError> {
        let value: Value =
            serde_json::from_str(state).map_err(|e| MvpError::BadState(e.to_string()))?;
        let mut session = Self::new(config, players);
        session.last_seq = value
            .get("lastSeq")
            .and_then(Value::as_u64)
            .ok_or(MvpError::BadState("lastSeq".into()))?;
        session.applied = value.get("applied").and_then(Value::as_u64).unwrap_or(0);
        session.duplicates = value.get("duplicates").and_then(Value::as_u64).unwrap_or(0);
        session.gated = value.get("gated").and_then(Value::as_u64).unwrap_or(0);
        if let Some(list) = value.get("scores").and_then(Value::as_array) {
            for entry in list {
                let id = entry
                    .get("id")
                    .and_then(Value::as_u64)
                    .ok_or(MvpError::BadState("scores.id".into()))? as u32;
                let Some(score) = session.scores.get_mut(&id) else {
                    return Err(MvpError::UnknownPlayer(id));
                };
                score.attack = entry.get("attack").and_then(Value::as_i64).unwrap_or(0);
                score.heal = entry.get("heal").and_then(Value::as_i64).unwrap_or(0);
                score.support = entry.get("support").and_then(Value::as_i64).unwrap_or(0);
                score.penalty = entry.get("penalty").and_then(Value::as_i64).unwrap_or(0);
                score.base = entry.get("base").and_then(Value::as_i64).unwrap_or(100);
            }
        }
        if let Some(list) = value.get("stats").and_then(Value::as_array) {
            for entry in list {
                let id = entry
                    .get("id")
                    .and_then(Value::as_u64)
                    .ok_or(MvpError::BadState("stats.id".into()))? as u32;
                let Some(stats) = session.stats.get_mut(&id) else {
                    return Err(MvpError::UnknownPlayer(id));
                };
                stats.damage = entry.get("damage").and_then(Value::as_i64).unwrap_or(0);
                stats.damaged = entry.get("damaged").and_then(Value::as_i64).unwrap_or(0);
                stats.gain = entry.get("gain").and_then(Value::as_i64).unwrap_or(0);
                stats.play = entry.get("play").and_then(Value::as_i64).unwrap_or(0);
                stats.kill = entry.get("kill").and_then(Value::as_i64).unwrap_or(0);
            }
        }
        Ok(session)
    }

    /// 紧凑文本报告（wasm / Node 目标平台面）。
    pub fn report(&self, settlement: &MvpSettlement) -> String {
        let scores = self
            .scores
            .iter()
            .map(|(id, s)| {
                format!(
                    "{id}:{}/{}/{}/{}/{}/{}",
                    s.attack,
                    s.heal,
                    s.support,
                    s.penalty,
                    s.base,
                    s.total()
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        let stats = self
            .stats
            .iter()
            .map(|(id, s)| {
                format!(
                    "{id}:{}/{}/{}/{}/{}",
                    s.damage, s.damaged, s.gain, s.play, s.kill
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        let join = |ids: &[u32]| {
            if ids.is_empty() {
                "-".to_string()
            } else {
                ids.iter()
                    .map(|id| id.to_string())
                    .collect::<Vec<_>>()
                    .join(",")
            }
        };
        format!(
            "enable:{} fix:{} | facts:{} dup:{} gated:{} | scores:{} | stats:{} | rank:{} | mvp:{} | win:{} fail:{} tie:{}",
            u8::from(self.config.enabled),
            u8::from(self.config.jiesuan_fix),
            self.applied,
            self.duplicates,
            self.gated,
            scores,
            stats,
            join(&settlement.ranking),
            settlement
                .mvp
                .map(|id| id.to_string())
                .unwrap_or_else(|| "-".to_string()),
            join(&settlement.victory),
            join(&settlement.fail),
            join(&settlement.tie),
        )
    }
}

/// 解析包资产 `config.json`（严格：缺字段/类型错一律报错）。
pub fn parse_mvp_config(text: &str) -> Result<MvpConfig, MvpError> {
    let value: Value = serde_json::from_str(text).map_err(|e| MvpError::Json(e.to_string()))?;
    let settings = value
        .get("settings")
        .ok_or(MvpError::Missing("settings"))?;
    let labels_value = value.get("labels").ok_or(MvpError::Missing("labels"))?;
    let labels_obj = labels_value
        .as_object()
        .ok_or(MvpError::BadField("labels"))?;
    let mut labels = BTreeMap::new();
    for (key, entry) in labels_obj {
        let text = entry.as_str().ok_or(MvpError::BadField("labels"))?;
        labels.insert(key.clone(), text.to_string());
    }
    let string_field = |name: &'static str| -> Result<String, MvpError> {
        let text = value
            .get(name)
            .and_then(Value::as_str)
            .ok_or(MvpError::Missing(name))?;
        if text.is_empty() {
            return Err(MvpError::BadField(name));
        }
        Ok(text.to_string())
    };
    let bool_field = |name: &'static str| -> Result<bool, MvpError> {
        settings
            .get(name)
            .and_then(Value::as_bool)
            .ok_or(MvpError::BadField(name))
    };
    Ok(MvpConfig {
        source: string_field("source")?,
        upstream_path: string_field("upstream_path")?,
        upstream_commit: string_field("upstream_commit")?,
        package_name: string_field("package_name")?,
        package_version: string_field("package_version")?,
        author: string_field("author")?,
        enabled: bool_field("shoushajiesuan")?,
        jiesuan_fix: bool_field("jiesuanfix")?,
        yangshi: settings
            .get("yangshi")
            .and_then(Value::as_str)
            .ok_or(MvpError::BadField("yangshi"))?
            .to_string(),
        one_more: bool_field("oneMore")?,
        style_audio: bool_field("qingyao_shoushapeiyin")?,
        labels,
    })
}

pub fn parse_mvp_request(text: &str) -> Result<(u32, String, String, Vec<MvpPlayer>, Vec<MvpFact>, Option<String>), MvpError> {
    let value: Value = serde_json::from_str(text).map_err(|e| MvpError::Json(e.to_string()))?;
    let viewer = value
        .get("viewer")
        .and_then(Value::as_u64)
        .ok_or(MvpError::Missing("viewer"))? as u32;
    let result = value
        .get("result")
        .and_then(Value::as_str)
        .ok_or(MvpError::Missing("result"))?
        .to_string();
    let mode = value
        .get("mode")
        .and_then(Value::as_str)
        .unwrap_or("identity")
        .to_string();
    let players_value = value
        .get("players")
        .and_then(Value::as_array)
        .ok_or(MvpError::Missing("players"))?;
    let mut players = Vec::new();
    for entry in players_value {
        let id = entry
            .get("id")
            .and_then(Value::as_u64)
            .ok_or(MvpError::Missing("players.id"))? as u32;
        let set_of = |name: &'static str| -> Result<BTreeSet<u32>, MvpError> {
            let mut out = BTreeSet::new();
            if let Some(list) = entry.get(name).and_then(Value::as_array) {
                for item in list {
                    out.insert(
                        item.as_u64()
                            .ok_or(MvpError::BadField("players.relations"))?
                            as u32,
                    );
                }
            }
            Ok(out)
        };
        players.push(MvpPlayer {
            id,
            name: entry
                .get("name")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string(),
            identity: entry
                .get("identity")
                .and_then(Value::as_str)
                .ok_or(MvpError::Missing("players.identity"))?
                .to_string(),
            alive: entry
                .get("alive")
                .and_then(Value::as_bool)
                .ok_or(MvpError::Missing("players.alive"))?,
            enemies: set_of("enemies")?,
            friends: set_of("friends")?,
        });
    }
    let mut facts = Vec::new();
    if let Some(list) = value.get("facts").and_then(Value::as_array) {
        for entry in list {
            facts.push(parse_fact(entry)?);
        }
    }
    let state = value
        .get("state")
        .and_then(Value::as_str)
        .map(str::to_string);
    Ok((viewer, result, mode, players, facts, state))
}

fn fact_seq(entry: &Value) -> Result<u64, MvpError> {
    entry
        .get("seq")
        .and_then(Value::as_u64)
        .ok_or(MvpError::Missing("facts.seq"))
}

fn fact_bool(entry: &Value, name: &'static str) -> bool {
    entry.get(name).and_then(Value::as_bool).unwrap_or(false)
}

/// 缺省为真的布尔字段（上游 `isIn()` 类存活判据：未显式给出即视为存活）。
fn fact_bool_default(entry: &Value, name: &'static str) -> bool {
    entry.get(name).and_then(Value::as_bool).unwrap_or(true)
}

fn fact_u32(entry: &Value, name: &'static str) -> Result<u32, MvpError> {
    entry
        .get(name)
        .and_then(Value::as_u64)
        .map(|v| v as u32)
        .ok_or(MvpError::Missing("facts.player"))
}

fn parse_fact(entry: &Value) -> Result<MvpFact, MvpError> {
    let seq = fact_seq(entry)?;
    let kind = entry
        .get("kind")
        .and_then(Value::as_str)
        .ok_or(MvpError::Missing("facts.kind"))?;
    Ok(match kind {
        "useCard" => MvpFact::UseCard {
            seq,
            player: fact_u32(entry, "player")?,
            damage_tag: fact_bool(entry, "damageTag"),
            wuxie: fact_bool(entry, "wuxie"),
            trick: fact_bool(entry, "trick"),
            to_self: fact_bool(entry, "toSelf"),
            select_target: entry.get("selectTarget").and_then(Value::as_i64).unwrap_or(1),
        },
        "damage" => MvpFact::Damage {
            seq,
            source: fact_u32(entry, "source")?,
            target: fact_u32(entry, "target")?,
            num: entry.get("num").and_then(Value::as_u64).unwrap_or(0) as u32,
            attitude_negative: fact_bool(entry, "attitudeNegative"),
            source_is_nei: fact_bool(entry, "sourceIsNei"),
        },
        "gain" => MvpFact::Gain {
            seq,
            player: fact_u32(entry, "player")?,
            source: fact_u32(entry, "source")?,
            cards: entry.get("cards").and_then(Value::as_u64).unwrap_or(0) as u32,
            source_alive: fact_bool_default(entry, "sourceAlive"),
            source_is_nei: fact_bool(entry, "sourceIsNei"),
            player_enemy: fact_bool(entry, "playerEnemy"),
        },
        "discard" => MvpFact::Discard {
            seq,
            player: fact_u32(entry, "player")?,
            source: fact_u32(entry, "source")?,
            cards: entry.get("cards").and_then(Value::as_u64).unwrap_or(0) as u32,
            source_alive: fact_bool_default(entry, "sourceAlive"),
            source_is_nei: fact_bool(entry, "sourceIsNei"),
            player_enemy: fact_bool(entry, "playerEnemy"),
        },
        "recover" => MvpFact::Recover {
            seq,
            player: fact_u32(entry, "player")?,
            source: fact_u32(entry, "source")?,
            num: entry.get("num").and_then(Value::as_u64).unwrap_or(0) as u32,
            source_alive: fact_bool_default(entry, "sourceAlive"),
            source_is_nei: fact_bool(entry, "sourceIsNei"),
            friend_or_self: fact_bool(entry, "friendOrSelf"),
        },
        "die" => MvpFact::Die {
            seq,
            victim: fact_u32(entry, "victim")?,
            source: fact_u32(entry, "source")?,
            source_alive: fact_bool_default(entry, "sourceAlive"),
            source_is_nei: fact_bool(entry, "sourceIsNei"),
            victim_is_lord: fact_bool(entry, "victimIsLord"),
            victim_friend: fact_bool(entry, "victimFriend"),
            victim_enemy: fact_bool(entry, "victimEnemy"),
        },
        other => return Err(MvpError::UnknownFact(other.to_string())),
    })
}

/// wasm / Node 单行入口：config + request（玩家表 + 事实 + 结算参数 +
/// 可选状态恢复）→ 紧凑报告。
pub fn apply_mvp_line(config_json: &str, request_json: &str) -> Result<String, MvpError> {
    let config = parse_mvp_config(config_json)?;
    let (viewer, result, mode, players, facts, state) = parse_mvp_request(request_json)?;
    let mut session = match state {
        Some(state) => MvpSession::restore_state(config, players, &state)?,
        None => MvpSession::new(config, players),
    };
    for fact in facts {
        session.apply(fact)?;
    }
    let settlement = session.settle(&result, viewer, &mode)?;
    Ok(session.report(&settlement))
}
