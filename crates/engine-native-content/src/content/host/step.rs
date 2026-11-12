use super::*;

mod boss;
mod gujian;
mod gwent;
mod hearth;
mod jiguan;
mod mtg;
mod skills_card_use;
mod skills_cards;
mod skills_virtual_use;
mod skills_discard_costs;
mod skills_damage;
mod skills_recovery;
mod skills_status;
mod skills_targets;
mod swd_card_contests;
mod swd_card_effects;
mod swd_jiguan_cards;
mod swd_lingjian_cards;
mod swd_zhenfa_cards;
mod swd_card_targets;
mod swd_card_virtual_sha;
mod swd_card_attach;
mod swd_cards;
mod swd_virtual_cards;
mod swd_response_cards;
mod swd_damage_combos;
mod swd_damage_costs;
mod swd_damage_multi;
mod swd_damage_reactions;
mod swd_damage_targets;
mod swd_equipment;
mod swd_equipment_skills;
mod swd_recovery;
mod swd_status;
mod swd_targets;
mod wudian_cards;
mod wudian_ow_cards;
mod wudian_damage;
mod wudian_recovery;
mod wudian_status;
mod wudian_targets;
mod xianjian_cards;
mod xianjian_card_use;
mod xianjian_damage;
mod xianjian_recovery;
mod xianjian_status;
mod xianjian_targets;
mod yunchou;

impl HostMachine {
    pub(super) fn step_behavior(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        match self.behavior {
            // 平恤：无效果步（computed_distance 字段消费）
            HostBehavior::SwdPingxu => self.step_swd_status(ctx),
            // 每步·距离：无效果步（computed_distance 字段消费）
            HostBehavior::NewMeibuRange => self.step_skills_status(ctx),
            // 反击·红杀直伤：无效果步（杀机器内联消费）
            HostBehavior::YxsFanji2 => self.step_skills_status(ctx),
            // 烈弓：无效果步（杀机器内联消费）
            HostBehavior::FzLiegong => self.step_skills_status(ctx),
            // 门神·标记持有：无效果步（主体 70 KB 效果待批次）
            HostBehavior::YxsMenshen2 => self.step_skills_status(ctx),
            // 摸牌翻倍：无效果步（摸牌收束内联消费）
            HostBehavior::XiguaMopaiFanbei => self.step_skills_status(ctx),
            // 诈降：无效果步（出牌校验/杀机器/摸牌收束内联消费）
            HostBehavior::SbZhaXiang => self.step_skills_status(ctx),
            // 狂骨：选「回复 1」或「摸 1」
            HostBehavior::FzXinKuanggu => self.step_skills_recovery(ctx),
            // 苦肉：失去 1 点体力并摸 2 张
            HostBehavior::Kurou => self.step_skills_status(ctx),
            // 主亡忠继：忠臣继位为主公（身份互换）
            HostBehavior::ZhuWanzhongJi => self.step_skills_status(ctx),
            // 血量翻倍：体力上限 + 当前体力并回复等量（一次性）
            HostBehavior::XiguaXueliangFanbei => self.step_skills_recovery(ctx),
            // 门神：选一名其他角色 → 授予门神标记
            HostBehavior::YxsMenshen => self.step_skills_targets(ctx),
            // 门神·清除：移除全场门神实例与标记
            HostBehavior::YxsMenshen3 => self.step_skills_status(ctx),
            // 反击：授予临时红杀直伤 → 选一张【杀】对来源使用
            HostBehavior::YxsFanji => self.step_skills_damage(ctx),
            // 灾变：摸「忠 + 2 - 反 - 内」（下限 0）张
            HostBehavior::Zaibian => self.step_skills_status(ctx),
            // 誓师：体力上限 +1 并回复 1
            HostBehavior::Shishi => self.step_skills_recovery(ctx),
            // 循梦：伤害 +1；体力 >1 时失去 1 点体力
            HostBehavior::Xunmeng => self.step_skills_status(ctx),
            // 集舍：摸 1 + 累计标记（上游 jishe2 的手牌上限削减不生效）
            HostBehavior::Jishe => self.step_skills_status(ctx),
            // 疾影：无效果步（sha_range_free 链查询）
            HostBehavior::SwdJiyingHeld => self.step_swd_equipment(ctx),
            // 沐风：确认门后摸 1（discoverCard 确定性=牌堆末摸 1）
            HostBehavior::SwdMufengHeld => self.step_swd_equipment(ctx),
            // 望尘：确认门后 TargetChoice 选任意角色翻面
            HostBehavior::SwdWangchenHeld => self.step_swd_equipment(ctx),
            // 还侠：无效果步（PlayCard 转化臂+kernel 跟踪钩承载）
            HostBehavior::SwdHuanxia => self.step_swd_status(ctx),
            // 破月：无效果步
            HostBehavior::SwdPoyue => self.step_swd_status(ctx),
            // 收纳：SelectCard 弃 1 手牌 → TargetChoice 其他有手牌角色 →
            // Act 目标首张手牌移入壶仓（登记差异上游随机取）
            HostBehavior::SwdLianyaohuShouna => self.step_swd_equipment(ctx),
            // 炼化：SelectCard 壶仓选 2 → Act 弃 2+牌堆顶首张类别均不同入手
            HostBehavior::SwdLianyaohuLianhua => self.step_swd_equipment(ctx),
            // 壶境：弃牌堆回收炼妖壶入装备区（宝物栏）
            HostBehavior::SwdHujing => self.step_swd_equipment(ctx),
            // 神农鼎：SelectCard 弃 2 手牌 → Act 回复 1（受伤时；门控在
            // activate 臂 hp<maxHp+手牌≥2）
            HostBehavior::SwdShennongdingCard => self.step_swd_card_effects(ctx),
            // 鬼蛇旗：目标 +1 护甲（change_hujia=受伤抵 1 伤语义）
            HostBehavior::GuisheqiCard => self.step_skills_status(ctx),
            // 仙露回：全体受伤目标各 +1 护甲（targets 已过滤受伤）
            HostBehavior::XianluhuiCard => self.step_skills_status(ctx),
            // 空洞印：死亡取消——hp=1、摸 1、弃全部宝物（death_cancelled
            // 基建消费，DeathMachine 开头拦截跳过死亡执行）
            HostBehavior::SwdKongdongyinGuard => self.step_swd_equipment(ctx),
            // 逆攻积攒：标记 +amount（上限 4）
            HostBehavior::SwdNigongAccumulate => self.step_swd_equipment(ctx),
            // 四非珍面：TargetChoice 选其他有手牌角色 → Act 判定（非红桃
            // 则目标用首张手牌指向座位序首个其他存活者——登记差异上游
            // 随机牌/随机目标）
            HostBehavior::SwdSifeizhenmianCard => self.step_swd_card_effects(ctx),
            // 青龙之圭：摸 2 → 弃 1 张 he（SelectCard min1）
            HostBehavior::SwdQinglongzhiguiHeld => self.step_swd_equipment(ctx),
            // 朱雀之璋：来源火 1 伤（上游 fire；native 火属性同壳）
            HostBehavior::SwdZhuquezhizhangHeld => self.step_swd_equipment(ctx),
            // 白兽之琥：TargetChoice 选其他有牌角色 → Act 其首张 he 弃置
            //（确定性首张，登记差异上游随机）
            HostBehavior::SwdBaishouzhihuHeld => self.step_swd_equipment(ctx),
            // 玄武之璜：回复等量体力（伤害结算帧读 amount，hp<max 门在 filter）
            HostBehavior::SwdXuanwuzhihuangHeld => self.step_swd_equipment(ctx),
            // 黄麟之琮：+1 护甲（hujia==0 门在 filter）
            HostBehavior::SwdHuanglinzhicongHeld => self.step_swd_equipment(ctx),
            // 苍螭之璧（裁决 33）：选 1–3 名存活角色横置或重置
            HostBehavior::SwdCangchizhibiHeld => self.step_swd_equipment(ctx),
            // 神草结：确认门后弃一张神草结——受伤者是自己则伤-1、是自己
            // 造成则伤+1
            HostBehavior::SwdShencaojieGuard => self.step_swd_equipment(ctx),
            // 神秘果窗口（卡牌面窗口机批）：确认后——发射载体神秘果用牌
            //（提交段自取+Shenmiguo 臂弃置=上游 useCard 同簿记），随后
            // 原体自弃堆上桌面重发射（登记差异：上游凭空复制同花色点数
            // 新实体，原生原体复用——弃堆在再结算窗口内少一张观测差）。
            // 确认即置位每阶段一次旗（上游 shenmiguo2 临时技：再结算牌
            // 的用牌后窗口被同一旗挡住=循环守卫）。
            HostBehavior::SwdShenmiguoWindow => self.step_swd_equipment_skills(ctx),
            // 鼠槌（equip5 窗口机批）：凤略 Yield 循环同型——选攻击范围
            // 内目标（sha_range_free 链同 engine 出杀口径）→ 至多 3 张
            // 逐张自动用出（每次重扫手中首张杀=上游 getCards("h")[0]
            // 确定性口径）；终局若链上任一杀造成伤害（commit_damage 钩
            // 置位 shuchui_hit）则摸 1。cards 复用为剩余次数计数。
            HostBehavior::SwdShuchuiUse => self.step_swd_equipment_skills(ctx),
            // 浴尘：Act 选目标（he 非空其他角色）→ TargetChoice 随机弃其
            // 一张（登记差异：上游弃牌者自选，原生 RNG 代选——冰河要塞
            // 同款登记；装备随 revoke）
            HostBehavior::SwdYuchen => self.step_swd_targets(ctx),
            // 挟雷：Act 弃一张牌（he 可取消=放弃）→ TargetChoice 选合法
            // 目标（use 排除原目标；respond 路径登记差异：事件载荷无来源
            // 不排除）→ 1 点雷伤
            HostBehavior::SwdXielei => self.step_swd_damage_costs(ctx),
            // 湛卢：SelectCard 弃一张黑桃牌（he 可取消=放弃）→ TargetChoice
            // 选至多 3 名已受伤角色 → 各回复 1
            HostBehavior::SwdZhanlu => self.step_swd_recovery(ctx),
            // 苏生：死亡前置窗口——SelectCard 弃一张手牌（可取消=放弃，
            // 死亡照常）→ 目标体力回至 1 + death_cancelled 写回（空洞印
            // 取消基建；DeathMachine 清 dying 旗）
            HostBehavior::SwdSusheng => self.step_swd_response_cards(ctx),
            // 相惠：SelectCard 弃一张红色手牌 → 全体体力最少且已受伤角色
            // 各回复 1（目标面自动求取，无选择窗）
            HostBehavior::SwdXianghui => self.step_swd_recovery(ctx),
            // 挥琪：TargetChoice 选一名其他角色 → 摸 X（X=自己已损失体力）
            HostBehavior::SwdHuiqi => self.step_swd_targets(ctx),
            // 森罗：activate 门已验手牌最少——对其他有手牌角色各：弃 2
            //（确定性首两张，登记差异上游自选）+摸 1
            HostBehavior::SwdSenluo => self.step_swd_status(ctx),
            // 连打：SelectCard 弃一张牌（he 可取消=放弃）→ 虚拟杀对原目
            // 标再结算（凤略直发口径；每回合旗置位=循环守卫）
            HostBehavior::SwdLianda => self.step_swd_virtual_cards(ctx),
            // 隳延：SelectCard 弃一张牌 → 弃置对方全部装备（随 revoke）
            HostBehavior::SwdSuiyan => self.step_swd_damage_reactions(ctx),
            // 断行：TargetChoice 选杀合法目标（距离内）→ 虚拟杀
            HostBehavior::SwdDuanxing => self.step_swd_targets(ctx),
            // 破浪：SelectCards 选对方装备一张（可取消=放弃；公开信息自选
            // 面保留）→ 弃置
            HostBehavior::SwdPolang => self.step_swd_damage_reactions(ctx),
            // 诱引：frequent 锁定式——直摸 1（无确认窗）
            HostBehavior::SwdYouyin => self.step_swd_status(ctx),
            // 连舞：锁定 marker（engine max_targets 链+kernel 红杀 directHit
            // 臂查询），无独立效果步
            HostBehavior::SwdLianwu => self.step_swd_status(ctx),
            // 万钧：锁定 marker（engine PlayCard viewAs 链查询），无效果步
            HostBehavior::SwdWanjun => self.step_swd_status(ctx),
            // 火扑：锁定 marker（engine PlayCard viewAs 链查询），无效果步
            HostBehavior::SwdHuopu => self.step_swd_status(ctx),
            // 夜影：锁定 marker（engine PlayCard viewAs 链查询），无效果步
            HostBehavior::SwdYeying => self.step_swd_status(ctx),
            // 归影：锁定 marker（engine PlayCard viewAs 链查询），无效果步
            HostBehavior::SwdGuiying => self.step_swd_status(ctx),
            // 凤泽：锁定 marker（engine PlayCard viewAs 链查询），无效果步
            HostBehavior::SwdFengze => self.step_swd_status(ctx),
            // 玲月：锁定（sha 机器内联臂查询），无独立效果步
            HostBehavior::SwdLingyue => self.step_swd_status(ctx),
            // 暗莲映：frequent 锁定式——直摸 2
            HostBehavior::SwdAnlianying => self.step_swd_status(ctx),
            // 疾风：锁定 marker（engine max_targets 链+state computed_distance
            // 链查询），无独立效果步
            HostBehavior::SwdLjifeng => self.step_swd_status(ctx),
            // 断义：Act 弃两张杀（代价，selectCard 2 非默认弃置面——
            // 机器显式弃）→ SelectCard 消费 → TargetChoice 选目标 →
            // 1 伤 + 目标随机弃 X 张 he（X=其已损失体力）
            HostBehavior::SwdDuanyi => self.step_swd_damage_costs(ctx),
            // 敷衍·护甲面：TargetChoice 选无护甲角色 → +1 护甲
            HostBehavior::SwdFuyan => self.step_swd_targets(ctx),
            // 鸣凤：SelectCard 弃一张装备牌 → TargetChoice 任一存活角色
            // → 回复 1 + 摸 1
            HostBehavior::SwdFengming => self.step_swd_recovery(ctx),
            // 冰箭：SelectCard 弃黑色杀 → TargetChoice 有手牌目标 → 闪全
            // 弃或 1 雷伤
            HostBehavior::SwdBingjian => self.step_swd_damage_costs(ctx),
            // 神鼎·麦星面：确定性判定——红桃回 1/方块摸 1/梅花随机敌弃 1/
            // 黑桃无（内联口径同铁骑）
            HostBehavior::SwdShending => self.step_swd_recovery(ctx),
            // 鬼眼：SelectCards 取目标手牌一张梅花 → 获得之
            HostBehavior::SwdGuiyan => self.step_swd_card_targets(ctx),
            // 小莫雨·山小小面：锁定——回复 1（满血改摸 1）；每回合一次
            //（inst.once_used 消费——limits once_per_turn 走回合重置）
            HostBehavior::SwdXiaomoyu => self.step_swd_recovery(ctx),
            // 袭西·凤钰面：SelectCard 弃一张 he → TargetChoice 攻击距离内
            // 且体力不小于自己的角色 → 1 伤
            HostBehavior::SwdShangxi => self.step_swd_damage_costs(ctx),
            // 花间·剑仙面：SelectCard 弃一张 he → TargetChoice 杀合法目标 →
            // 虚拟杀
            HostBehavior::SwdHuajian => self.step_swd_card_virtual_sha(ctx),
            // 星坠·慕月面：SelectCard 弃一张牌（定类别）→ TargetChoice 有
            // he 目标 → 目标弃同类别一张或受 1 伤
            HostBehavior::SwdXingzhui => self.step_swd_damage_combos(ctx),
            // 焰羽·少微面：SelectCards 弃两张红色牌 → 虚拟炽羽袭（全体其他）
            HostBehavior::SwdHuangyu => self.step_swd_virtual_cards(ctx),
            // 摄引：座位序逐判定区有牌者——弃其判定牌 + 其交一张手牌（循环
            // 面一次机器内完成，确定性座位序登记差异上游 chooseTarget 循环）
            HostBehavior::SwdXianyin => self.step_swd_status(ctx),
            // 破茧·齐面：牌堆→弃堆寻首张装备牌装备之（Equipped 帧+授技走
            // 正式 PlayCard 装备提交路径）
            HostBehavior::SwdPojian => self.step_swd_status(ctx),
            // 横刀·慧彦面：当前回合角色二选一窗（弃同花色 he 牌或被获得
            // 一张）——上游 chooseToDiscard 可取消=被获得面
            HostBehavior::SwdHwendao => self.step_swd_cards(ctx),
            // 凌锋：TargetChoice 攻击距离内 → Confirm 窗（Confirm=1 伤，
            // 取消=自获 1 护甲）
            HostBehavior::SwdLingfeng => self.step_swd_damage_targets(ctx),
            // 天火·秦壬面：判定区有牌的其他角色各——弃判定牌+受等量火伤
            //（nosource 登记差异：原生 Damage 需 source，改 owner 登记差异）
            HostBehavior::SwdTianhuo => self.step_swd_damage_reactions(ctx),
            // 破晓·张烈面：锁定 marker（viewAs 链+距离链+目标上限链三查询）
            HostBehavior::SwdPoxiao => self.step_swd_status(ctx),
            // 疾风·司空宇面：锁定 marker（engine 杀距离校验臂查询），无效果步
            HostBehavior::SwdSliufeng => self.step_swd_status(ctx),
            // 乱击·多鹏面：SelectCards 弃两张同花色手牌 → 虚拟万箭齐发
            //（AOE 全体其他存活，座位序）
            HostBehavior::SwdLuanji => self.step_swd_virtual_cards(ctx),
            // 聒噪·疾鹏面：TargetChoice 距离 2 内有手牌角色（含自己）→
            // 其手牌全部弃置，牌堆顶三张入手（上游 lose._triggered=null +
            // gain "draw" 同口径；登记差异：上游观看面为二段交互，原生
            // 单窗收敛）
            HostBehavior::SwdGuozao => self.step_swd_targets(ctx),
            // 英姿·刷新面：自己摸牌阶段额外摸 1（手牌上限查询在
            // state.hand_limit；此处只补摸牌面）
            HostBehavior::SwdReyingzi => self.step_swd_status(ctx),
            // 枕流·临月面：SelectCard 红色手牌 → TargetChoice 其他未挂附
            // 存活 → off_court 挂附（上游 placeCard 武将牌上同口径；
            // 触发面 SwdZhenjiu2 在其准备阶段取回）
            HostBehavior::SwdZhenjiu => self.step_swd_card_attach(ctx),
            // 麻醉·临月面：SelectCard 黑色手牌 → TargetChoice → off_court
            // 挂附（触发面 SwdLmazui2 在持有者伤害前置返还+降 1）
            HostBehavior::SwdLmazui => self.step_swd_card_attach(ctx),
            // 乐学·久悠面：回合开始授技——自己回合授{制衡/集智}之一，
            // 他人回合授{遗计/急救/鬼道/反馈}之一（RNG 选择；
            // expires_end_of_turn 到期移除。登记差异：上游池含缔盟/驱虎，
            // 原生未实现故回合内池取已实现子集）
            HostBehavior::SwdLexue => self.step_swd_status(ctx),
            // 枕流触发面：off_court 红牌入手并回复 1（登记差异：按花色
            // 过滤取牌——若 off_court 有他技红色挂附可能误取）
            HostBehavior::SwdZhenjiu2 => self.step_swd_recovery(ctx),
            // 圣手·淳于恒面：锁定 marker（engine viewAs 链查询），无效果步
            HostBehavior::SwdShengshou => self.step_swd_status(ctx),
            // 义绝（武将面批 86）：CompareTargetPick 目标选拼点牌 →
            // ComparePlayerPick 己方选牌 → 比点：赢=目标置封印（本回合
            // 基本/锦囊失效）+受 1 伤；输=目标已受伤回 1
            HostBehavior::SwdYijue => self.step_swd_damage_combos(ctx),
            // 拷贝栏：锁定（Draw 内联钩），无独立效果步
            HostBehavior::SwdKbolan => self.step_swd_status(ctx),
            // 奥剑：SelectCard 弃 1..X 张手牌 → TargetChoice 攻击距离内
            // 至多 X 名 → 各 1 伤
            HostBehavior::SwdAojian => self.step_swd_damage_multi(ctx),
            // 化金：SelectCard 弃一张 he → 授予 swd_huajin2
            HostBehavior::SwdHuajin => self.step_swd_cards(ctx),
            // 化金触发面：伤害前置 +1；准备阶段开始移除实例
            HostBehavior::SwdHuajin2 => self.step_swd_damage_reactions(ctx),
            // 明镜：1/3 伤害 -1（等同光纱天衣；无防具门在 filter）
            HostBehavior::SwdMinjing => self.step_swd_status(ctx),
            // 兰芷：使用梅花牌后，体力不大于自己的已受伤角色（含自己）
            // 按座位序各回复 1（上游 list.sort(lib.sort.seat)；恒发动近似）。
            HostBehavior::SwdLanzhi => self.step_swd_recovery(ctx),
            // 玄咒：SelectCard 置出一张普通锦囊（discard:false=置出而非
            // 弃——原生近似弃置登记）→ TargetChoice 任一角色（含自己，
            // 判定区无同名牌）→ 确定性 kind 规则置入判定区
            HostBehavior::SwdXuanzhou => self.step_swd_card_attach(ctx),
            // 因果：确认 → 弃一张 he → DamageBefore 帧 source/target 对调
            //（unique 一次移除实例）
            HostBehavior::SwdYinguo => self.step_swd_damage_reactions(ctx),
            // 唤魂：确认 → SelectCard 弃一张 he → 濒死者判定（内联）：
            // 红→回 1；黑→弃置牌入手濒死者
            HostBehavior::SwdHuanhun => self.step_swd_recovery(ctx),
            // 倾城：frequent——判定循环至多 3 次：红判定牌入手，非红弃置
            // 终止（确定性顺序判定，内联口径同铁骑）
            HostBehavior::SwdLqingcheng => self.step_swd_status(ctx),
            // 狩猎：Sha 机器内联臂承载，无独立效果步
            HostBehavior::SwdShoulie => self.step_swd_status(ctx),
            // 不死：濒死强制判定（内联口径同铁骑）——黑桃照常（落救援/
            // 死亡）；非黑桃体力回至 1+翻面（TurnOverBefore 帧经 commit
            // 含凶兽守卫/急冻解除）；unique 一次后移除实例
            HostBehavior::SwdBusi => self.step_swd_recovery(ctx),
            // 破阵：Act 读差值分流——多：弃差值张→来源 1 伤；少：确认
            // →弃来源差值张（确定性首 N 张）
            HostBehavior::SwdPozhen => self.step_swd_damage_reactions(ctx),
            // 鬼眼·复活面：濒死强制回 1（recover 自清 dying 旗→救援机循
            // 环顶自然 DyingSaved）+ 移除鬼眼双技实例
            HostBehavior::SwdGuiyan2 => self.step_swd_recovery(ctx),
            // 孵鸟：SelectCard 一张手牌 → TargetChoice 其他角色 → 交牌
            // + 摸 1（观看手牌=展示面登记差异）
            HostBehavior::SwdFuniao => self.step_swd_card_targets(ctx),
            // 凝弦：确认 → SelectCards 弃任意张黑色 he 牌 → SelectPlayers
            // 等量目标 → 各受 1 伤
            HostBehavior::SwdNingxian => self.step_swd_damage_multi(ctx),
            // 妖童：锁定 marker（engine viewAs 链查询），无效果步
            HostBehavior::SwdYaotong => self.step_swd_status(ctx),
            // 黑火：SelectCard 弃一张装备牌（he）→ 摸等同弃前手牌数的牌
            // → 若手牌 ≥8 自受 3 火伤
            HostBehavior::SwdHeihuo => self.step_swd_damage_reactions(ctx),
            // 真威：确认 → TargetChoice（非响应者存活角色）→ 弃堆牌入手
            HostBehavior::SwdFzhenwei => self.step_swd_targets(ctx),
            // 遁行：确认 → SelectCard 弃一张 he → 判定非红桃 → 取消该牌
            //（cancelled 写回+桌面归还使用者手牌，工神同款）
            HostBehavior::SwdDunxing => self.step_swd_response_cards(ctx),
            // 极谋·姬良面：回合外受伤后摸 1（上游 jqimou draw 同口径；
            // 使用牌续段=主动选择面登记差异不收）
            HostBehavior::SwdJqimou => self.step_swd_status(ctx),
            // 剑击·少微面：SelectCard 弃装备牌 → TargetChoice 杀合法目标 →
            // 虚拟杀 + 摸 1（上游 useCard sha + draw 同口径）
            HostBehavior::SwdJianji => self.step_swd_card_virtual_sha(ctx),
            // 头弹·麦星面：SelectCard 黑桃 he → TargetChoice 其他存活 →
            // 目标 1 火伤 + 自受害者起（含自己）座位序各弃一张 he
            //（确定性首张登记差异）
            HostBehavior::SwdToudan => self.step_swd_damage_costs(ctx),
            // 玄影·端蒙面（受伤反击面）：解除横置 + 对来源虚拟杀
            //（ActivateSkill 面走下方 stage 分支——两入口按触发帧区分：
            // 帧存在且为 Damage=反击面；无帧=主动面）
            HostBehavior::SwdXuanying => self.step_swd_damage_targets(ctx),
            // 玄影伤害子技：与父技同一台受伤反击机（按触发帧区分面）。
            HostBehavior::SwdXuanyingDamage => self.step_swd_damage_targets(ctx),
            // 烈刃：与受害者拼点——CompareTargetPick 目标选牌 →
            // ComparePlayerPick 己方选牌 → 赢者获得对方拼点牌+随机一张 he
            HostBehavior::SwdLieren => self.step_swd_damage_reactions(ctx),
            // 仙匠·车芸面：SelectCard 目标装备区含未持有种（消费段校验）
            // → 实体发现同牌种牌入手+Equipped 帧全托管装备
            //（take_hand_card+同栏替换+grant_equip_skills——破茧同款）
            HostBehavior::SwdCyxianjiang => self.step_swd_cards(ctx),
            // 命符·姜子牙面：SelectCard 梅花手牌 → TargetChoice 任一其他
            // 存活 → 弃梅花牌+实体发现鬼幽结置其判定区（上游 prepare throw
            // + addJudge 同口径；登记差异：实体发现替代 createCard；同名
            // 判定区重复由命令期校验承担）
            HostBehavior::SwdMingfu => self.step_swd_card_attach(ctx),
            // 幻形（武将面批 87）：SelectCard 弃一张 he → TargetChoice
            // 男性其他存活角色 → 复制其（非 unique/未持有）技能实例并
            // 置 huanxing_granted（受伤后移除）
            HostBehavior::SwdHuanxing => self.step_swd_card_targets(ctx),
            // 幻形·移除面：移除幻形授予的技能实例（在位旗标=内部状态技能
            // 实例；按其他玩家将技能全表尝试移除——登记差异：不追踪具体
            // 授予集）
            HostBehavior::SwdHuanxing2 => self.step_swd_status(ctx),
            // 荡平（武将面批 86）：SelectCard 弃一张 he → TargetChoice 受害
            // 者距离 ≤1 的另一名角色 → 1 伤
            HostBehavior::SwdDangping => self.step_swd_damage_costs(ctx),
            // 活弹·舒莞儿面：SelectCard 红色手牌 → TargetChoice 1-2 名任一
            // 存活（1 名=2 火伤/2 名=各 1 火伤）→ 自失 1 体力（contentBefore
            // 同口径：先弃牌再失体力）
            HostBehavior::SwdHuodan => self.step_swd_damage_multi(ctx),
            // 激昂·凤天凌面：摸 1（frequent 非 optional，无效果步以外的
            // 状态变化）
            HostBehavior::SwdJiang => self.step_swd_status(ctx),
            // 麻醉触发面：伤害前置——off_court 黑牌返还（登记差异：上游
            // 伤害后归还，原生伤前归还）并令本次伤害 -1
            HostBehavior::SwdLmazui2 => self.step_swd_damage_reactions(ctx),
            // 愚钝·福面：SelectCards 两张锦囊手牌（消费段校验）→
            // TargetChoice 攻击距离内其他存活 → 首张作 declared-Sha 载荷
            //（use_as 声明）进入杀结算，次张弃置（丈八蛇矛机同型，
            // 不计出杀次数经 kernel 豁免臂）
            HostBehavior::SwdYudun => self.step_swd_card_virtual_sha(ctx),
            // 占星·肯迪面：SelectCards 弃任意张手牌 → 亮等量牌堆顶牌：
            // 方块摸 2/红桃回 1 或 +1 护甲/梅花各敌人随机弃 1/黑桃选一名
            // 角色受 1 无来源雷伤
            HostBehavior::SwdZhanxing => self.step_swd_damage_multi(ctx),
            // 运筹·姬亮面：SelectCard 弃一张手牌 → TargetChoice 有手牌
            // 目标 → 目标随机弃一张手牌：同色自己摸 1 否则目标摸 1
            HostBehavior::SwdYunchou => self.step_swd_card_targets(ctx),
            // 凌波·恒爱面：frequent——闪响应/使用后摸 2
            HostBehavior::SwdHlingbo => self.step_swd_status(ctx),
            // 血裔·Miles 面：锁定——伤害前置取消+失去 1 点体力（无条件；
            // Boss 绝情同窗）
            HostBehavior::SwdMilesXueyi => self.step_swd_damage_reactions(ctx),
            // 观虎：确认后——弃目标一张手牌+一张装备（确定性首张，登记
            // 差异上游自选 discardPlayerCard 可选面）
            HostBehavior::SwdGuanhu => self.step_swd_damage_reactions(ctx),
            // 灵心：frequent——牌堆顶三张（pop×3）：红桃入手，其余弃置
            HostBehavior::SwdLingxin => self.step_swd_status(ctx),
            // 渔人：锁定（sha 机器内联臂查询），无独立效果步
            HostBehavior::SwdYueren => self.step_swd_status(ctx),
            // 无邪：锁定 marker（engine 延时校验臂查询），无独立效果步
            HostBehavior::SwdWuxie => self.step_swd_status(ctx),
            // 幻剑：锁定 marker（engine PlayCard viewAs 链查询），无效果步
            HostBehavior::SwdHuanjian => self.step_swd_status(ctx),
            // 乖戾：Before 帧 +1（缚斧同款直写）；After 帧弃两张手牌
            //（手牌不足弃全部）
            HostBehavior::SwdGuaili => self.step_swd_damage_reactions(ctx),
            // 虚影：锁定——伤害前置取消 + 手牌非空则失去 1 点体力（Boss
            // 绝情同窗：帧 cancelled 写回+LoseHp 发射）
            HostBehavior::SwdXuying => self.step_swd_damage_reactions(ctx),
            // 乐跃：TargetChoice-free 分流——回复者非自己且其手牌≥自己则
            // 各摸 1，否则回复者摸 1
            HostBehavior::SwdYinyue => self.step_swd_status(ctx),
            // 对诗：Act 弃一张手牌 → SelectCard 指定目标 → TargetChoice
            // 目标同花色弃牌窗 → CardsAsk 消费（空选=拒绝→随机获得其一
            // 张）。usable:1 收敛上游 duishi2/duishi3 临时技面（登记差异）。
            HostBehavior::SwdDuishi => self.step_swd_card_contests(ctx),
            // 玉蝉换形: 手牌中每张玉蝉实体轮换为八卦序下一 kind 的实体
            //（牌堆/弃堆确定性发现首张；登记差异上游随机+init 原地变形）
            HostBehavior::SwdYuchanSwap => self.step_swd_equipment_skills(ctx),
            // 逆攻发动：TargetChoice 选目标 → Act 弃全部标记——奇数摸 1+
            // floor(标记/2) 伤；标记=1 时仅摸 1（activate 分流两 activate 面）
            HostBehavior::SwdNigongStrike => self.step_swd_equipment(ctx),
            // 女娲石：确认门后濒死者回复 1、自己失去 1（hp>1 门在 filter）
            HostBehavior::SwdNvwashiGuard => self.step_swd_equipment(ctx),
            // 昆仑镜：Act 亮牌堆顶 3 选 1 → SelectCard 手牌替换（弃 1 手
            // 牌，登记差异上游替换牌入特殊区）→ TargetChoice 获得选中牌
            HostBehavior::SwdKunlunjingcCard => self.step_swd_card_effects(ctx),
            // 东皇钟：SelectCard 弃 1 红牌 → TargetChoice 选目标 → Act 从
            // 牌堆发现延时锦囊实体（Le/Bing/ShanDian 且目标判定区未有）置
            // 入判定区（实体发现替代 createCard 凭空造；登记差异随机→首张）
            HostBehavior::SwdDonghuangzhongCard => self.step_swd_card_effects(ctx),
            // 分舵（习嫣 jiefen）：三段——选交牌目标（手>自己）→ 目标选
            // 手牌交出 → 自己选 1 张手牌交给手牌最少的其他存活角色（登
            // 记差异：上游 chooseCardTarget 同时选牌与目标，原生确定性
            // 目标两段询问）。
            HostBehavior::SwdJiefen => self.step_swd_card_targets(ctx),
            // 复仇：目录锚（取消逻辑在 ShaResolveMachine 内联）
            HostBehavior::WudianFuchou => self.step_wudian_status(ctx),
            // 坚壁：锁定查询面（AOE 目标段查询），无独立效果步
            HostBehavior::WudianJianbi => self.step_wudian_status(ctx),
            // 厉焰：目录锚（转化逻辑在 PlayCard 门，无独立效果步）
            HostBehavior::YunchouLuweiyan => self.step_yunchou(ctx),
            // 雄姿：补摸 1+⌊装备数/2⌋
            HostBehavior::YunchouXiongzi => self.step_yunchou(ctx),
            // 雅量：伤害双方各摸 1（座位序逐个）
            HostBehavior::YunchouYaliang => self.step_yunchou(ctx),
            // 蝉缘·变体：来源体力上限 -1（体力钳上限）
            HostBehavior::YunchouDiychanyuan => self.step_yunchou(ctx),
            // 理断：同栏已有装备 → 其交首张手牌（登记差异：上游自选）；
            // 否则直接装备该牌（不走 Equipped 帧防双 grant）
            HostBehavior::WudianLiduan => self.step_wudian_status(ctx),
            // 复仇·清算：标记来源逐个摸 1+虚拟杀强制指定（死亡移除）
            HostBehavior::WudianFuchou2 => self.step_wudian_status(ctx),
            // 悲歌（refresh 变体）：判定→弃 1 张（可取消）→按判定花色
            // 分支+收回奖励。登记差异：草花来源弃 2 取首两张确定性；
            // addExpose（AI 面）不做。
            HostBehavior::YunchouBeige => self.step_yunchou(ctx),
            // 铁壁：弃首张黑牌 → 目标 +1 护甲（登记差异：上游玩家选弃牌）
            HostBehavior::SwdTiebi => self.step_swd_status(ctx),
            // 流虹：使用杀后摸 1
            HostBehavior::SwdLiuhong => self.step_swd_status(ctx),
            // 墨剑：目标摸 1 + 自己回 1（未满血门）
            HostBehavior::SwdMojian => self.step_swd_recovery(ctx),
            // 道法：来源弃首张手牌（登记差异：上游玩家自选）
            HostBehavior::SwdDaofa => self.step_swd_status(ctx),
            // 玉鳞：Act 问选装备牌（手/装备区）→ SelectCard 弃置+免伤；
            // 取消=不免伤（照常伤害）
            HostBehavior::SwdYulin => self.step_swd_response_cards(ctx),
            // 凝霜：Act 问选黑牌 → SelectCard 弃置+使用者阶梯
            // （已翻面→失 1 体力；已横置→翻面；否则→横置且自己摸 1）
            HostBehavior::SwdNingshuang => self.step_swd_response_cards(ctx),
            // 工神：Act 问选装备牌 → SelectCard 弃置+取消该使用（筑城
            // cancelled 写回同款；取消=使用照常）
            HostBehavior::SwdGongshen => self.step_swd_response_cards(ctx),
            // 还灵：Act 问选其他角色 → TargetChoice 自己与目标同时翻面
            //（取消=不发动；上游 turnOver×2）
            HostBehavior::SwdHuanling => self.step_swd_targets(ctx),
            // 断月：SelectCard 选装备牌（手/装备区）→ TargetChoice 选其
            // 他角色 → Act 弃牌+造成 1 点伤害（牌留区到末段）
            HostBehavior::SwdDuanyue => self.step_swd_damage_costs(ctx),
            // 屠阵：弃置受伤者全部非基本手牌（确定性整组）
            HostBehavior::SwdTuzhen => self.step_swd_status(ctx),
            // 落梅：使用或打出梅花牌摸 1
            HostBehavior::SwdLuomei => self.step_swd_status(ctx),
            // 星点：SelectCard 弃 1 张手牌 → Act 确定性两名其他存活角色
            // 各弃手/装备区首张（登记差异：上游随机 2 名敌人各随机弃 1）
            HostBehavior::SwdXingdian => self.step_swd_cards(ctx),
            // 协律：确认门后回 1（未满血门）+ 弃置场上全部判定牌（座位序）
            HostBehavior::SwdXielv => self.step_swd_recovery(ctx),
            // 朱羽：Act 问选红牌 → SelectCard 弃置+伤害 +1 且改火属性
            //（取消=伤害照常）
            HostBehavior::SwdZhuyu => self.step_swd_damage_reactions(ctx),
            // 雪煌：limited 置位 → 弃全部手牌 → 逐张视为对（至多）两名
            // 其他存活角色使用火杀（座位序确定性；登记差异上游随机 2 名
            // 敌人；目标中途死亡即终止该张）
            HostBehavior::SwdXuehuang => self.step_swd_status(ctx),
            // 大同：全场极差 ≤1 → 摸 2
            HostBehavior::SwdDatong => self.step_swd_status(ctx),
            // 承剑：来源摸 1
            HostBehavior::SwdChengjian => self.step_swd_status(ctx),
            // 缚斧：DamageBefore 帧直写 num+1（乱剑先例）
            HostBehavior::SwdJufu => self.step_swd_damage_reactions(ctx),
            // 花镜：回 1 + 摸 1（受伤者判定面：上游回 1 无判定——纯回摸）
            HostBehavior::SwdHuajing => self.step_swd_recovery(ctx),
            // 护盾：自己无护甲 → +1
            HostBehavior::SwdHudun => self.step_swd_status(ctx),
            // 破星：目标体力 > 持有者 → 伤害 +1（帧直写）
            HostBehavior::SwdPoxing => self.step_swd_damage_reactions(ctx),
            // 良策·余获：锁定查询面（五谷机器内联查询），无独立效果步
            HostBehavior::WudianLiangceRemain => self.step_wudian_status(ctx),
            // 归汉：目录锚（无独立效果步）
            HostBehavior::WudianGuihan => self.step_wudian_status(ctx),
            HostBehavior::SwdGTiandao => self.step_swd_status(ctx),
            // 热血：对阶段玩家视为使用一张杀，然后摸 1
            // （登记差异：虚拟杀不消耗实体杀；恒发动近似）
            HostBehavior::SwdRexue => self.step_swd_status(ctx),
            // 幻音：判定——黑桃取消该牌、红桃摸 1、其余无事
            HostBehavior::SwdHuanyin => self.step_swd_status(ctx),
            // 灵弦：与座位序首个攻击距离 >1 的其他存活者各摸 1
            // （登记差异：自选目标→首个确定性近似；可放弃→恒发动近似）
            HostBehavior::SwdLingxian => self.step_swd_status(ctx),
            // 拼点（deferred 消解批 独孤宁珂）：弃一张手牌，令两名角色拼点：
            // 赢者获得该牌、输者受赢者 1 伤（拼点牌两目标自选；登记差异：
            // 目标/拼点牌座位序与确定性近似）
            HostBehavior::SwdNlianji => self.step_swd_damage_combos(ctx),
            // 蛇魂（deferred 消解批 山小小）：弃 1–4 张手牌并选一名有牌角色：
            // 其弃等量 he 牌，若其中有与弃牌同花色者则其受 1 伤
            // （登记差异：花色互异约束未强制——任意 1–4 张）
            HostBehavior::SwdShehun => self.step_swd_damage_combos(ctx),
            // 魂身（deferred 消解批 胡月）：使用/打出闪后计数 +1
            HostBehavior::SwdHyunshen => self.step_swd_status(ctx),
            // 魂身·收束：准备阶段摸等量并清零
            HostBehavior::SwdHyunshen2 => self.step_swd_status(ctx),
            // 待行：结束阶段弃 1..张 he 换取等量护甲
            HostBehavior::SwdDaixing => self.step_swd_cards(ctx),
            // 待行·清甲：准备阶段按剩余计数移除护甲
            HostBehavior::SwdDaixing2 => self.step_swd_status(ctx),
            // 待行·吸收：受伤后递减计数（近似：min(计数, 伤害)）
            HostBehavior::SwdDaixing3 => self.step_swd_status(ctx),
            // 聚悉·计数：非当前回合角色弃牌后计数 +1
            HostBehavior::SwdJuxiCount => self.step_swd_status(ctx),
            // 破舟：受伤后计数累加伤害值
            HostBehavior::SwdPozhou => self.step_swd_status(ctx),
            // 破舟·封印：消耗 1 计数，封印一名其他角色（全部技能禁用至其
            // 下一阶段开始；登记差异：上游至其下一回合结束）
            HostBehavior::SwdPozhou2 => self.step_swd_targets(ctx),
            // 陷阱：出牌阶段置出一张花色未在陷阱表中的手牌为陷阱
            HostBehavior::SwdSxianjing => self.step_swd_cards(ctx),
            // 陷阱·触发：同花色陷阱弃置 + 随机获得使用者一张牌（首个 he）
            HostBehavior::SwdSxianjingGain => self.step_swd_status(ctx),
            // 陷阱·取回：受伤后随机取回一张陷阱（确定性：首个）
            HostBehavior::SwdSxianjingDamage => self.step_swd_status(ctx),
            // 孤行：将全部手牌当杀使用（1–3 目标），结算后摸伤害数
            HostBehavior::SwdGuxing => self.step_swd_targets(ctx),
            // 孤行·计数：孤行杀造成伤害 +1
            HostBehavior::SwdGuxing3 => self.step_swd_status(ctx),
            // 灵矢：锁定旗标行为（无机器步；距离/杀限次/直中/摸牌内联）
            HostBehavior::SwdLingshi => self.step_swd_status(ctx),
            // 锦鳞：选任意名角色各 +3 护甲并授予「锦鳞·退」（计数 3）
            HostBehavior::SwdJinlin => self.step_swd_targets(ctx),
            // 禳星：累计计数；达 3 判定免除/失星，随后分配 3 点雷伤
            HostBehavior::HlssXiangxing => self.step_skills_damage(ctx),
            // 隐月：伤害前置窗——num+1 写回（上游 damageBegin4 num++），
            // 并记录伤害类型（雷/火/锦囊牌；上游 damage 面记录，原生同窗
            // 记录，登记差异：防止伤害的窗口内已记录）
            HostBehavior::HlssYueyin => self.step_skills_status(ctx),
            // 起风：选一张未用过的可用锦囊牌名并使用（上游 chooseButton
            // + chooseUseTarget；原生以目录牌种码承载牌名选择）
            HostBehavior::HlssFengqi => self.step_skills_virtual_use(ctx),
            // 起风·闭 / 改命：锁定旗标（无机器步；无懈门/判定机内联）
            HostBehavior::HlssFengqi2 => self.step_skills_status(ctx),
            HostBehavior::HlssGaiming => self.step_skills_status(ctx),
            // 潜龙：按触发面累加道心值（上限 99），随后按阈值刷新解锁
            HostBehavior::MbQianlong => self.step_skills_status(ctx),
            // 涅槃：弃全部牌 + 解除横置 + 翻回正面 + 摸 3 + 回复至 3；
            // 随后触发浴火（上游独立 niepanAfter 事件面 → 原生链式，登记差异）
            HostBehavior::Niepan => self.step_skills_status(ctx),
            HostBehavior::Yuhuo => self.step_skills_status(ctx),
            HostBehavior::Shenwei => self.step_skills_status(ctx),
            // 集控：选攻击距离内一名其他角色 → 虚拟雷杀（取消=不消耗次数）
            // 每步：弃一张牌 → 令目标本回合止息（+ 条件性距离视为 1）
            HostBehavior::NewMeibu => self.step_skills_virtual_use(ctx),
            // 止息：标记 +1；锦囊 → 置锦囊记录；显示标记 = 体力 - 计数
            HostBehavior::NewZhixi => self.step_skills_status(ctx),
            // 撷芳：刷新「存活女性数」标记（0 时清除）
            HostBehavior::Xiefang => self.step_skills_status(ctx),
            // 让节：按伤害点数逐次——获得牌（摸 1 + 牌堆取指定类型）或移动牌
            HostBehavior::Rangjie => self.step_skills_cards(ctx),
            // 蛊惑：弃一张手牌 → 声明基本/锦囊牌名 → 选目标 → 转化使用
            HostBehavior::Reguhuo => self.step_skills_virtual_use(ctx),
            // 攻心：选目标 → 看其手牌选一张红桃 → 弃置或置于牌堆顶
            HostBehavior::Gongxin => self.step_skills_cards(ctx),
            // 魔化：移除旧三技 → 获得撒旦四技 → 上限 4 → 回复 2 → 摸 2
            HostBehavior::Mohua2 => self.step_skills_recovery(ctx),
            // 修罗：选一张手牌 → 选一张同花色判定区牌 → 一并弃置
            HostBehavior::Xiuluo => self.step_skills_discard_costs(ctx),
            // 灭道·弃：强制弃「已损失体力」张（手/装备）
            HostBehavior::Miedao => self.step_skills_discard_costs(ctx),
            // 魔焰：选 N 张红牌 → 选恰 N 名其他角色 → 逐个 1 点火焰伤害
            HostBehavior::Moyan => self.step_skills_damage(ctx),
            // 疾风：多目标使用 → 按概率整牌取消（上游 `trigger.cancel()`）
            HostBehavior::Jifeng => self.step_skills_status(ctx),
            // 昆仑镜：阶段收束记录快照 / 阶段开始（机器内确认门）回滚
            HostBehavior::Kunlunjing => self.step_skills_status(ctx),
            HostBehavior::Shengong => self.step_skills_status(ctx),
            // 轩辕剑·伤增：帧内 +1 并转雷属性、置标记
            HostBehavior::XuanyuanjianDamage => self.step_skills_damage(ctx),
            // 轩辕剑·反噬：失去 1 点体力（无「轩辕」时）
            HostBehavior::XuanyuanjianBacklash => self.step_skills_status(ctx),
            // 轩辕：弃一张黑桃牌 → 从弃牌堆取【轩辕剑】装备（限定）
            HostBehavior::Xuanyuan => self.step_skills_discard_costs(ctx),
            // 燎原：循环弃同花色牌（首张必问；弃后余 >1 张才续问）→ 记额外闪数
            HostBehavior::Liaoyuan => self.step_skills_discard_costs(ctx),
            // 改造：选一个空栏位 → 写入 declared_slots（提交按新栏位入槽）
            HostBehavior::Gaizao => self.step_skills_status(ctx),
            // 天书：弃一张锦囊手牌 → 选角色 → 选其一个未持有技能 → 永久获取
            HostBehavior::SwdTianshu => self.step_swd_card_contests(ctx),
            // 李香龙：选攻击范围内目标 → 使用虚拟杀
            HostBehavior::Lxianglong => self.step_skills_targets(ctx),
            // 移花：弃 2 张手牌 → 帧「使用者/目标」对调（反弹）
            HostBehavior::Yihua => self.step_skills_discard_costs(ctx),
            // 贞烈：失去 1 体力 → 移出目标 → 弃置使用者一张牌（手/装备）
            HostBehavior::Zhenlie => self.step_skills_discard_costs(ctx),
            HostBehavior::Tianlun => self.step_skills_status(ctx),
            // 脉轮：选 1..8 效果 → 立即结算（1/2）或置模式位（3..8）
            HostBehavior::Mailun => self.step_skills_recovery(ctx),
            // 极略：看牌堆顶 3 张 → 选一张基本/锦囊牌 → 选目标 → 使用
            HostBehavior::Jilve => self.step_skills_virtual_use(ctx),
            // 革新印：选花色 → 静默弃该花色全部手牌 → 选另一花色 → 获得等量
            HostBehavior::Gxianyin => self.step_skills_status(ctx),
            // 冰封·封：翻回正面 → 移除自身
            HostBehavior::Bingfeng2 => self.step_skills_status(ctx),
            // 冰封：翻面自己与至多 3 名目标 + 失去体力上限 + 授予封锁
            HostBehavior::Bingfeng => self.step_skills_targets(ctx),
            // 此桥邪：失去装备后逐次使用机关牌（自目标/单目标/AOE）
            HostBehavior::Cyqiaoxie => self.step_skills_virtual_use(ctx),
            // 魂殉志：万箭齐发 + 授予武圣/咆哮/殉 + 觉醒自身
            HostBehavior::Hxunzhi => self.step_skills_status(ctx),
            HostBehavior::Hxunzhi2 => self.step_skills_status(ctx),
            // 护天：选任意张牌 → 选体力上限足够的目标 → 寄存 + 补至张数
            HostBehavior::Hutian => self.step_skills_recovery(ctx),
            // 天香：选红桃手牌 → 选目标 → 弃牌 + 伤害转移 + 授予天香·受
            HostBehavior::Tianxiang => self.step_skills_damage(ctx),
            // 天香·受：按本次伤害量摸牌后移除自身
            HostBehavior::Tianxiang2 => self.step_skills_status(ctx),
            // 颂词：选未记录角色 → 手牌多于体力弃 2 否则摸 2 → 记录
            HostBehavior::Songci => self.step_skills_cards(ctx),
            // 诛害：选一张手中的【杀】→ 对结束阶段角色使用（实体牌）
            HostBehavior::SwdZhuhai => self.step_swd_virtual_cards(ctx),
            // 林韵：两张牌当杀（手牌或装备；首张为载荷牌，declared Sha）
            HostBehavior::SwdLinyun => self.step_swd_card_virtual_sha(ctx),
            HostBehavior::SwdJikong => self.step_swd_targets(ctx),
            // 魅惑：选目标 → 取其一张牌（装备牌则装备之）
            HostBehavior::SwdMeihuo => self.step_swd_card_targets(ctx),
            // 守印：置限定位 + 自翻面 + 全员回复至满（座位序）
            HostBehavior::SwdShouyin => self.step_swd_recovery(ctx),
            // 义山：取回最近失去且仍在他处的至多 2 张（新在前）
            HostBehavior::SwdYishan => self.step_swd_status(ctx),
            // 入梦：弃非基本牌 → 目标弃基本牌或跳过出牌/弃牌阶段
            HostBehavior::SwdRumeng => self.step_swd_cards(ctx),
            // 神焰：前置面（+1 并记录蔓延目标）与结算面（逐个蔓延火伤）
            HostBehavior::SwdShenyan => self.step_swd_damage_reactions(ctx),
            HostBehavior::SwdLingwu => self.step_swd_status(ctx),
            // 清正：选花色 → 选目标 → 选目标花色 → 双向弃牌 → 少弃者受伤
            HostBehavior::MbcmQingzheng => self.step_skills_damage(ctx),
            // 卫统：登记面（实际加成由潜龙内容按同条件判定）
            HostBehavior::MbWeitong => self.step_skills_status(ctx),
            // 隐月·判：取消翻面（写回前置帧取消位）
            HostBehavior::HlssYueyin2 => self.step_skills_status(ctx),
            // 锦鳞·退：阶段开始 -1 护甲与计数；护甲归零或计数归零移除
            HostBehavior::SwdJinlin2 => self.step_swd_status(ctx),
            // 聚悉：扣存活数并选一名角色——已受伤则自选受伤/回复，否则受伤
            HostBehavior::SwdJuxi => self.step_swd_damage_targets(ctx),
            // 禁言：查询锚（PlayCard 门），无独立效果步
            HostBehavior::WudianJinyan => self.step_wudian_status(ctx),
            // 屯田·DIY：自选一张装备牌（手/场）弃之，牌堆随机获得一张
            // 装备牌（实体发现模式；deck+discard 无装备则仅弃置）
            HostBehavior::WudianDiyjuntun => self.step_wudian_cards(ctx),
            // 隙焰：弃首张红手牌（登记差异）→ 全体其他存活座位序各 1 火
            HostBehavior::YunchouJieyan => self.step_yunchou(ctx),
            // 野升：自选任意张黑手牌 → 弃所选摸等量（舒筋散机器同构）
            HostBehavior::YunchouYiesheng => self.step_yunchou(ctx),
            // 厉焰·链杀：对座位序首目标视为用杀（虚拟杀直发；登记差异：
            // 上游玩家选目标+可取消，原生自动首目标）
            HostBehavior::YunchouLuweiyan2 => self.step_yunchou(ctx),
            // 蛊惑·变体：摸 2 然后弃自己区域 2 张（判定区→装备→手牌首张序）
            HostBehavior::YunchouDiyguhuo => self.step_yunchou(ctx),
            // 狡黠：摸 1
            HostBehavior::YunchouJiaoxia => self.step_yunchou(ctx),
            // 红莲：来源弃其手牌与装备区全部红牌（装备走 revoke 同款）
            HostBehavior::YunchouHonglian => self.step_yunchou(ctx),
            // 纵火：弃首张手牌（登记差异）→ 直写帧 nature=Fire
            HostBehavior::YunchouZonghuo => self.step_yunchou(ctx),
            // 烧营：座位序首合法目标（受伤者距离 ≤1 且非受伤者）→ 弃堆顶
            // 牌亮出，红色则其受 1 点火伤（登记差异：上游玩家选目标）
            HostBehavior::YunchouShaoying => self.step_yunchou(ctx),
            // 戏才：获得伤害牌（table 优先、弃堆兜底；登记近似：不校验
            // 上游 position 'o' 处理区位）
            HostBehavior::YunchouXicai => self.step_yunchou(ctx),
            // 狂骨·变体：目标攻击距离 >1 摸 1，否则已受伤回复 1
            HostBehavior::YunchouDiykuanggu => self.step_yunchou(ctx),
            HostBehavior::ShaDamageForceDiscard => self.step_skills_status(ctx),
            HostBehavior::PhaseEndRecoverOrDraw => self.step_skills_recovery(ctx),
            HostBehavior::TargetDiscardEquipOrDamage => self.step_skills_damage(ctx),
            HostBehavior::ShengkeCheck => self.step_skills_status(ctx),
            HostBehavior::ChangeCardNature => self.step_skills_card_use(ctx),
            HostBehavior::MaxRangeWeaponDamage => self.step_skills_damage(ctx),
            HostBehavior::WangmeiZhikeCard => self.step_skills_recovery(ctx),
            HostBehavior::SijiDiscardDraw => self.step_skills_status(ctx),
            HostBehavior::SanjianWeaponDamage => self.step_skills_damage(ctx),
            HostBehavior::MoshouImmunity => self.step_skills_status(ctx),
            HostBehavior::ShuigongCard => self.step_skills_status(ctx),
            HostBehavior::FudichouxinCompare => self.step_skills_card_use(ctx),
            HostBehavior::ChenhuodajieCard => self.step_skills_status(ctx),
            HostBehavior::TouliangSwap => self.step_skills_card_use(ctx),
            HostBehavior::LinghunzhihuoCard => self.step_skills_damage(ctx),
            HostBehavior::ZhaomingdanCard => self.step_skills_status(ctx),
            HostBehavior::JihuocardDraw => self.step_skills_status(ctx),
            HostBehavior::GeanguanhuoCompare => self.step_skills_card_use(ctx),
            HostBehavior::CaochuanCard => self.step_skills_damage(ctx),
            HostBehavior::ShezhanqunruCompare => self.step_skills_status(ctx),
            HostBehavior::GujianShujinsanCard => self.step_gujian(ctx),
            HostBehavior::GujianMianliCard => self.step_gujian(ctx),
            HostBehavior::GujianFeibiaoCard => self.step_gujian(ctx),
            HostBehavior::JinlianzhuCancel => self.step_skills_status(ctx),
            HostBehavior::GujianFoodTanhuadong => self.step_gujian(ctx),
            HostBehavior::GujianFoodYougeng => self.step_gujian(ctx),
            HostBehavior::GujianFoodGrant => self.step_gujian(ctx),
            HostBehavior::GujianFoodQingtuan => self.step_gujian(ctx),
            HostBehavior::GujianFoodYuanbaorou => self.step_gujian(ctx),
            HostBehavior::GujianFoodGudonggeng => self.step_gujian(ctx),
            HostBehavior::GujianFoodMapodoufu => self.step_gujian(ctx),
            HostBehavior::GujianFoodChunbing => self.step_gujian(ctx),
            HostBehavior::GujianFoodLiyutang => self.step_gujian(ctx),
            HostBehavior::GujianFoodMolicha => self.step_gujian(ctx),
            HostBehavior::GujianFoodXiajiao => self.step_gujian(ctx),
            HostBehavior::GujianFoodMizhilianou => self.step_gujian(ctx),
            HostBehavior::GujianFoodLuyugeng => self.step_gujian(ctx),
            HostBehavior::JiguanBingpoCard => self.step_jiguan(ctx),
            HostBehavior::JiguanQiankunCard => self.step_jiguan(ctx),
            HostBehavior::JiguanLongxuCard => self.step_jiguan(ctx),
            HostBehavior::JiguanShenhuoCard => self.step_jiguan(ctx),
            HostBehavior::GujianFoodZiyangdan => self.step_gujian(ctx),
            HostBehavior::GujianYunvyuanshen => self.step_gujian(ctx),
            HostBehavior::GujianFoodShihuifenBan => self.step_gujian(ctx),
            HostBehavior::GujianShihuifenCard => self.step_gujian(ctx),
            HostBehavior::GujianLiufengsanCard => self.step_gujian(ctx),
            HostBehavior::JiguanShatangCard => self.step_jiguan(ctx),
            HostBehavior::JiguanHeilongCard => self.step_jiguan(ctx),
            HostBehavior::JiguanLiutougeCard => self.step_jiguan(ctx),
            HostBehavior::GujianMutouEquipSkill => self.step_gujian(ctx),
            HostBehavior::JiguanDinvCard => self.step_jiguan(ctx),
            HostBehavior::GujianYuhengEquipSkill => self.step_gujian(ctx),
            HostBehavior::HearthDunpaiCard => self.step_hearth(ctx),
            HostBehavior::HearthZhiliaoboCard => self.step_hearth(ctx),
            HostBehavior::HearthYuansuCard => self.step_hearth(ctx),
            HostBehavior::HearthXingjieCard => self.step_hearth(ctx),
            HostBehavior::HearthChuansongCard => self.step_hearth(ctx),
            HostBehavior::HearthTansheCard => self.step_hearth(ctx),
            HostBehavior::HearthShijieshuCard => self.step_hearth(ctx),
            HostBehavior::GwentYanzhiCard => self.step_gwent(ctx),
            HostBehavior::GwentAozuCard => self.step_gwent(ctx),
            HostBehavior::GwentWenyiCard => self.step_gwent(ctx),
            HostBehavior::GwentZhihuanCard => self.step_gwent(ctx),
            HostBehavior::GwentBaoxueCard => self.step_gwent(ctx),
            HostBehavior::GwentQinpenState | HostBehavior::GwentBirinState => self.step_gwent(ctx),
            HostBehavior::GwentCiguState => self.step_gwent(ctx),
            HostBehavior::GwentWeatherGrant => self.step_gwent(ctx),
            HostBehavior::GwentKunenState => self.step_gwent(ctx),
            HostBehavior::GwentKunenCard => self.step_gwent(ctx),
            HostBehavior::GwentShanbengCard => self.step_gwent(ctx),
            HostBehavior::GwentPoxiaoCard => self.step_gwent(ctx),
            HostBehavior::GwentFengyinState => self.step_gwent(ctx),
            HostBehavior::GwentZumoCard => self.step_gwent(ctx),
            HostBehavior::HuanleMonkeyCard => self.step_skills_status(ctx),
            HostBehavior::HuanleXuelunyangCard => self.step_skills_targets(ctx),
            HostBehavior::HuanleMianjuCard => self.step_skills_status(ctx),
            HostBehavior::ZhangbaSha => self.step_skills_card_use(ctx),
            HostBehavior::SwdLiuxinghuoyuCard => self.step_swd_card_effects(ctx),
            HostBehavior::SwdShentouEquipSkill => self.step_swd_equipment_skills(ctx),
            HostBehavior::SwdYufuluEquipSkill => self.step_swd_equipment_skills(ctx),
            HostBehavior::SwdYangpijuanCard => self.step_swd_card_effects(ctx),
            HostBehavior::SwdLangeguaiyiEquipSkill => self.step_swd_equipment_skills(ctx),
            HostBehavior::MtgLongshanfengLand => self.step_mtg(ctx),
            HostBehavior::MtgHaidaoLand => self.step_mtg(ctx),
            HostBehavior::MtgShuimomuxueLand => self.step_mtg(ctx),
            HostBehavior::MtgFeixuLand => self.step_mtg(ctx),
            HostBehavior::MtgYixialanLand => self.step_mtg(ctx),
            HostBehavior::MtgYixialanRecast => self.step_mtg(ctx),
            HostBehavior::MtgShamolvzhouLand => self.step_mtg(ctx),
            HostBehavior::MtgLinzhongjiantaLand => self.step_mtg(ctx),
            HostBehavior::MtgLinzhongjiantaStrike => self.step_mtg(ctx),
            HostBehavior::MtgBingheyaosaiLand => self.step_mtg(ctx),
            HostBehavior::MtgYoulinLand => self.step_mtg(ctx),
            HostBehavior::MtgYoulinDiscard => self.step_mtg(ctx),
            HostBehavior::MtgLindixiliuLand => self.step_mtg(ctx),
            HostBehavior::WudianCiqiu => self.step_wudian_status(ctx),
            HostBehavior::WudianShuaiyan => self.step_wudian_cards(ctx),
            HostBehavior::WudianZhucheng => self.step_wudian_status(ctx),
            HostBehavior::WudianZhuchengGuard => self.step_wudian_cards(ctx),
            HostBehavior::WudianKangyin => self.step_wudian_cards(ctx),
            HostBehavior::WudianChezhen => self.step_wudian_status(ctx),
            HostBehavior::WudianJuedao => self.step_wudian_status(ctx),
            HostBehavior::WudianGeju => self.step_wudian_status(ctx),
            HostBehavior::WudianTuji => self.step_wudian_status(ctx),
            HostBehavior::WudianMujing => self.step_wudian_ow_cards(ctx),
            HostBehavior::WudianGuangshu => self.step_wudian_ow_cards(ctx),
            HostBehavior::WudianGuangshuHeart => self.step_wudian_recovery(ctx),
            HostBehavior::WudianGuangshuDiamond => self.step_wudian_status(ctx),
            HostBehavior::WudianGuangshuClub => self.step_wudian_status(ctx),
            HostBehavior::WudianGuangshuSpade => self.step_wudian_status(ctx),
            HostBehavior::WudianZiyu => self.step_wudian_recovery(ctx),
            HostBehavior::WudianShouhu => self.step_wudian_recovery(ctx),
            HostBehavior::WudianDuwen => self.step_wudian_status(ctx),
            HostBehavior::WudianPaotai => self.step_wudian_ow_cards(ctx),
            HostBehavior::WudianPaotaiFire => self.step_wudian_damage(ctx),
            HostBehavior::WudianBaoleiSwitch => self.step_wudian_status(ctx),
            HostBehavior::WudianZhenchaDraw => self.step_wudian_recovery(ctx),
            HostBehavior::WudianYuedong => self.step_wudian_recovery(ctx),
            HostBehavior::WudianKuoyin => self.step_wudian_ow_cards(ctx),
            HostBehavior::WudianHuhuan => self.step_wudian_ow_cards(ctx),
            HostBehavior::WudianJidong => self.step_wudian_recovery(ctx),
            HostBehavior::WudianBaoxue => self.step_wudian_targets(ctx),
            HostBehavior::WudianLiangou => self.step_wudian_ow_cards(ctx),
            HostBehavior::WudianXiyang => self.step_wudian_recovery(ctx),
            HostBehavior::WudianFeiren => self.step_wudian_status(ctx),
            HostBehavior::WudianFeidan => self.step_wudian_damage(ctx),
            HostBehavior::WudianZhiyuan => self.step_wudian_recovery(ctx),
            HostBehavior::WudianShenqiang => self.step_wudian_status(ctx),
            HostBehavior::WudianShihuo => self.step_wudian_status(ctx),
            HostBehavior::WudianMengji => self.step_wudian_status(ctx),
            HostBehavior::WudianYinbo => self.step_wudian_ow_cards(ctx),
            HostBehavior::WudianQianggu => self.step_wudian_ow_cards(ctx),
            HostBehavior::WudianQiangguGuard => self.step_wudian_ow_cards(ctx),
            HostBehavior::WudianFeitiao => self.step_wudian_ow_cards(ctx),
            HostBehavior::WudianPingzhang => self.step_wudian_ow_cards(ctx),
            HostBehavior::WudianOwliyong => self.step_wudian_status(ctx),
            HostBehavior::WudianYinshen => self.step_wudian_status(ctx),
            HostBehavior::WudianXiandan => self.step_wudian_status(ctx),
            HostBehavior::WudianJijia => self.step_wudian_status(ctx),
            HostBehavior::XianjianLingxue => self.step_xianjian_status(ctx),
            HostBehavior::XianjianBinxin => self.step_xianjian_status(ctx),
            HostBehavior::XianjianHuimeng => self.step_xianjian_status(ctx),
            HostBehavior::XianjianTianshe => self.step_xianjian_status(ctx),
            HostBehavior::XianjianTianshe2 => self.step_xianjian_recovery(ctx),
            HostBehavior::XianjianFanling => self.step_xianjian_recovery(ctx),
            HostBehavior::WudianLongxiang => self.step_wudian_status(ctx),
            HostBehavior::XianjianGuiyuan => self.step_xianjian_recovery(ctx),
            HostBehavior::XianjianTannang => self.step_xianjian_status(ctx),
            HostBehavior::XianjianFeixu => self.step_xianjian_status(ctx),
            HostBehavior::XianjianFeng => self.step_xianjian_status(ctx),
            HostBehavior::XianjianYa => self.step_xianjian_status(ctx),
            HostBehavior::XianjianSong => self.step_xianjian_status(ctx),
            HostBehavior::XianjianXjyufeng => self.step_xianjian_status(ctx),
            HostBehavior::XianjianXuanyan2 => self.step_xianjian_status(ctx),
            HostBehavior::XianjianXuanyan3 => self.step_xianjian_status(ctx),
            HostBehavior::XianjianNingbin => self.step_xianjian_recovery(ctx),
            HostBehavior::XianjianXuanyan => self.step_xianjian_status(ctx),
            HostBehavior::XianjianDiewu => self.step_xianjian_card_use(ctx),
            HostBehavior::XianjianLingyu => self.step_xianjian_recovery(ctx),
            HostBehavior::WudianLueying => self.step_wudian_status(ctx),
            HostBehavior::XianjianAnwugu => self.step_xianjian_status(ctx),
            HostBehavior::XianjianAnwugu2 => self.step_xianjian_status(ctx),
            HostBehavior::XianjianTxianqu => self.step_xianjian_recovery(ctx),
            HostBehavior::XianjianFenshi => self.step_xianjian_status(ctx),
            HostBehavior::XianjianLonghuo => self.step_xianjian_damage(ctx),
            HostBehavior::XianjianSajin => self.step_xianjian_recovery(ctx),
            HostBehavior::XianjianJtjubao => self.step_xianjian_status(ctx),
            HostBehavior::XianjianYuexing => self.step_xianjian_targets(ctx),
            HostBehavior::XianjianMinsha => self.step_xianjian_damage(ctx),
            HostBehavior::XianjianLinghuo => self.step_xianjian_damage(ctx),
            HostBehavior::XianjianGuijin => self.step_xianjian_cards(ctx),
            HostBehavior::XianjianZhuyue => self.step_xianjian_cards(ctx),
            HostBehavior::XianjianGuanri => self.step_xianjian_damage(ctx),
            HostBehavior::XianjianShuiyun => self.step_xianjian_cards(ctx),
            HostBehavior::XianjianShuiyun5 => self.step_xianjian_recovery(ctx),
            HostBehavior::XianjianWangyou => self.step_xianjian_cards(ctx),
            HostBehavior::XianjianWangyou2 => self.step_xianjian_status(ctx),
            HostBehavior::XianjianLingquan => self.step_xianjian_cards(ctx),
            HostBehavior::XianjianShenwu => self.step_xianjian_recovery(ctx),
            HostBehavior::XianjianChengxin => self.step_xianjian_recovery(ctx),
            HostBehavior::XianjianShenmu => self.step_xianjian_recovery(ctx),
            HostBehavior::XianjianDuci => self.step_xianjian_damage(ctx),
            HostBehavior::XianjianRunxin => self.step_xianjian_recovery(ctx),
            HostBehavior::XianjianXjzhimeng => self.step_xianjian_targets(ctx),
            HostBehavior::XianjianXjzhimeng2 => self.step_xianjian_status(ctx),
            HostBehavior::XianjianSheling => self.step_xianjian_status(ctx),
            HostBehavior::XianjianHushaCount => self.step_xianjian_damage(ctx),
            HostBehavior::XianjianHusha => self.step_xianjian_targets(ctx),
            HostBehavior::XianjianChangnian => self.step_xianjian_targets(ctx),
            HostBehavior::XianjianChangnian2 => self.step_xianjian_status(ctx),
            HostBehavior::XianjianShiying => self.step_xianjian_status(ctx),
            HostBehavior::XianjianZhaoyao => self.step_xianjian_card_use(ctx),
            HostBehavior::XianjianLiguang => self.step_xianjian_cards(ctx),
            HostBehavior::XianjianXuanmo => self.step_xianjian_cards(ctx),
            HostBehavior::XianjianSheying => self.step_xianjian_status(ctx),
            HostBehavior::XianjianHuahu => self.step_xianjian_recovery(ctx),
            HostBehavior::XianjianXtanxi => self.step_xianjian_cards(ctx),
            HostBehavior::XianjianFenglue => self.step_xianjian_targets(ctx),
            HostBehavior::XianjianLiefeng => self.step_xianjian_status(ctx),
            HostBehavior::XianjianTianjian => self.step_xianjian_card_use(ctx),
            HostBehavior::XianjianPoyun => self.step_xianjian_cards(ctx),
            HostBehavior::XianjianXshuangren => self.step_xianjian_status(ctx),
            HostBehavior::XianjianXshuangren2 => self.step_xianjian_status(ctx),
            HostBehavior::XianjianZhangmu => self.step_xianjian_damage(ctx),
            HostBehavior::XianjianQianfang => self.step_xianjian_card_use(ctx),
            HostBehavior::XianjianXunying => self.step_xianjian_status(ctx),
            HostBehavior::XianjianTianxian => self.step_xianjian_status(ctx),
            HostBehavior::XianjianXjyingfeng => self.step_xianjian_status(ctx),
            HostBehavior::XianjianXiepan => self.step_xianjian_cards(ctx),
            HostBehavior::XianjianLuanjian => self.step_xianjian_card_use(ctx),
            HostBehavior::XianjianLuanjian2 => self.step_xianjian_damage(ctx),
            HostBehavior::XianjianCtianfu => self.step_xianjian_cards(ctx),
            HostBehavior::XianjianCtianfu2 => self.step_xianjian_damage(ctx),
            HostBehavior::XianjianCtianfu3 => self.step_xianjian_status(ctx),
            HostBehavior::SwdZhiluxiaohuCard => self.step_swd_card_effects(ctx),
            HostBehavior::SwdMujiarenCard => self.step_swd_card_effects(ctx),
            HostBehavior::SwdGouhunluoCard => self.step_swd_card_effects(ctx),
            HostBehavior::SwdGouhunluoSub => self.step_swd_status(ctx),
            HostBehavior::SwdJiguantongCard => self.step_swd_jiguan_cards(ctx),
            // 机关凤（U05 swd 第二十批，规则 v195）：单体——目标选择打出
            // 一张闪（无闪/取消=未响应）；未响应→目标获得"封印"（全部技
            // 能实例禁用，其下一阶段开始解除[SwdFengyinState]，登记差异：
            // 原生无锁定技区分）+受 1 点伤害；响应则无事。本体入弃堆。
            HostBehavior::SwdJiguanfengCard => self.step_swd_jiguan_cards(ctx),
            // 机关凤封印状态（规则 v195）：拥有者任一阶段开始→全部技能
            // 实例重新启用并移除自身（上游 addTempSkill {player:
            // 'phaseBegin'} 移除同口径）
            HostBehavior::SwdFengyinState => self.step_swd_status(ctx),
            // 机关怨（U05 swd 第二十一批，规则 v196）：单体——使用者选一
            // 张手牌，与本体一起置于目标的武将牌上（离场区），其下一结束
            // 阶段强制获得（SwdJiguanyuanGain）；使用者摸 1。本体不进弃
            // 堆（随置牌走）。
            HostBehavior::SwdJiguanyuanCard => self.step_swd_jiguan_cards(ctx),
            // 机关怨待获得（规则 v196）：拥有者结束阶段——离场区待获得牌
            // 移入手牌并移除自身（上游 jiguanyuan 技 phaseEnd forced 同口径）
            HostBehavior::SwdJiguanyuanGain => self.step_swd_status(ctx),
            // 旋风之刃（灵剑，规则 v198）：目标随机弃 1 张 he；本体入弃堆。
            HostBehavior::SwdXuanfengzhirenCard => self.step_swd_lingjian_cards(ctx),
            // 中行虎甲（灵剑，规则 v198）：目标非 isMin——从牌堆+弃堆发现防
            // 具 Kind 实体装配（登记差异：上游装备定义池凭空造），再随机弃
            // 1 张手牌；本体入弃堆。
            HostBehavior::SwdZhongxinghujiaCard => self.step_swd_lingjian_cards(ctx),
            // 星体改造（灵剑，规则 v198）：使用者（自目标）摸 1 + 手牌上限
            // -1 累加（其下一结束阶段重置，kernel 相位钩）；本体入弃堆。
            HostBehavior::SwdXingtigaizaoCard => self.step_swd_lingjian_cards(ctx),
            // 时间回溯（灵剑，规则 v198）：目标获得其装备区全部牌（装备技
            // 随迁撤销）；本体入弃堆。
            HostBehavior::SwdShijianhuisuCard => self.step_swd_lingjian_cards(ctx),
            // 紧急冷冻（灵剑，规则 v198）：目标≠使用者且未翻面——+2 护甲、
            // 翻面；仍翻面则冻结（不能使用牌也不能成为目标，翻回解除，
            // kernel TurnOvered 钩）；本体入弃堆。
            HostBehavior::SwdJinjilengdongCard => self.step_swd_lingjian_cards(ctx),
            // 圣休号角（灵剑，规则 v198）：目标获嘲讽（相邻角色只能以其为
            // 杀目标；其下一阶段开始解除，kernel 相位钩）；本体入弃堆。
            HostBehavior::SwdShengxiuhaojiaoCard => self.step_swd_lingjian_cards(ctx),
            // 隐秘离场（灵剑，规则 v198）：目标≠使用者——潜行（不能成为牌
            // 的目标；当前回合结束解除，kernel 回合收束钩）；本体入弃堆。
            HostBehavior::SwdYinmilichangCard => self.step_swd_lingjian_cards(ctx),
            // 机关妖术补偿（规则 v198）：非自己回合弃牌含装备 Kind 牌（近
            // 似失去装备区牌，登记差异：替换/获得离去不触发）→ 随机补获
            // 等量灵剑（牌堆+弃堆发现；无则不补，登记差异：上游全定义池
            // 凭空造）。状态不移除（持久防重面不变）。
            HostBehavior::SwdJiguanyaoGain => self.step_swd_status(ctx),
            // 机关术（U05 swd 机关术批，规则 v199/ADR 0780）：自目标——
            // 每张有效装备受控合成：随机附着一张灵剑实体（PlayerState.
            // card_mods 表）。登记差异：①名称合成/数值升级仅入改造表（目
            // 录不可变）；②jiqi/yuchan 池暂缓，全部附着灵剑；③演示呈现暂
            // 缓。本体入弃堆。
            HostBehavior::SwdJiguanshuCard => self.step_swd_jiguan_cards(ctx),
            // 天覆阵（zhenfa，规则 v200/T19a）：AOE 逐大国目标强制弃 1 张
            // he（targets 队列；目标自选牌 min 1 不可取消；self.target=当
            // 前询问目标）。全部结算完后本体入弃堆。
            HostBehavior::SwdTianfuzhenCard => self.step_swd_zhenfa_cards(ctx),
            // 七星阵（zhenfa，规则 v200/T19a）：阶段一 Act——逐围攻者弃 1
            //（SelectCard 询问，self.target=围攻者）；队列空后阶段二——
            // 视为对围攻者用杀（响应闪或 1 伤；登记差异：虚拟杀简化，无
            // 加成/不可无懈额外面），阶段二队列存 self.cards。本体入弃堆。
            HostBehavior::SwdQixingzhenCard => self.step_swd_zhenfa_cards(ctx),
            // 鸟翔阵（zhenfa，规则 v200/T19a）：逐合法目标响应闪或受 1 伤
            //（targets 队列；TargetChoice 询问，取消=伤害）。本体入弃堆。
            HostBehavior::SwdNiaoxiangzhenCard => self.step_swd_zhenfa_cards(ctx),
            // 长蛇阵（zhenfa，规则 v201/T19b）：队列中→全队列（含自己）各
            // 摸 1；否则与距离最近的友方换座成队列（swapSeat）；本体入弃堆。
            HostBehavior::SwdChangshezhenCard => self.step_swd_zhenfa_cards(ctx),
            // 破阵诀（zhenfa，规则 v201/T19b）：n 次随机座位交换（n=存活
            // 数，上游 swapSeat(random, random) ×n 同口径）；本体入弃堆。
            HostBehavior::SwdPozhenjueCard => self.step_swd_zhenfa_cards(ctx),
            // 机关妖术（U05 swd 第二十二批，规则 v197）：单体（range.global
            // 1）——从牌堆+弃堆装备实体中发现一张「目标空装备栏」子类型的
            // 装备，置入其手牌后 emit Equipped 走完整装配（commit_equip 同
            // 栏替换/装备技授予），并授予防重标记（登记差异：①发现池=牌
            // 堆+弃堆可见实体，上游=全部已定义装备牌凭空造牌，无候选则不
            // 装备；②上游 jiguanyaoshu_skill 的补偿触发面——失去装备区牌
            // 时补获 hslingjian 灵剑——依赖灵剑类型池，暂缓登记至机关术
            // 批，本切片仅保留防重面）。本体入弃堆。
            HostBehavior::SwdJiguanyaoshuCard => self.step_swd_jiguan_cards(ctx),
            // 青木：标记四象链 + 表现隐藏（座位交互门控近似=日志；无存档字段）
            HostBehavior::BossQingmu2 => self.step_boss(ctx),
            // 青木·转阶：取消死亡执行 + 换勾芒 + 全场回满 + 两书精 + 挑战者重整
            HostBehavior::BossQingmu2x => self.step_boss(ctx),
            // 青木3：三人隐藏（本体+环邻）
            HostBehavior::BossQingmu3 => self.step_boss(ctx),
            // 青木3·转阶：太昊 + 侧座勾芒/书精 + 挑战者重整
            HostBehavior::BossQingmu3x => self.step_boss(ctx),
            // 白芒 / 赤炎 / 玄麟：同青木 dieBegin hide（D4/B2）
            HostBehavior::BossBaimang2 => self.step_boss(ctx),
            HostBehavior::BossChiyan2 => self.step_boss(ctx),
            HostBehavior::BossXuanlin2 => self.step_boss(ctx),
            // 白芒·转阶：蓐收 + 两明星 + 全场回满 + 挑战者重整
            HostBehavior::BossBaimang2x => self.step_boss(ctx),
            // 赤炎·转阶：祝融 + 两焰灵 + 全场回满 + 挑战者重整
            HostBehavior::BossChiyan2x => self.step_boss(ctx),
            // 玄麟·转阶：共工 + 一玄冥 + 全场回满 + 挑战者重整
            HostBehavior::BossXuanlin2x => self.step_boss(ctx),
            // 白芒3 / 赤炎3 / 玄麟3：同青木3 三人 hide（D4/B3）
            HostBehavior::BossBaimang3 => self.step_boss(ctx),
            HostBehavior::BossChiyan3 => self.step_boss(ctx),
            HostBehavior::BossXuanlin3 => self.step_boss(ctx),
            // 白芒3·转阶：少昊 + 侧座蓐收/明星 + 一明星 fellow
            HostBehavior::BossBaimang3x => self.step_boss(ctx),
            // 赤炎3·转阶：炎帝 + 侧座祝融/焰灵
            HostBehavior::BossChiyan3x => self.step_boss(ctx),
            // 玄麟3·转阶：颛顼 + 共工 fellow + 次座玄冥
            HostBehavior::BossXuanlin3x => self.step_boss(ctx),
            // 后援3/4：死亡前置隐藏（日志近似；不改 stage.kind）
            HostBehavior::BossBianshen3 => self.step_boss(ctx),
            HostBehavior::BossBianshen4 => self.step_boss(ctx),
            // 后援3·转阶：种子化 → 白无常/黑无常
            HostBehavior::BossBianshen3x => self.step_boss(ctx),
            // 后援4·转阶：种子化 → 夜叉/罗刹
            HostBehavior::BossBianshen4x => self.step_boss(ctx),
            HostBehavior::BossHuanren => self.step_boss(ctx),
            HostBehavior::BossHuanrenDie => self.step_boss(ctx),
            HostBehavior::BossQingyi => self.step_boss(ctx),
            HostBehavior::BossChiyi => self.step_boss(ctx),
            HostBehavior::BossChiyiMark => self.step_boss(ctx),
            HostBehavior::BossBaiyi => self.step_boss(ctx),
            HostBehavior::BossBaiyiThunder => self.step_boss(ctx),
            HostBehavior::BossChuanyun | HostBehavior::BossTianyun => self.step_boss(ctx),
            HostBehavior::DddXiaxing
            | HostBehavior::MbcmJiushi
            | HostBehavior::MbcmFangzhu
            | HostBehavior::MbJuejin => self.step_skills_targets(ctx),
            HostBehavior::D3Moyan => self.step_skills_damage(ctx),
            // 机关妖术防重标记：纯状态无触发面（补偿触发暂缓）
            HostBehavior::SwdJiguanyaoState => self.step_swd_status(ctx),
            HostBehavior::WudianDuoqi
            | HostBehavior::WudianYouzhan
            | HostBehavior::WudianChoudu
            | HostBehavior::WudianDiaobingCard => self.step_wudian_cards(ctx),
        }
    }
}
