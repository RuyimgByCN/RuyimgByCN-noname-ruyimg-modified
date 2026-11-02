use super::*;

mod common;
mod swd;
mod xianjian;

impl HostSkillBehavior {
    pub(super) fn filter_behavior(
        &self,
        ctx: &QueryCtx<'_>,
        owner: PlayerId,
        event: &EventData,
    ) -> bool {
        match self.behavior {
            // 厉焰·链杀：持有者以水攻用牌（使用者=持有者；kind 按**结算
            // 种**（use_as 转化声明优先，目录种兜底）——厉焰转化的牌目录
            // 种非水攻）
            // 厉焰·链杀：持有者以水攻用牌（结算种 use_as 优先；语义集
            // 中在 helper）
            HostBehavior::YunchouLuweiyan2 => self.filter_common(ctx, owner, event),
            HostBehavior::YunchouJieyan => self.filter_common(ctx, owner, event),
            HostBehavior::YunchouYiesheng => self.filter_common(ctx, owner, event),
            HostBehavior::WudianDiyjuntun => self.filter_common(ctx, owner, event),
            HostBehavior::WudianJianbi => self.filter_common(ctx, owner, event),
            // 复仇·清算：自己 End 阶段结束（登记差异：上游 global
            // phaseAfter，原生自己回合结束）
            HostBehavior::WudianFuchou2 => self.filter_common(ctx, owner, event),
            HostBehavior::WudianFuchou => self.filter_common(ctx, owner, event),
            HostBehavior::WudianLiangceRemain => self.filter_common(ctx, owner, event),
            HostBehavior::WudianGuihan => self.filter_common(ctx, owner, event),
            HostBehavior::SwdGTiandao => self.filter_swd(ctx, owner, event),
            HostBehavior::SwdShehun => self.filter_swd(ctx, owner, event),
            // 魂身：自己使用/打出的牌为闪
            HostBehavior::SwdHyunshen => self.filter_swd(ctx, owner, event),
            // 魂身·收束：自己准备阶段开始且计数 >0
            HostBehavior::SwdHyunshen2 => self.filter_swd(ctx, owner, event),
            // 待行：自己结束阶段且有 he 牌
            HostBehavior::SwdDaixing => self.filter_swd(ctx, owner, event),
            // 待行·清甲：自己准备阶段开始且计数 >0
            HostBehavior::SwdDaixing2 => self.filter_swd(ctx, owner, event),
            // 待行·吸收：自己受伤后且计数 >0
            HostBehavior::SwdDaixing3 => self.filter_swd(ctx, owner, event),
            HostBehavior::SwdJuxi => self.filter_swd(ctx, owner, event),
            // 聚悉·计数：弃牌者非当前回合玩家
            HostBehavior::SwdJuxiCount => self.filter_swd(ctx, owner, event),
            // 破舟：自己受伤后
            HostBehavior::SwdPozhou => self.filter_swd(ctx, owner, event),
            HostBehavior::SwdPozhou2 => self.filter_swd(ctx, owner, event),
            // 陷阱：主动技
            HostBehavior::SwdSxianjing => self.filter_swd(ctx, owner, event),
            // 陷阱·触发：他人以同花色牌指定你，且该花色在陷阱表中
            HostBehavior::SwdSxianjingGain => self.filter_swd(ctx, owner, event),
            // 陷阱·取回：自己受伤后且陷阱表非空
            HostBehavior::SwdSxianjingDamage => self.filter_swd(ctx, owner, event),
            HostBehavior::SwdGuxing => self.filter_swd(ctx, owner, event),
            // 灵矢：锁定旗标（无触发；四臂内联）
            HostBehavior::SwdLingshi => self.filter_swd(ctx, owner, event),
            HostBehavior::SwdJinlin => self.filter_swd(ctx, owner, event),
            // 锦鳞·退：持有者阶段开始
            HostBehavior::SwdJinlin2 => self.filter_swd(ctx, owner, event),
            // 禳星：自己受到未防止伤害 / 自己失去体力
            HostBehavior::HlssXiangxing => self.filter_common(ctx, owner, event),
            // 隐月：自己受到未防止伤害（前置窗）
            HostBehavior::HlssYueyin => self.filter_common(ctx, owner, event),
            HostBehavior::HlssFengqi => self.filter_common(ctx, owner, event),
            HostBehavior::HlssFengqi2 => self.filter_common(ctx, owner, event),
            HostBehavior::HlssGaiming => self.filter_common(ctx, owner, event),
            // 潜龙：开局首阶段（原生 phase_number 在触发前已 +1，故 ==1
            // 等价上游 game.phaseNumber == 0）/ 自己获得牌 / 自己受伤 /
            // 自己造成伤害（点数未满 99 才计）
            HostBehavior::MbQianlong => self.filter_common(ctx, owner, event),
            // 涅槃：自己濒死且未发动
            HostBehavior::Niepan => self.filter_common(ctx, owner, event),
            HostBehavior::Yuhuo => self.filter_common(ctx, owner, event),
            HostBehavior::Shenwei => self.filter_common(ctx, owner, event),
            // 集控：自己阶段开始（任意阶段）/ 自己失去手牌且手牌已空
            HostBehavior::Hxunzhi => self.filter_common(ctx, owner, event),
            HostBehavior::Hxunzhi2 => self.filter_common(ctx, owner, event),
            HostBehavior::Gxianyin => self.filter_common(ctx, owner, event),
            HostBehavior::Jilve => self.filter_common(ctx, owner, event),
            HostBehavior::Gongxin => self.filter_common(ctx, owner, event),
            // 每步：他人出牌阶段开始，自己有牌，且其在攻击范围内能及自己
            HostBehavior::NewMeibu => self.filter_common(ctx, owner, event),
            HostBehavior::NewMeibuRange => self.filter_common(ctx, owner, event),
            HostBehavior::SbZhaXiang => self.filter_common(ctx, owner, event),
            // 反击：自己为伤害承受者、手牌有【杀】、伤害载体为【杀】或【决斗】
            HostBehavior::YxsFanji => self.filter_common(ctx, owner, event),
            HostBehavior::YxsFanji2 => self.filter_common(ctx, owner, event),
            // 门神：自己结束阶段且存活角色 >1
            HostBehavior::YxsMenshen => self.filter_common(ctx, owner, event),
            // 门神·清除：任意阶段开始 / 任意死亡，且场上存在门神标记
            HostBehavior::YxsMenshen3 => self.filter_common(ctx, owner, event),
            HostBehavior::YxsMenshen2 => self.filter_common(ctx, owner, event),
            HostBehavior::XiguaMopaiFanbei => self.filter_common(ctx, owner, event),
            HostBehavior::Kurou => self.filter_common(ctx, owner, event),
            HostBehavior::FzLiegong => self.filter_common(ctx, owner, event),
            // 狂骨：自己为伤害来源、距离 ≤1 且伤害 >0
            HostBehavior::FzXinKuanggu => self.filter_common(ctx, owner, event),
            // 主亡忠继：死亡者为主公且自己为忠臣
            HostBehavior::ZhuWanzhongJi => self.filter_common(ctx, owner, event),
            // 血量翻倍：自己首次阶段开始（标记未置位）
            HostBehavior::XiguaXueliangFanbei => self.filter_common(ctx, owner, event),
            // 灾变：自己的出牌阶段开始
            HostBehavior::Zaibian => self.filter_common(ctx, owner, event),
            // 誓师：自己为击杀者
            HostBehavior::Shishi => self.filter_common(ctx, owner, event),
            // 循梦：自己为伤害来源、伤害载体为【杀】且非铁索传导
            HostBehavior::Xunmeng => self.filter_common(ctx, owner, event),
            HostBehavior::Jishe => self.filter_common(ctx, owner, event),
            // 止息：自己使用牌
            HostBehavior::NewZhixi => self.filter_common(ctx, owner, event),
            // 撷芳：死亡事件（任意角色）或**自己的**摸牌阶段结束
            HostBehavior::Xiefang => self.filter_common(ctx, owner, event),
            // 让节：自己为该伤害承受者且未防止
            HostBehavior::Rangjie => self.filter_common(ctx, owner, event),
            HostBehavior::Reguhuo => self.filter_common(ctx, owner, event),
            // 魔化：自己进入濒死
            HostBehavior::Mohua2 => self.filter_common(ctx, owner, event),
            // 修罗：自己准备阶段开始、判定区有牌且手牌非空
            HostBehavior::Xiuluo => self.filter_common(ctx, owner, event),
            // 灭道·弃：自己弃牌阶段开始且体力未满
            HostBehavior::Miedao => self.filter_common(ctx, owner, event),
            HostBehavior::Moyan => self.filter_common(ctx, owner, event),
            // 疾风：自己使用【杀】或单目标锦囊且目标数 >1、未取消
            HostBehavior::Jifeng => self.filter_common(ctx, owner, event),
            // 昆仑镜：自己的阶段收束（快照）或阶段开始（体力低于快照值）
            HostBehavior::Kunlunjing => self.filter_common(ctx, owner, event),
            HostBehavior::Shengong => self.filter_common(ctx, owner, event),
            // 轩辕剑·伤增：自己是伤害来源且未横置伤害
            HostBehavior::XuanyuanjianDamage => self.filter_common(ctx, owner, event),
            // 轩辕剑·反噬：自己是伤害来源且本次加伤生效（标记位）
            HostBehavior::XuanyuanjianBacklash => self.filter_common(ctx, owner, event),
            // 轩辕：自己结束阶段且有黑桃牌且弃牌堆有【轩辕剑】
            HostBehavior::Xuanyuan => self.filter_common(ctx, owner, event),
            // 燎原：自己是【杀】的使用者且持有与该杀同花色的牌
            HostBehavior::Liaoyuan => self.filter_common(ctx, owner, event),
            // 改造：自己装备牌入槽前，且该牌固有栏位已被自己占用（有空格）
            HostBehavior::Gaizao => self.filter_common(ctx, owner, event),
            HostBehavior::SwdTianshu => self.filter_swd(ctx, owner, event),
            // 李香龙：自己是该杀的使用者且闪避者存活
            HostBehavior::Lxianglong => self.filter_common(ctx, owner, event),
            // 移花：他人使用的**单目标**牌以自己为目标、且自己手牌 ≥2
            HostBehavior::Yihua => self.filter_common(ctx, owner, event),
            // 贞烈：他人使用【杀】或锦囊牌指定自己为目标且未取消
            HostBehavior::Zhenlie => self.filter_common(ctx, owner, event),
            HostBehavior::Tianlun => self.filter_common(ctx, owner, event),
            // 脉轮：自己的任意阶段开始
            HostBehavior::Mailun => self.filter_common(ctx, owner, event),
            // 冰封·封：自己已翻回正面（移除自身）
            HostBehavior::Bingfeng2 => self.filter_common(ctx, owner, event),
            HostBehavior::Bingfeng => self.filter_common(ctx, owner, event),
            // 此桥邪：自己失去的牌中含装备区来源
            HostBehavior::Cyqiaoxie => self.filter_common(ctx, owner, event),
            // 护天：自己阶段结束、手牌非空且当前无寄存
            HostBehavior::Hutian => self.filter_common(ctx, owner, event),
            // 天香：自己受到未防止伤害且手中有红桃牌
            HostBehavior::Tianxiang => self.filter_common(ctx, owner, event),
            // 天香·受：自己是该伤害的承受者
            HostBehavior::Tianxiang2 => self.filter_common(ctx, owner, event),
            HostBehavior::Songci => self.filter_common(ctx, owner, event),
            // 诛害：他人结束阶段开始、其本回合造成过伤害、自己有杀且在射程内
            HostBehavior::SwdZhuhai => self.filter_swd(ctx, owner, event),
            HostBehavior::SwdLinyun => self.filter_swd(ctx, owner, event),
            HostBehavior::SwdJikong => self.filter_swd(ctx, owner, event),
            // 魅惑：自己失去装备牌（来源区=装备）且装备区已空
            HostBehavior::SwdMeihuo => self.filter_swd(ctx, owner, event),
            // 守印：任意角色濒死、自己未发动且未翻面
            HostBehavior::SwdShouyin => self.filter_swd(ctx, owner, event),
            // 义山：自己受伤且有「仍在他处」的已失去牌
            HostBehavior::SwdYishan => self.filter_swd(ctx, owner, event),
            // 入梦：他人出牌阶段开始且自己手/装备中有非基本牌
            HostBehavior::SwdRumeng => self.filter_swd(ctx, owner, event),
            // 神焰：前置面=自己造成的未防止火伤且未发动；结算面=已发动且
            // 有待蔓延目标（同一伤害事件的两面按状态区分）
            HostBehavior::SwdShenyan => self.filter_swd(ctx, owner, event),
            HostBehavior::SwdLingwu => self.filter_swd(ctx, owner, event),
            // 清正：自己出牌阶段开始且手牌非空
            HostBehavior::MbcmQingzheng => self.filter_common(ctx, owner, event),
            // 卫统：开局首阶段 + 自己持潜龙 + 另有魏势力持本技者
            HostBehavior::MbWeitong => self.filter_common(ctx, owner, event),
            // 隐月·判：自己未被翻面时取消翻面（上游 filter !isTurnedOver）
            HostBehavior::HlssYueyin2 => self.filter_common(ctx, owner, event),
            // 孤行·计数：孤行杀窗口内自己造成伤害
            HostBehavior::SwdGuxing3 => self.filter_swd(ctx, owner, event),
            HostBehavior::SwdNlianji => self.filter_swd(ctx, owner, event),
            // 热血：阶段玩家不是自己，且在杀范围内，自己手牌有杀
            HostBehavior::SwdRexue => self.filter_swd(ctx, owner, event),
            // 幻音：他人对你使用牌且未取消
            HostBehavior::SwdHuanyin => self.filter_swd(ctx, owner, event),
            // 灵弦：自己使用/打出牌，且非自己回合，且存在攻击距离 >1 的其他存活者
            HostBehavior::SwdLingxian => self.filter_swd(ctx, owner, event),
            // 铁壁：杀指定了与持有者距离 ≤1 的目标，持有者有黑牌
            HostBehavior::SwdTiebi => self.filter_swd(ctx, owner, event),
            // 大同：任意角色任意阶段结束（global phaseEnd）全场极差 ≤1
            HostBehavior::SwdDatong => self.filter_swd(ctx, owner, event),
            // 承剑：其他角色受到杀伤害（来源≠持有者）
            HostBehavior::SwdChengjian => self.filter_swd(ctx, owner, event),
            // 缚斧：持有者对目标用杀伤害前置（有武器）
            HostBehavior::SwdJufu => self.filter_swd(ctx, owner, event),
            // 花镜：持有者造成的锦囊牌伤害
            HostBehavior::SwdHuajing => self.filter_swd(ctx, owner, event),
            // 护盾：持有者造成伤害后自己无护甲
            HostBehavior::SwdHudun => self.filter_swd(ctx, owner, event),
            // 破星：持有者对体力高于自己的目标造成伤害
            HostBehavior::SwdPoxing => self.filter_swd(ctx, owner, event),
            // 流虹：持有者使用杀
            HostBehavior::SwdLiuhong => self.filter_swd(ctx, owner, event),
            // 墨剑：持有者使用杀且指定了目标（上游 shaBegin=自己用杀）
            HostBehavior::SwdMojian => self.filter_swd(ctx, owner, event),
            // 道法：其他角色受伤且来源有牌
            HostBehavior::SwdDaofa => self.filter_swd(ctx, owner, event),
            // 玉鳞：持有者即将受伤且手/装备区有装备牌
            HostBehavior::SwdYulin => self.filter_swd(ctx, owner, event),
            // 平恤：无触发面
            HostBehavior::SwdPingxu => self.filter_swd(ctx, owner, event),
            // 还灵：自己任一阶段结束（上游 player phaseEnd 逐阶段均触发）
            HostBehavior::SwdHuanling => self.filter_swd(ctx, owner, event),
            // 屠阵：自己伤害他人结算后且其有非基本手牌
            HostBehavior::SwdTuzhen => self.filter_swd(ctx, owner, event),
            // 协律：自己弃牌阶段结束（End 入场）且手牌≥2 同色
            HostBehavior::SwdXielv => self.filter_swd(ctx, owner, event),
            // 雪煌：无触发面
            HostBehavior::SwdXuehuang => self.filter_swd(ctx, owner, event),
            // 还侠：无触发面
            HostBehavior::SwdHuanxia => self.filter_swd(ctx, owner, event),
            // 破月：无触发面
            HostBehavior::SwdPoyue => self.filter_swd(ctx, owner, event),
            // 女娲石：濒死者 hp≤0 且自己 hp>1（上游 global dying 同口径；
            // 含自身濒死时 owner hp>1 恒否自然排除）
            HostBehavior::SwdNvwashiGuard => self.filter_swd(ctx, owner, event),
            // 收纳/炼化/神农鼎：无触发面
            HostBehavior::SwdLianyaohuShouna => self.filter_swd(ctx, owner, event),
            HostBehavior::SwdLianyaohuLianhua => self.filter_swd(ctx, owner, event),
            HostBehavior::SwdShennongdingCard => self.filter_swd(ctx, owner, event),
            // 昆仑镜：无触发面
            HostBehavior::SwdKunlunjingcCard => self.filter_swd(ctx, owner, event),
            // 鬼蛇旗/仙露回：无触发面（CardUsed 内建直派）
            HostBehavior::GuisheqiCard => self.filter_common(ctx, owner, event),
            HostBehavior::XianluhuiCard => self.filter_common(ctx, owner, event),
            // 手持门控 helper（四象五件共用）：持有者手牌含对应器族实体
            // 青龙之圭：自己准备阶段且手持
            HostBehavior::SwdQinglongzhiguiHeld => self.filter_swd(ctx, owner, event),
            // 逆攻积攒：自己受到真实伤害
            HostBehavior::SwdNigongAccumulate => self.filter_swd(ctx, owner, event),
            // 逆攻发动：无触发面
            HostBehavior::SwdNigongStrike => self.filter_swd(ctx, owner, event),
            // 四非珍面：无触发面
            HostBehavior::SwdSifeizhenmianCard => self.filter_swd(ctx, owner, event),
            // 朱雀之璋：受他人伤害结算后且手持（来源存活且非自己）
            HostBehavior::SwdZhuquezhizhangHeld => self.filter_swd(ctx, owner, event),
            // 白兽之琥：自己弃牌阶段结束且手持且有红牌
            HostBehavior::SwdBaishouzhihuHeld => self.filter_swd(ctx, owner, event),
            // 玄武之璜：自己造成伤害结算后且手持且受伤
            HostBehavior::SwdXuanwuzhihuangHeld => self.filter_swd(ctx, owner, event),
            // 玉蝉换形：自己用牌（非玉蝉本体）结算后且手牌有玉蝉
            HostBehavior::SwdYuchanSwap => self.filter_swd(ctx, owner, event),
            // 疾影：无触发面
            HostBehavior::SwdJiyingHeld => self.filter_swd(ctx, owner, event),
            // 沐风：结束阶段且本回合用过基本牌
            HostBehavior::SwdMufengHeld => self.filter_swd(ctx, owner, event),
            // 望尘：弃牌阶段结束（End 入场）且弃置过基本牌
            HostBehavior::SwdWangchenHeld => self.filter_swd(ctx, owner, event),
            // 神草结：锦囊伤害且自己是受伤者或来源、手持神草结
            HostBehavior::SwdShencaojieGuard => self.filter_swd(ctx, owner, event),
            // 黄麟之琮：自己准备阶段无护甲且手持
            HostBehavior::SwdHuanglinzhicongHeld => self.filter_swd(ctx, owner, event),
            // 空洞印：自己死亡（maxHp>0 gate 恒真登记）
            HostBehavior::SwdKongdongyinGuard => self.filter_swd(ctx, owner, event),
            // 神秘果窗口：自己出牌阶段用出基本/普通锦囊结算后、手持神秘
            // 果、每阶段限一次、目标全存活。登记差异：view-as 转化用牌、
            // 五谷（Remained 面未实现）、机关异类与延时锦囊不进再结算
            // 白名单（上游 complexTarget/exclusion 同口径收窄）。
            HostBehavior::SwdShenmiguoWindow => self.filter_swd(ctx, owner, event),
            // 浴尘：回合外使用/打出黑色牌结算后（双事件；he 有牌的目标
            // 在机器步求取，filter 只验事件形状）
            HostBehavior::SwdYuchen => self.filter_swd(ctx, owner, event),
            // 挟雷：使用/打出杀结算后（双事件；>2 人局；目标候选在机器
            // 步求取——use 排除原目标、respond 排除来源）
            HostBehavior::SwdXielei => self.filter_swd(ctx, owner, event),
            // 连打：自己杀结算完毕、每回合未链过、he 有牌、目标存活
            HostBehavior::SwdLianda => self.filter_swd(ctx, owner, event),
            // 隳延：自己造成伤害、对方有装备
            HostBehavior::SwdSuiyan => self.filter_swd(ctx, owner, event),
            // 断行：自己装备牌入装（"未强化"面登记差异：原生无强化语义）
            HostBehavior::SwdDuanxing => self.filter_swd(ctx, owner, event),
            // 破浪：自己造成伤害、对方有装备
            HostBehavior::SwdPolang => self.filter_swd(ctx, owner, event),
            // 诱引：其他角色弃置过含非基本牌且自己手牌<5（地/机关异类
            // 按非基本近似——登记差异同 is_basic_kind 保守集）
            HostBehavior::SwdYouyin => self.filter_swd(ctx, owner, event),
            // 暗莲映：弃牌结算后自己手牌为空（含自己弃牌——上游 loseEnd
            // 手牌丢失面≈弃置面登记差异）→ frequent 摸 2
            HostBehavior::SwdAnlianying => self.filter_swd(ctx, owner, event),
            // 乐跃：任意角色回复结算后（候选面在机器步分流）
            HostBehavior::SwdYinyue => self.filter_swd(ctx, owner, event),
            // 灵心：自己阶段结束帧（End/收束入场——PhaseEnd 语义由 filter
            // 验 begin=false 且 phase==owner 当前阶段末）
            HostBehavior::SwdLingxin => self.filter_swd(ctx, owner, event),
            // 虚影：自己受到伤害前置（DamageBefore 帧共用 Damage 载荷——
            // 取消+失去体力在机器步写回）
            HostBehavior::SwdXuying => self.filter_swd(ctx, owner, event),
            // 乖戾：自己杀伤害（非铁索传导）——Before 面 +1 / After 面弃两张
            HostBehavior::SwdGuaili => self.filter_swd(ctx, owner, event),
            // 敷衍·护甲面：自己受伤结算后（无护甲目标候选在机器步求取）
            HostBehavior::SwdFuyan => self.filter_swd(ctx, owner, event),
            // 凌波·恒爱面：闪的响应/使用结算（双事件；frequent 摸 2）
            HostBehavior::SwdHlingbo => self.filter_swd(ctx, owner, event),
            // 血裔·Miles 面：自己受到伤害前置
            HostBehavior::SwdMilesXueyi => self.filter_swd(ctx, owner, event),
            // 小莫雨·山小小面：自己造成伤害结算后（每回合一次经 inst 检查）
            HostBehavior::SwdXiaomoyu => self.filter_swd(ctx, owner, event),
            // 破茧·齐面：自己弃牌后手牌为空（上游 loseEnd≈弃置面登记差异）
            HostBehavior::SwdPojian => self.filter_swd(ctx, owner, event),
            // 运筹·姬亮面：出牌主动技（ActivateSkill 通道），无触发面
            HostBehavior::SwdYunchou => self.filter_swd(ctx, owner, event),
            // 破晓·张烈面：锁定 marker（三查询点），无触发面
            HostBehavior::SwdPoxiao => self.filter_swd(ctx, owner, event),
            // 疾风·司空宇面：锁定 marker（engine 杀距离校验臂查询），无触发面
            HostBehavior::SwdSliufeng => self.filter_swd(ctx, owner, event),
            // 乱击·多鹏面：出牌主动技（activate 通道），无触发 filter
            HostBehavior::SwdLuanji => self.filter_swd(ctx, owner, event),
            // 聒噪·疾鹏面：距离 1 以内角色受损（来源非自己；距离与手牌
            // 候选在机器步求取；受损者含自己——上游 filter 只排除来源）
            HostBehavior::SwdGuozao => self.filter_swd(ctx, owner, event),
            // 英姿·刷新面：自己摸牌阶段开始（额外摸 1；手牌上限查询在同位）
            HostBehavior::SwdReyingzi => self.filter_swd(ctx, owner, event),
            // 愚钝·福面：锁定 marker（engine PlayCard 禁锦囊臂+双卡 viewAs
            // 臂查询），无触发 filter
            HostBehavior::SwdYudun => self.filter_swd(ctx, owner, event),
            // 占星·肯迪面：出牌主动技（activate 通道），无触发 filter
            HostBehavior::SwdZhanxing => self.filter_swd(ctx, owner, event),
            // 乐学：任一角色回合开始（授技在机器步按当前回合角色分流：
            // 自己回合=回合内池，他人回合=回合外池；上游 lexue1/lexue2
            // 分别挂 phaseBegin/phaseEnd 的双池面收敛为回合开始单窗，
            // PhaseBegin 授至本回合结束由 expires_end_of_turn 承担）
            HostBehavior::SwdLexue => self.filter_swd(ctx, owner, event),
            // 枕流触发面：自己准备阶段开始且 off_court 有牌
            HostBehavior::SwdZhenjiu2 => self.filter_swd(ctx, owner, event),
            // 麻醉触发面：自己造成伤害前置（source 面——上游
            // trigger: {source: "damageBegin"} 同口径）且 off_court 有牌
            HostBehavior::SwdLmazui2 => self.filter_swd(ctx, owner, event),
            // 圣手·淳于恒面：锁定 marker（engine viewAs 链查询），无触发
            HostBehavior::SwdShengshou => self.filter_swd(ctx, owner, event),
            // 活弹·舒莞儿面：出牌主动技（activate 通道），无触发 filter
            HostBehavior::SwdHuodan => self.filter_swd(ctx, owner, event),
            // 义绝：出牌主动技（activate 通道），无触发 filter
            HostBehavior::SwdYijue => self.filter_swd(ctx, owner, event),
            // 荡平：自己造成伤害（真实、有受害存活）
            HostBehavior::SwdDangping => self.filter_swd(ctx, owner, event),
            // 幻形：自己准备阶段开始（有可弃牌与男性其他存活）
            HostBehavior::SwdHuanxing => self.filter_swd(ctx, owner, event),
            // 命符·姜子牙面：出牌主动技（activate 通道），无触发 filter
            HostBehavior::SwdMingfu => self.filter_swd(ctx, owner, event),
            // 玄影·端蒙面：父技主动面（无受伤 filter）
            HostBehavior::SwdXuanying => self.filter_swd(ctx, owner, event),
            // 玄影伤害子技：自己受伤（真实）、已横置、来源存活
            HostBehavior::SwdXuanyingDamage => self.filter_swd(ctx, owner, event),
            // 极谋·姬良面：自己受伤（真实）且回合非自己
            HostBehavior::SwdJqimou => self.filter_swd(ctx, owner, event),
            // 剑击·少微面：出牌主动技（activate 通道），无触发 filter
            HostBehavior::SwdJianji => self.filter_swd(ctx, owner, event),
            // 头弹·麦星面：出牌主动技（activate 通道），无触发 filter
            HostBehavior::SwdToudan => self.filter_swd(ctx, owner, event),
            // 真威：其他角色响应结算（响应者非自己、自己回合内、牌已入弃堆）
            HostBehavior::SwdFzhenwei => self.filter_swd(ctx, owner, event),
            // 遁行：成为其他角色卡牌目标（he 有牌可弃）
            HostBehavior::SwdDunxing => self.filter_swd(ctx, owner, event),
            // 黑火：出牌主动技（activate 通道），无触发 filter
            HostBehavior::SwdHeihuo => self.filter_swd(ctx, owner, event),
            // 凝弦：自己受伤结算后（he 有黑色牌）
            HostBehavior::SwdNingxian => self.filter_swd(ctx, owner, event),
            // 孵鸟/妖童：出牌主动技/marker（activate/viewAs 通道），无触发 filter
            HostBehavior::SwdFuniao => self.filter_swd(ctx, owner, event),
            HostBehavior::SwdYaotong => self.filter_swd(ctx, owner, event),
            // 破阵：自己受伤结算后（有来源、双方存活、手牌数不同）
            HostBehavior::SwdPozhen => self.filter_swd(ctx, owner, event),
            // 鬼眼·复活面：自己濒死且存活
            HostBehavior::SwdGuiyan2 => self.filter_swd(ctx, owner, event),
            // 狩猎：Sha 机器内联（无触发 filter）
            HostBehavior::SwdShoulie => self.filter_swd(ctx, owner, event),
            // 不死：自己濒死且存活
            HostBehavior::SwdBusi => self.filter_swd(ctx, owner, event),
            // 唤魂：任意角色濒死（hp≤0 面）且自己 he 有牌
            HostBehavior::SwdHuanhun => self.filter_swd(ctx, owner, event),
            // 倾城：自己结束阶段（End/收束帧）
            HostBehavior::SwdLqingcheng => self.filter_swd(ctx, owner, event),
            // 因果：他人受到有来源伤害前置（来源≠自己、自己 he 有牌）
            HostBehavior::SwdYinguo => self.filter_swd(ctx, owner, event),
            // 拷贝栏/奥剑：内联/主动通道，无触发 filter
            HostBehavior::SwdKbolan => self.filter_swd(ctx, owner, event),
            HostBehavior::SwdAojian => self.filter_swd(ctx, owner, event),
            // 化金：出牌主动技，无触发 filter
            HostBehavior::SwdHuajin => self.filter_swd(ctx, owner, event),
            // 化金触发面：自己造成伤害前置，或自己准备阶段开始（自清）
            HostBehavior::SwdHuajin2 => self.filter_swd(ctx, owner, event),
            // 明镜：自己受伤前置 + 无防具（1/3 在机器步掷；登记差异：上游
            // 在 filter 掷点，原生 filter 只读故效果步掷——同净效）
            HostBehavior::SwdMinjing => self.filter_swd(ctx, owner, event),
            // 兰芷：自己使用的牌花色为梅花（未取消）
            HostBehavior::SwdLanzhi => self.filter_swd(ctx, owner, event),
            // 烈刃：自己以杀造成真实伤害且受害者存活有牌
            HostBehavior::SwdLieren => self.filter_swd(ctx, owner, event),
            // 仙匠·车芸面：自己使用牌且唯一目标为他人、目标装备区有
            // 自己未持有（按目录牌种）的牌
            HostBehavior::SwdCyxianjiang => self.filter_swd(ctx, owner, event),
            // 幻形·移除面：自己受伤结算后（在位旗标=状态技能实例）
            HostBehavior::SwdHuanxing2 => self.filter_swd(ctx, owner, event),
            // 激昂·凤天凌面：①自己使用红杀/决斗（指定唯一目标）②成为
            // 红杀/决斗的目标（他人使用）——上游双面并集；登记差异：
            // 目标面按 CardUsed+targets 表达（useCardToTargeted 等价），
            // 来源面按 targets.len()==1 收窄
            HostBehavior::SwdJiang => self.filter_swd(ctx, owner, event),
            // 横刀·慧彦面：回合外用牌/响应结算（当前回合角色 he 有牌）
            HostBehavior::SwdHwendao => self.filter_swd(ctx, owner, event),
            // 凌锋：自己出牌阶段结束（出牌数门在机器步求取）
            HostBehavior::SwdLingfeng => self.filter_swd(ctx, owner, event),
            // 冰箭/神鼎/鬼眼：出牌主动技（ActivateSkill 通道），无触发面
            // 袭西·凤钰面：自己出牌阶段开始且 he 有牌（候选面机器步求取）
            HostBehavior::SwdShangxi => self.filter_swd(ctx, owner, event),
            // 花间·剑仙面：自己出牌阶段结束且 he 有牌
            HostBehavior::SwdHuajian => self.filter_swd(ctx, owner, event),
            // 观虎：自己杀伤害结算后、对方有牌（手/装备任一）
            HostBehavior::SwdGuanhu => self.filter_swd(ctx, owner, event),
            // 苏生：任意角色死亡前置、自己有手牌（每回合一次走 limits 在
            // OptionalGate 消费；目标体力重置在机器步写回）
            HostBehavior::SwdSusheng => self.filter_swd(ctx, owner, event),
            // 挥琪：自己受伤结算后（hp<maxHp 门在 filter——X=损失值）
            HostBehavior::SwdHuiqi => self.filter_swd(ctx, owner, event),
            // 东皇钟：自己任一阶段结束且手牌有红牌（蝶舞口径）
            HostBehavior::SwdDonghuangzhongCard => self.filter_swd(ctx, owner, event),
            // 壶境：自己准备阶段入场且弃牌堆有炼妖壶
            HostBehavior::SwdHujing => self.filter_swd(ctx, owner, event),
            // 朱羽：横置角色受到非传导伤害且持有者有红牌（手/装备区）
            HostBehavior::SwdZhuyu => self.filter_swd(ctx, owner, event),
            // 落梅：持有者使用或打出梅花牌
            HostBehavior::SwdLuomei => self.filter_swd(ctx, owner, event),
            // 凝霜：成为黑色牌的目标（使用者≠持有者、未「横置且翻面」、
            // 持有者手/装备区有黑牌）
            HostBehavior::SwdNingshuang => self.filter_swd(ctx, owner, event),
            // 工神：其他角色使用基本/锦囊牌且有目标（持有者手/装备区有
            // 装备牌；地/机关等异类种不计）
            HostBehavior::SwdGongshen => self.filter_swd(ctx, owner, event),
            // 悲歌：伤害由杀造成、受伤者在场、持有者有牌
            HostBehavior::YunchouBeige => self.filter_common(ctx, owner, event),
            HostBehavior::WudianLiduan => self.filter_common(ctx, owner, event),
            // 蝉缘·变体：有来源的死亡
            HostBehavior::YunchouDiychanyuan => self.filter_common(ctx, owner, event),
            // 蛊惑·变体：自己回合开始且区域有牌
            HostBehavior::YunchouDiyguhuo => self.filter_common(ctx, owner, event),
            // 狡黠：成为红色牌的目标（使用者不限）
            HostBehavior::YunchouJiaoxia => self.filter_common(ctx, owner, event),
            // 红莲：受到来源为其他角色的伤害且来源有红牌
            HostBehavior::YunchouHonglian => self.filter_common(ctx, owner, event),
            // 纵火：自己造成的非火伤害（前置窗）
            HostBehavior::YunchouZonghuo => self.filter_common(ctx, owner, event),
            // 烧营：自己造成的火伤害
            HostBehavior::YunchouShaoying => self.filter_common(ctx, owner, event),
            // 雄姿：自己摸牌阶段开始（固定摸 2 后补摸）
            HostBehavior::YunchouXiongzi => self.filter_common(ctx, owner, event),
            // 雅量：自己造成或受到伤害，双方存活且异人
            HostBehavior::YunchouYaliang => self.filter_common(ctx, owner, event),
            // 戏才：自己受到有牌伤害（可选确认门走 trigger_meta optional）
            HostBehavior::YunchouXicai => self.filter_common(ctx, owner, event),
            // 狂骨·变体：自己造成伤害；满血时要求目标攻击距离 >1（受伤
            // 恒真——回复分支在效果内自检）
            HostBehavior::YunchouDiykuanggu => self.filter_common(ctx, owner, event),
            HostBehavior::ShaDamageForceDiscard => self.filter_common(ctx, owner, event),
            HostBehavior::PhaseEndRecoverOrDraw => self.filter_common(ctx, owner, event),
            HostBehavior::TargetDiscardEquipOrDamage => self.filter_common(ctx, owner, event),
            HostBehavior::ShengkeCheck => self.filter_common(ctx, owner, event),
            HostBehavior::ChangeCardNature => self.filter_common(ctx, owner, event),
            HostBehavior::WangmeiZhikeCard => self.filter_common(ctx, owner, event),
            HostBehavior::ShuigongCard => self.filter_common(ctx, owner, event),
            HostBehavior::FudichouxinCompare => self.filter_common(ctx, owner, event),
            HostBehavior::ChenhuodajieCard => self.filter_common(ctx, owner, event),
            HostBehavior::TouliangSwap => self.filter_common(ctx, owner, event),
            HostBehavior::LinghunzhihuoCard => self.filter_common(ctx, owner, event),
            HostBehavior::ZhaomingdanCard => self.filter_common(ctx, owner, event),
            HostBehavior::JihuocardDraw => self.filter_common(ctx, owner, event),
            HostBehavior::GeanguanhuoCompare => self.filter_common(ctx, owner, event),
            HostBehavior::CaochuanCard => self.filter_common(ctx, owner, event),
            HostBehavior::ShezhanqunruCompare => self.filter_common(ctx, owner, event),
            HostBehavior::GujianShujinsanCard => self.filter_common(ctx, owner, event),
            HostBehavior::GujianMianliCard => self.filter_common(ctx, owner, event),
            HostBehavior::GujianFeibiaoCard => self.filter_common(ctx, owner, event),
            HostBehavior::JinlianzhuCancel => self.filter_common(ctx, owner, event),
            HostBehavior::MoshouImmunity => self.filter_common(ctx, owner, event),
            HostBehavior::SanjianWeaponDamage => self.filter_common(ctx, owner, event),
            HostBehavior::SijiDiscardDraw => self.filter_common(ctx, owner, event),
            HostBehavior::MaxRangeWeaponDamage => self.filter_common(ctx, owner, event),
            HostBehavior::XianjianXiepan => self.filter_xianjian(ctx, owner, event),
            HostBehavior::XianjianXjyingfeng => self.filter_xianjian(ctx, owner, event),
            HostBehavior::XianjianXjzhimeng2 => self.filter_xianjian(ctx, owner, event),
            HostBehavior::XianjianSheling => self.filter_xianjian(ctx, owner, event),
            HostBehavior::WudianZhenchaDraw => self.filter_common(ctx, owner, event),
            HostBehavior::WudianZiyu => self.filter_common(ctx, owner, event),
            HostBehavior::XianjianFanling => self.filter_xianjian(ctx, owner, event),
            HostBehavior::XianjianNingbin => self.filter_xianjian(ctx, owner, event),
            HostBehavior::XianjianLingyu => self.filter_xianjian(ctx, owner, event),
            HostBehavior::XianjianTxianqu => self.filter_xianjian(ctx, owner, event),
            HostBehavior::XianjianCtianfu3 => self.filter_xianjian(ctx, owner, event),
            HostBehavior::WudianGuangshuSpade => self.filter_common(ctx, owner, event),
            HostBehavior::GujianFoodGudonggeng => self.filter_common(ctx, owner, event),
            HostBehavior::GujianFoodXiajiao => self.filter_common(ctx, owner, event),
            HostBehavior::GujianFoodLiyutang => self.filter_common(ctx, owner, event),
            HostBehavior::GujianFoodMapodoufu => self.filter_common(ctx, owner, event),
            HostBehavior::GujianFoodQingtuan => self.filter_common(ctx, owner, event),
            HostBehavior::GujianFoodTanhuadong => self.filter_common(ctx, owner, event),
            HostBehavior::GujianFoodYougeng => self.filter_common(ctx, owner, event),
            HostBehavior::GujianFoodZiyangdan => self.filter_common(ctx, owner, event),
            HostBehavior::GujianYunvyuanshen => self.filter_common(ctx, owner, event),
            HostBehavior::GwentBirinState => self.filter_common(ctx, owner, event),
            HostBehavior::GwentCiguState => self.filter_common(ctx, owner, event),
            HostBehavior::GwentKunenState => self.filter_common(ctx, owner, event),
            HostBehavior::GwentQinpenState => self.filter_common(ctx, owner, event),
            HostBehavior::HuanleMonkeyCard => self.filter_common(ctx, owner, event),
            HostBehavior::HuanleXuelunyangCard => self.filter_common(ctx, owner, event),
            HostBehavior::XianjianChangnian => self.filter_xianjian(ctx, owner, event),
            // 长念·思：自己阶段末（runtime 授技实例持有者）,
            HostBehavior::XianjianChangnian2 => self.filter_xianjian(ctx, owner, event),
            HostBehavior::XianjianChengxin => self.filter_xianjian(ctx, owner, event),
            HostBehavior::XianjianCtianfu2 => self.filter_xianjian(ctx, owner, event),
            // 天符·清挂：挂附者死亡,
            HostBehavior::XianjianDuci => self.filter_xianjian(ctx, owner, event),
            HostBehavior::XianjianFenglue => self.filter_xianjian(ctx, owner, event),
            HostBehavior::XianjianFenshi => self.filter_xianjian(ctx, owner, event),
            // 龙火：自己阶段结束且已觉醒（持有龙火）,
            HostBehavior::XianjianHusha => self.filter_xianjian(ctx, owner, event),
            HostBehavior::XianjianHushaCount => self.filter_xianjian(ctx, owner, event),
            // 虎煞：有标记+（可虚拟杀：有其他存活目标）或（可虚拟南蛮：
            // 标记≥2）,
            HostBehavior::XianjianJtjubao => self.filter_xianjian(ctx, owner, event),
            HostBehavior::XianjianLiefeng => self.filter_xianjian(ctx, owner, event),
            HostBehavior::XianjianLiguang => self.filter_xianjian(ctx, owner, event),
            HostBehavior::XianjianLinghuo => self.filter_xianjian(ctx, owner, event),
            HostBehavior::XianjianLingquan => self.filter_xianjian(ctx, owner, event),
            // 神雾：水云计数≥3+未觉醒（juexingji 一次性）,
            HostBehavior::XianjianLonghuo => self.filter_xianjian(ctx, owner, event),
            HostBehavior::XianjianPoyun => self.filter_xianjian(ctx, owner, event),
            HostBehavior::XianjianQianfang => self.filter_xianjian(ctx, owner, event),
            HostBehavior::XianjianRunxin => self.filter_xianjian(ctx, owner, event),
            HostBehavior::XianjianShenmu => self.filter_xianjian(ctx, owner, event),
            // 毒刺：自己弃牌含装备 kind+距离 1 内有其他存活角色（登记差
            // 异：仅弃置路径；被获得/替换不触发）,
            HostBehavior::XianjianShenwu => self.filter_xianjian(ctx, owner, event),
            HostBehavior::XianjianSheying => self.filter_xianjian(ctx, owner, event),
            // 花好：任意其他存活角色（多目标 [1,∞]）+未发动,
            HostBehavior::XianjianShiying => self.filter_xianjian(ctx, owner, event),
            HostBehavior::XianjianShuiyun5 => self.filter_xianjian(ctx, owner, event),
            // 忘忧：他人阶段末+自己有牌+本回合存在受伤标记者,
            HostBehavior::XianjianWangyou => self.filter_xianjian(ctx, owner, event),
            HostBehavior::XianjianWangyou2 => self.filter_xianjian(ctx, owner, event),
            HostBehavior::XianjianXjzhimeng => self.filter_xianjian(ctx, owner, event),
            // 织梦·梦：暗牌持有者被使用同类型（三分类）牌（targets 含
            // 持有者+暗牌保留=上游 storage 不清除同口径）,
            HostBehavior::XianjianXshuangren => self.filter_xianjian(ctx, owner, event),
            // 双刃·殁：自己死亡帧,
            HostBehavior::XianjianXshuangren2 => self.filter_xianjian(ctx, owner, event),
            HostBehavior::XianjianXunying => self.filter_xianjian(ctx, owner, event),
            HostBehavior::XianjianZhangmu => self.filter_xianjian(ctx, owner, event),
            // 双刃·殁：自己死亡帧,
            HostBehavior::GwentFengyinState => self.filter_common(ctx, owner, event),
            // 机关凤封印：拥有者任一阶段开始即解除（上游
            // {player:'phaseBegin'} 同口径）
            HostBehavior::SwdFengyinState => self.filter_swd(ctx, owner, event),
            HostBehavior::SwdJiguanyuanGain => self.filter_swd(ctx, owner, event),
            HostBehavior::SwdJiguanfengCard => self.filter_swd(ctx, owner, event),
            HostBehavior::SwdJiguanyuanCard => self.filter_swd(ctx, owner, event),
            HostBehavior::SwdJiguanyaoshuCard => self.filter_swd(ctx, owner, event),
            HostBehavior::SwdJiguanyaoState => self.filter_swd(ctx, owner, event),
            HostBehavior::SwdJiguanshuCard => self.filter_swd(ctx, owner, event),
            HostBehavior::SwdTianfuzhenCard => self.filter_swd(ctx, owner, event),
            HostBehavior::SwdQixingzhenCard => self.filter_swd(ctx, owner, event),
            HostBehavior::SwdNiaoxiangzhenCard => self.filter_swd(ctx, owner, event),
            HostBehavior::SwdChangshezhenCard => self.filter_swd(ctx, owner, event),
            HostBehavior::SwdPozhenjueCard => self.filter_swd(ctx, owner, event),
            // 机关妖术补偿：非自己回合、弃牌含装备 Kind 牌（以弃装备牌近
            // 似"失去装备区牌"，登记差异：替换/获得离去路径不触发）
            HostBehavior::SwdJiguanyaoGain => self.filter_swd(ctx, owner, event),
            HostBehavior::SwdXuanfengzhirenCard => self.filter_swd(ctx, owner, event),
            HostBehavior::SwdZhongxinghujiaCard => self.filter_swd(ctx, owner, event),
            HostBehavior::SwdXingtigaizaoCard => self.filter_swd(ctx, owner, event),
            HostBehavior::SwdShijianhuisuCard => self.filter_swd(ctx, owner, event),
            HostBehavior::SwdJinjilengdongCard => self.filter_swd(ctx, owner, event),
            HostBehavior::SwdShengxiuhaojiaoCard => self.filter_swd(ctx, owner, event),
            HostBehavior::SwdYinmilichangCard => self.filter_swd(ctx, owner, event),
            HostBehavior::HuanleMianjuCard => self.filter_common(ctx, owner, event),
            HostBehavior::ZhangbaSha => self.filter_common(ctx, owner, event),
            HostBehavior::SwdGouhunluoSub => self.filter_swd(ctx, owner, event),
            HostBehavior::XianjianShuiyun => self.filter_xianjian(ctx, owner, event),
            HostBehavior::XianjianZhaoyao => self.filter_xianjian(ctx, owner, event),
            HostBehavior::WudianShuaiyan => self.filter_common(ctx, owner, event),
            HostBehavior::WudianGeju => self.filter_common(ctx, owner, event),
            HostBehavior::WudianFeidan => self.filter_common(ctx, owner, event),
            HostBehavior::WudianGuangshuDiamond => self.filter_common(ctx, owner, event),
            HostBehavior::WudianGuangshuHeart => self.filter_common(ctx, owner, event),
            HostBehavior::WudianJidong => self.filter_common(ctx, owner, event),
            HostBehavior::WudianMengji => self.filter_common(ctx, owner, event),
            HostBehavior::WudianOwliyong => self.filter_common(ctx, owner, event),
            HostBehavior::WudianPingzhang => self.filter_common(ctx, owner, event),
            // 应用：自己摸牌阶段开始+上回合有屏障次数,
            HostBehavior::WudianQiangguGuard => self.filter_common(ctx, owner, event),
            HostBehavior::WudianShenqiang => self.filter_common(ctx, owner, event),
            HostBehavior::WudianShihuo => self.filter_common(ctx, owner, event),
            // 猛击：你无护甲时的杀伤害（非传导；上游 filter storage.
            // zhongdun && !hujia && sha && notLink 同口径——zhongdun 置
            // 位以持技等价）,
            HostBehavior::WudianXiyang => self.filter_common(ctx, owner, event),
            HostBehavior::WudianYuedong => self.filter_common(ctx, owner, event),
            HostBehavior::WudianZhiyuan => self.filter_common(ctx, owner, event),
            HostBehavior::WudianZhucheng => self.filter_common(ctx, owner, event),
            HostBehavior::WudianZhuchengGuard => self.filter_common(ctx, owner, event),
            HostBehavior::XianjianAnwugu => self.filter_xianjian(ctx, owner, event),
            HostBehavior::XianjianBinxin => self.filter_xianjian(ctx, owner, event),
            // 回梦：你回复体力（灵雪同款 filter）,
            HostBehavior::XianjianFeixu => self.filter_xianjian(ctx, owner, event),
            // 风：自己摸牌（有量）,
            HostBehavior::XianjianFeng => self.filter_xianjian(ctx, owner, event),
            // 雅：自己受伤（未防止有量）,
            HostBehavior::XianjianHuimeng => self.filter_xianjian(ctx, owner, event),
            // 蛇蜕：属性伤害对你（有属性=非普通；传导事件同免疫=上游
            // hasNature 同口径）,
            HostBehavior::XianjianLingxue => self.filter_xianjian(ctx, owner, event),
            // 冰心：任意体力为 1 的角色阶段结束（含自己）,
            HostBehavior::XianjianSong => self.filter_xianjian(ctx, owner, event),
            // 御风：失去手牌（弃置路径含手牌）且手牌<2、本回合<2 次,
            HostBehavior::XianjianTianshe => self.filter_xianjian(ctx, owner, event),
            // 蛇蜕·息：你造成属性伤害（未防止有量）且已受伤,
            HostBehavior::XianjianTianshe2 => self.filter_xianjian(ctx, owner, event),
            // 返灵：他人失去体力（有量）且自己已受伤、本回合未发动,
            HostBehavior::XianjianXjyufeng => self.filter_xianjian(ctx, owner, event),
            // 玄炎·燃：你的火属性伤害（非传导）,
            HostBehavior::XianjianXuanyan2 => self.filter_xianjian(ctx, owner, event),
            // 玄炎·烬：你造成火属性伤害（未防止有量）,
            HostBehavior::XianjianXuanyan3 => self.filter_xianjian(ctx, owner, event),
            // 凝冰：你受到雷属性伤害（未防止有量）
            HostBehavior::XianjianYa => self.filter_xianjian(ctx, owner, event),
            // 颂：自己造成伤害（未防止有量）
            HostBehavior::WudianYinshen => self.filter_common(ctx, owner, event),
            HostBehavior::WudianPaotaiFire => self.filter_common(ctx, owner, event),
            HostBehavior::XianjianLuanjian2 => self.filter_xianjian(ctx, owner, event),
            // 四象隐藏链 2/3 + 后援3/4：本人死亡且本人是 Boss
            HostBehavior::BossQingmu2
            | HostBehavior::BossQingmu3
            | HostBehavior::BossBaimang2
            | HostBehavior::BossChiyan2
            | HostBehavior::BossXuanlin2
            | HostBehavior::BossBaimang3
            | HostBehavior::BossChiyan3
            | HostBehavior::BossXuanlin3
            | HostBehavior::BossBianshen3
            | HostBehavior::BossBianshen4 => self.filter_common(ctx, owner, event),
            // 四象隐藏链·转阶 + 后援3/4·转阶：同上（技能仍在 DeathMachine 前）
            HostBehavior::BossQingmu2x
            | HostBehavior::BossQingmu3x
            | HostBehavior::BossBaimang2x
            | HostBehavior::BossChiyan2x
            | HostBehavior::BossXuanlin2x
            | HostBehavior::BossBaimang3x
            | HostBehavior::BossChiyan3x
            | HostBehavior::BossXuanlin3x
            | HostBehavior::BossBianshen3x
            | HostBehavior::BossBianshen4x => self.filter_common(ctx, owner, event),
            // 还人开局：关卡尚未进入十殿，且持有者是 Boss
            HostBehavior::BossHuanren => self.filter_common(ctx, owner, event),
            // 还人转关：十殿链且等级未终（0..2 还有下一关）
            HostBehavior::BossHuanrenDie => self.filter_common(ctx, owner, event),
            HostBehavior::BossQingyi => self.filter_common(ctx, owner, event),
            HostBehavior::BossChiyi => self.filter_common(ctx, owner, event),
            HostBehavior::BossChiyiMark => self.filter_common(ctx, owner, event),
            HostBehavior::BossBaiyi => self.filter_common(ctx, owner, event),
            HostBehavior::BossBaiyiThunder => self.filter_common(ctx, owner, event),
            HostBehavior::D3Moyan => self.filter_common(ctx, owner, event),
            _ => false,
        }
    }
}
