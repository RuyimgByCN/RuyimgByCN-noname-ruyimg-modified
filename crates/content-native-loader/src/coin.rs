//! U04 金币：余额、商店、下注和终局结算。
//!
//! 上游在非象棋主公、非录像、非联机时创建金币按钮，并用 `game.changeCoin`
//! 改 `lib.config.coin`。终局公式在核心 `game.over`：胜利先加 20，下注再加 10
//! 并另记 20；模式加成乘以 `0.8 + random * 0.4` 后向上取整。
//!
//! 这里保留数值。不创建画布，不播放音效，不弹提示，也不做烟花、下雪、星云和闪烁。

use std::collections::BTreeSet;

use serde_json::{json, Value};

/// 商店条目。价格和名称与上游 `game.haveFun.list` 一致。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ShopItem {
    /// 烟花，50 金。
    Firework,
    /// 下雪，20 金。
    Snow,
    /// 星云，10 金。
    Star,
    /// 闪烁，10 金。
    Blink,
}

/// 金币数字的显示方式。只记录取值，不生成 HTML。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoinDisplay {
    /// `coin_display_playpackconfig === "text"`。
    Text,
    /// 其余取值，上游画符号。
    Icon,
}

/// 雪的大小。`coinSnowSize === true` 表示小雪。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SnowSize {
    /// 默认大雪。
    Large,
    /// 小雪。
    Small,
}

/// 终局使用的模式。只取上游 `switch (lib.config.mode)` 的分支。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoinMode {
    /// 身份。
    Identity,
    /// 国战。
    Guozhan,
    /// 对决。
    Versus,
    /// 象棋。主公局由 [`CoinGate::chess_leader`] 另判。
    Chess,
    /// 其余模式，加成为 10。
    Other,
}

/// 身份。未列出的身份加成为 0。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IdentityRole {
    /// 主公。
    Zhu,
    /// 忠臣。
    Zhong,
    /// 明忠。
    Mingzhong,
    /// 反贼。
    Fan,
    /// 内奸。
    Nei,
    /// 其他身份。
    Other,
}

/// 是否创建金币按钮，以及终局是否累加 `_status.coin`。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CoinGate {
    /// `lib.config.mode`。
    pub mode: CoinMode,
    /// 象棋且 `chess_mode === "leader"`。
    pub chess_leader: bool,
    /// `_status.video`。
    pub video: bool,
    /// `_status.connectMode`。
    pub connect: bool,
    /// `game.online`。只挡住新的下注。
    pub online: bool,
    /// `game.phaseNumber` 已非 0。只挡住新的下注。
    pub phase_started: bool,
}

/// 包内配置。对应上游写入 `lib.config` 的那几项。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoinConfig {
    /// `lib.config.coin`。
    pub balance: i64,
    /// 显示方式。
    pub display: CoinDisplay,
    /// `coin_canvas_playpackconfig`。为真也不创建画布。
    pub canvas: bool,
    /// `snowFall`。为真时开局视为已购买下雪并开始。
    pub snow_fall: bool,
    /// `coinSnowSize`。为真时雪是小雪。
    pub snow_size_small: bool,
    /// 来源仓。已发布的 noname 配置没有这个键，解析为 `noname`。
    pub source: String,
}

/// 一局内的金币状态。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoinSession {
    balance: i64,
    display: CoinDisplay,
    canvas: bool,
    snow_fall: bool,
    snow_size: SnowSize,
    owned: BTreeSet<ShopItem>,
    running: BTreeSet<ShopItem>,
    bet: bool,
    online: bool,
    phase_started: bool,
    /// 有金币按钮时，`changeCoin` 才改余额。
    persist: bool,
    /// `typeof _status.coin == "number" && !connect`。
    match_coin: Option<i64>,
    source: String,
}

/// 终局参数。随机数由调用方给入，范围是 0 到 1。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SettleRequest {
    /// 结果是「战斗胜利」。
    pub victory: bool,
    /// 身份局的身份。其他模式忽略。
    pub role: IdentityRole,
    /// `enhance_zhu`。
    pub enhance_zhu: bool,
    /// 存活人数。
    pub alive: u32,
    /// 阵亡人数。
    pub dead: u32,
    /// 国战野心家。
    pub ye: bool,
    /// 国战该势力人数。非野心家时必须大于 0。
    pub population: u32,
    /// 对决友方人数。`None` 表示没有 `_status.friend`。
    pub friends: Option<u32>,
    /// 象棋战斗且有额外玩家。
    pub chess_combat_extra: bool,
    /// 对应 `Math.random()`。
    pub unit_random: f64,
    /// `_status.coinCoeff`。
    pub coin_coeff: Option<f64>,
    /// `_status.additionalReward()`。
    pub additional_reward: i64,
}

/// 终局写回的数额。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SettleOutcome {
    /// `changeCoin(_status.coin)` 的数额。
    pub payout: i64,
    /// 下注获胜时额外的 `changeCoin(20)`。
    pub bet_return: i64,
    /// 是否写入余额。录像局只计算、不写入。
    pub persisted: bool,
}

/// 金币操作的失败。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CoinError {
    /// 配置无法使用。
    BadConfig(String),
    /// 没有金币按钮，或终局不累加。
    Inactive,
    /// 商店里没有这个名字。
    UnknownItem(String),
    /// 余额不够。
    NotEnough {
        /// 现有余额。
        balance: i64,
        /// 售价。
        cost: i64,
    },
    /// 已经买过。
    AlreadyOwned(ShopItem),
    /// 还没买。
    NotOwned(ShopItem),
    /// 效果已经在跑。
    AlreadyRunning(ShopItem),
    /// 效果没在跑。
    NotRunning(ShopItem),
    /// 不能再下注。
    BetClosed,
    /// 已经下注。
    AlreadyBet,
    /// 随机数或系数不合法。
    BadRandom(String),
    /// 国战势力人数为 0。
    EmptyPopulation,
    /// 快照里有不认识的字段。
    BadSnapshot(String),
}

impl ShopItem {
    /// 售价。
    pub fn cost(self) -> i64 {
        match self {
            ShopItem::Firework => 50,
            ShopItem::Snow => 20,
            ShopItem::Star | ShopItem::Blink => 10,
        }
    }

    /// 显示名。
    pub fn title(self) -> &'static str {
        match self {
            ShopItem::Firework => "烟花",
            ShopItem::Snow => "下雪",
            ShopItem::Star => "星云",
            ShopItem::Blink => "闪烁",
        }
    }

    fn key(self) -> &'static str {
        match self {
            ShopItem::Firework => "firework",
            ShopItem::Snow => "snow",
            ShopItem::Star => "star",
            ShopItem::Blink => "blink",
        }
    }

    fn parse(name: &str) -> Result<Self, CoinError> {
        match name {
            "firework" => Ok(ShopItem::Firework),
            "snow" => Ok(ShopItem::Snow),
            "star" => Ok(ShopItem::Star),
            "blink" => Ok(ShopItem::Blink),
            other => Err(CoinError::UnknownItem(other.to_string())),
        }
    }
}

impl CoinSession {
    /// 余额。
    pub fn balance(&self) -> i64 {
        self.balance
    }

    /// 是否已下注。
    pub fn bet_placed(&self) -> bool {
        self.bet
    }

    /// 已购买的条目。
    pub fn owned(&self) -> impl Iterator<Item = ShopItem> + '_ {
        self.owned.iter().copied()
    }

    /// 正在播放的条目。播放只是标记，没有画布。
    pub fn running(&self) -> impl Iterator<Item = ShopItem> + '_ {
        self.running.iter().copied()
    }

    /// 雪的大小。
    pub fn snow_size(&self) -> SnowSize {
        self.snow_size
    }

    /// 持久化的常下雪开关。
    pub fn snow_fall(&self) -> bool {
        self.snow_fall
    }

    /// 显示方式。
    pub fn display(&self) -> CoinDisplay {
        self.display
    }

    /// 画布开关。为真也不创建画布。
    pub fn canvas(&self) -> bool {
        self.canvas
    }

    /// 来源仓。
    pub fn source(&self) -> &str {
        &self.source
    }
}

fn source_token(value: Option<&Value>, missing: &str) -> Result<String, ()> {
    match value {
        None => Ok(missing.to_string()),
        Some(Value::String(text)) if is_source_token(text) => Ok(text.clone()),
        Some(_) => Err(()),
    }
}

fn is_source_token(text: &str) -> bool {
    !text.is_empty()
        && text
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_')
}

/// 解析包内 `config.json`。
pub fn parse_coin_config(text: &str) -> Result<CoinConfig, CoinError> {
    let value: Value =
        serde_json::from_str(text).map_err(|err| CoinError::BadConfig(err.to_string()))?;
    let obj = value
        .as_object()
        .ok_or_else(|| CoinError::BadConfig("config must be an object".into()))?;
    let balance = obj
        .get("coin")
        .and_then(Value::as_i64)
        .ok_or_else(|| CoinError::BadConfig("coin must be an integer".into()))?;
    let display = match obj
        .get("coin_display_playpackconfig")
        .and_then(Value::as_str)
    {
        Some("text") => CoinDisplay::Text,
        Some("icon") => CoinDisplay::Icon,
        Some(other) => return Err(CoinError::BadConfig(format!("unknown display {other}"))),
        None => return Err(CoinError::BadConfig("missing display".into())),
    };
    let canvas = obj
        .get("coin_canvas_playpackconfig")
        .and_then(Value::as_bool)
        .ok_or_else(|| CoinError::BadConfig("canvas must be a bool".into()))?;
    let snow_fall = obj
        .get("snowFall")
        .and_then(Value::as_bool)
        .ok_or_else(|| CoinError::BadConfig("snowFall must be a bool".into()))?;
    let snow_size_small = obj
        .get("coinSnowSize")
        .and_then(Value::as_bool)
        .ok_or_else(|| CoinError::BadConfig("coinSnowSize must be a bool".into()))?;
    let source = source_token(obj.get("source"), "noname")
        .map_err(|_| CoinError::BadConfig("source must be a token".into()))?;
    Ok(CoinConfig {
        balance,
        display,
        canvas,
        snow_fall,
        snow_size_small,
        source,
    })
}

/// 按门槛打开一局。象棋主公和联机没有金币；录像有局内累计但按钮不存在。
pub fn open_coin_session(config: &CoinConfig, gate: CoinGate) -> CoinSession {
    let chess_leader = gate.mode == CoinMode::Chess && gate.chess_leader;
    let persist = !gate.video && !gate.connect && !chess_leader;
    let match_coin = if gate.connect || chess_leader {
        None
    } else {
        Some(0)
    };
    let mut owned = BTreeSet::new();
    let mut running = BTreeSet::new();
    let snow_size = if config.snow_size_small {
        SnowSize::Small
    } else {
        SnowSize::Large
    };
    if persist && config.snow_fall {
        owned.insert(ShopItem::Snow);
        running.insert(ShopItem::Snow);
    }
    CoinSession {
        balance: config.balance,
        display: config.display,
        canvas: config.canvas,
        snow_fall: config.snow_fall,
        snow_size,
        owned,
        running,
        bet: false,
        online: gate.online,
        phase_started: gate.phase_started,
        persist,
        match_coin,
        source: config.source.clone(),
    }
}

/// 进入阶段后不能新下注。
pub fn mark_phase_started(session: &mut CoinSession) {
    session.phase_started = true;
}

/// 购买并立刻开始。余额不足或已经拥有时不改状态。
pub fn buy_item(session: &mut CoinSession, item: ShopItem) -> Result<(), CoinError> {
    if !session.persist {
        return Err(CoinError::Inactive);
    }
    if session.owned.contains(&item) {
        return Err(CoinError::AlreadyOwned(item));
    }
    let cost = item.cost();
    if session.balance < cost {
        return Err(CoinError::NotEnough {
            balance: session.balance,
            cost,
        });
    }
    session.balance -= cost;
    session.owned.insert(item);
    session.running.insert(item);
    Ok(())
}

/// 开始一个已购买、当前没在跑的效果。
pub fn start_item(session: &mut CoinSession, item: ShopItem) -> Result<(), CoinError> {
    if !session.persist {
        return Err(CoinError::Inactive);
    }
    if !session.owned.contains(&item) {
        return Err(CoinError::NotOwned(item));
    }
    if !session.running.insert(item) {
        return Err(CoinError::AlreadyRunning(item));
    }
    Ok(())
}

/// 停止一个正在跑的效果。
pub fn stop_item(session: &mut CoinSession, item: ShopItem) -> Result<(), CoinError> {
    if !session.persist {
        return Err(CoinError::Inactive);
    }
    if !session.owned.contains(&item) {
        return Err(CoinError::NotOwned(item));
    }
    if !session.running.remove(&item) {
        return Err(CoinError::NotRunning(item));
    }
    Ok(())
}

/// 切换雪的大小。大雪变小雪时 `coinSnowSize` 为真。
pub fn toggle_snow_size(session: &mut CoinSession) -> Result<bool, CoinError> {
    if !session.persist {
        return Err(CoinError::Inactive);
    }
    if !session.owned.contains(&ShopItem::Snow) {
        return Err(CoinError::NotOwned(ShopItem::Snow));
    }
    session.snow_size = match session.snow_size {
        SnowSize::Large => SnowSize::Small,
        SnowSize::Small => SnowSize::Large,
    };
    Ok(session.snow_size == SnowSize::Small)
}

/// 切换常下雪。相当于上游保存后重载：局内临时购买不保留。
pub fn toggle_snow_fall(session: &CoinSession) -> CoinConfig {
    persist_config(session, !session.snow_fall)
}

/// 直接改余额。没有金币按钮时不改，对应 `ui.coin` 不存在。
pub fn change_coin(session: &mut CoinSession, amount: i64) -> Result<(), CoinError> {
    if !session.persist {
        return Err(CoinError::Inactive);
    }
    session.balance += amount;
    Ok(())
}

/// 开局下注 20。不检查余额，因此可以变成负数。重复下注不改余额。
pub fn place_bet(session: &mut CoinSession) -> Result<(), CoinError> {
    if !session.persist {
        return Err(CoinError::Inactive);
    }
    if session.online || session.phase_started {
        return Err(CoinError::BetClosed);
    }
    if session.bet {
        return Err(CoinError::AlreadyBet);
    }
    session.bet = true;
    session.balance -= 20;
    Ok(())
}

/// 按核心终局公式结算。`mode` 选择加成分支。`unit_random` 代替 `Math.random`。
pub fn settle_for(
    session: &mut CoinSession,
    mode: CoinMode,
    request: SettleRequest,
) -> Result<SettleOutcome, CoinError> {
    if session.match_coin.is_none() {
        return Err(CoinError::Inactive);
    }
    if !(0.0..=1.0).contains(&request.unit_random) {
        return Err(CoinError::BadRandom("unit random out of 0..=1".into()));
    }
    if let Some(coeff) = request.coin_coeff {
        if !coeff.is_finite() {
            return Err(CoinError::BadRandom("coin coeff not finite".into()));
        }
    }
    let coeff = request.unit_random * 0.4 + 0.8;
    let seats = f64::from(request.alive.saturating_add(request.dead));
    let mut added = if request.victory {
        match mode {
            CoinMode::Identity => {
                let base = match request.role {
                    IdentityRole::Zhu | IdentityRole::Zhong | IdentityRole::Mingzhong => {
                        if request.enhance_zhu {
                            10.0
                        } else {
                            20.0
                        }
                    }
                    IdentityRole::Fan => {
                        if request.enhance_zhu {
                            16.0
                        } else {
                            8.0
                        }
                    }
                    IdentityRole::Nei => 40.0,
                    IdentityRole::Other => 0.0,
                };
                base * seats / 8.0
            }
            CoinMode::Guozhan => {
                let base = if request.ye {
                    8.0
                } else if request.population == 0 {
                    return Err(CoinError::EmptyPopulation);
                } else {
                    5.0 / f64::from(request.population)
                };
                base * seats
            }
            CoinMode::Versus => match request.friends {
                Some(friends) => 5.0 * (f64::from(request.alive) + f64::from(friends)),
                None => 0.0,
            },
            CoinMode::Chess | CoinMode::Other => 10.0,
        }
    } else {
        10.0
    };
    if mode == CoinMode::Chess && request.chess_combat_extra {
        added = 2.0;
    }
    let mut pool = 0.0;
    let mut bet_return = 0i64;
    if request.victory {
        if session.bet {
            pool += 10.0;
            bet_return = 20;
        }
        pool += 20.0;
        pool += request.additional_reward as f64;
    }
    pool += added * coeff;
    if let Some(extra) = request.coin_coeff {
        pool *= extra;
    }
    if !pool.is_finite() {
        return Err(CoinError::BadRandom("payout not finite".into()));
    }
    let payout = pool.ceil() as i64;
    let persisted = session.persist;
    if persisted {
        session.balance += payout + bet_return;
    }
    Ok(SettleOutcome {
        payout,
        bet_return,
        persisted,
    })
}

/// 写回可跨重启保留的配置。局内购买除 `snowFall` 外不保留。
pub fn persist_config(session: &CoinSession, snow_fall: bool) -> CoinConfig {
    CoinConfig {
        balance: session.balance,
        display: session.display,
        canvas: session.canvas,
        snow_fall,
        snow_size_small: session.snow_size == SnowSize::Small,
        source: session.source.clone(),
    }
}

/// 局内快照。恢复时不认识的商品名会拒绝。
pub fn snapshot_coin(session: &CoinSession) -> String {
    let owned: Vec<&str> = session.owned.iter().map(|item| item.key()).collect();
    let running: Vec<&str> = session.running.iter().map(|item| item.key()).collect();
    json!({
        "balance": session.balance,
        "display": match session.display { CoinDisplay::Text => "text", CoinDisplay::Icon => "icon" },
        "canvas": session.canvas,
        "snow_fall": session.snow_fall,
        "snow_size": match session.snow_size { SnowSize::Large => "large", SnowSize::Small => "small" },
        "owned": owned,
        "running": running,
        "bet": session.bet,
        "online": session.online,
        "phase_started": session.phase_started,
        "persist": session.persist,
        "match_coin": session.match_coin,
        "source": session.source,
    })
    .to_string()
}

/// 从 [`snapshot_coin`] 恢复。
pub fn restore_coin(text: &str) -> Result<CoinSession, CoinError> {
    let value: Value =
        serde_json::from_str(text).map_err(|err| CoinError::BadSnapshot(err.to_string()))?;
    let obj = value
        .as_object()
        .ok_or_else(|| CoinError::BadSnapshot("snapshot must be an object".into()))?;
    let balance = obj
        .get("balance")
        .and_then(Value::as_i64)
        .ok_or_else(|| CoinError::BadSnapshot("balance".into()))?;
    let display = match obj.get("display").and_then(Value::as_str) {
        Some("text") => CoinDisplay::Text,
        Some("icon") => CoinDisplay::Icon,
        _ => return Err(CoinError::BadSnapshot("display".into())),
    };
    let canvas = obj
        .get("canvas")
        .and_then(Value::as_bool)
        .ok_or_else(|| CoinError::BadSnapshot("canvas".into()))?;
    let snow_fall = obj
        .get("snow_fall")
        .and_then(Value::as_bool)
        .ok_or_else(|| CoinError::BadSnapshot("snow_fall".into()))?;
    let snow_size = match obj.get("snow_size").and_then(Value::as_str) {
        Some("large") => SnowSize::Large,
        Some("small") => SnowSize::Small,
        _ => return Err(CoinError::BadSnapshot("snow_size".into())),
    };
    let owned = parse_items(obj.get("owned"))?;
    let running = parse_items(obj.get("running"))?;
    if !running.is_subset(&owned) {
        return Err(CoinError::BadSnapshot("running item is not owned".into()));
    }
    let bet = obj
        .get("bet")
        .and_then(Value::as_bool)
        .ok_or_else(|| CoinError::BadSnapshot("bet".into()))?;
    let online = obj
        .get("online")
        .and_then(Value::as_bool)
        .ok_or_else(|| CoinError::BadSnapshot("online".into()))?;
    let phase_started = obj
        .get("phase_started")
        .and_then(Value::as_bool)
        .ok_or_else(|| CoinError::BadSnapshot("phase".into()))?;
    let persist = obj
        .get("persist")
        .and_then(Value::as_bool)
        .ok_or_else(|| CoinError::BadSnapshot("persist".into()))?;
    let match_coin = match obj.get("match_coin") {
        Some(Value::Null) => None,
        Some(value) => Some(
            value
                .as_i64()
                .ok_or_else(|| CoinError::BadSnapshot("match_coin".into()))?,
        ),
        None => return Err(CoinError::BadSnapshot("match_coin".into())),
    };
    let source = source_token(obj.get("source"), "noname")
        .map_err(|_| CoinError::BadSnapshot("source".into()))?;
    Ok(CoinSession {
        balance,
        display,
        canvas,
        snow_fall,
        snow_size,
        owned,
        running,
        bet,
        online,
        phase_started,
        persist,
        match_coin,
        source,
    })
}

fn parse_items(value: Option<&Value>) -> Result<BTreeSet<ShopItem>, CoinError> {
    let list = value
        .and_then(Value::as_array)
        .ok_or_else(|| CoinError::BadSnapshot("items".into()))?;
    let mut out = BTreeSet::new();
    for item in list {
        let name = item
            .as_str()
            .ok_or_else(|| CoinError::BadSnapshot("item".into()))?;
        out.insert(ShopItem::parse(name)?);
    }
    Ok(out)
}

/// 供目标平台字符串辅助调用的一条命令序列。
pub fn apply_coin_commands(
    config_text: &str,
    gate: CoinGate,
    mode: CoinMode,
    commands: &str,
) -> Result<String, CoinError> {
    let config = parse_coin_config(config_text)?;
    let mut session = open_coin_session(&config, gate);
    for command in commands.split(';').filter(|part| !part.is_empty()) {
        let mut parts = command.split(':');
        let op = parts.next().unwrap_or("");
        match op {
            "buy" => buy_item(&mut session, ShopItem::parse(parts.next().unwrap_or(""))?)?,
            "start" => start_item(&mut session, ShopItem::parse(parts.next().unwrap_or(""))?)?,
            "stop" => stop_item(&mut session, ShopItem::parse(parts.next().unwrap_or(""))?)?,
            "bet" => place_bet(&mut session)?,
            "gain" => {
                let amount = parts
                    .next()
                    .unwrap_or("")
                    .parse::<i64>()
                    .map_err(|_| CoinError::BadConfig("gain".into()))?;
                change_coin(&mut session, amount)?;
            }
            "settle" => {
                let victory = parts.next() == Some("win");
                let unit = parts
                    .next()
                    .unwrap_or("")
                    .parse::<f64>()
                    .map_err(|_| CoinError::BadRandom("unit".into()))?;
                let request = SettleRequest {
                    victory,
                    role: IdentityRole::Zhu,
                    enhance_zhu: false,
                    alive: 8,
                    dead: 0,
                    ye: false,
                    population: 1,
                    friends: None,
                    chess_combat_extra: mode == CoinMode::Chess,
                    unit_random: unit,
                    coin_coeff: None,
                    additional_reward: 0,
                };
                settle_for(&mut session, mode, request)?;
            }
            other => return Err(CoinError::BadConfig(format!("unknown command {other}"))),
        }
    }
    let owned: Vec<&str> = session.owned.iter().map(|item| item.key()).collect();
    let running: Vec<&str> = session.running.iter().map(|item| item.key()).collect();
    Ok(format!(
        "balance:{} owned:{} running:{} bet:{} fall:{} source:{}",
        session.balance,
        owned.join(","),
        running.join(","),
        u8::from(session.bet),
        u8::from(session.snow_fall),
        session.source
    ))
}
