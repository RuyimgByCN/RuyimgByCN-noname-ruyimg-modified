use super::*;

mod common;
mod wudian;
mod swd;
mod xianjian;
mod source_order;

impl HostBehavior {
    pub fn trigger_meta(&self) -> Option<TriggerMeta> {
        let mut meta = match self {
            HostBehavior::ShaDamageForceDiscard => self.trigger_meta_common(),
            HostBehavior::PhaseEndRecoverOrDraw => self.trigger_meta_common(),
            HostBehavior::YunchouXicai => self.trigger_meta_common(),
            HostBehavior::YunchouDiykuanggu => self.trigger_meta_common(),
            HostBehavior::YunchouXiongzi => self.trigger_meta_common(),
            HostBehavior::YunchouYaliang => self.trigger_meta_common(),
            // 纵火：伤害前置窗（nature 写回须在 commit 前生效）
            HostBehavior::YunchouZonghuo => self.trigger_meta_common(),
            HostBehavior::YunchouShaoying => self.trigger_meta_common(),
            HostBehavior::YunchouLuweiyan => self.trigger_meta_common(),
            HostBehavior::YunchouJieyan => self.trigger_meta_common(),
            HostBehavior::YunchouYiesheng => self.trigger_meta_common(),
            HostBehavior::WudianDiyjuntun => self.trigger_meta_wudian(),
            HostBehavior::WudianJinyan => self.trigger_meta_wudian(),
            // 理断：获得结算后（CardsMoved after）
            HostBehavior::WudianLiduan => self.trigger_meta_wudian(),
            // 坚壁：锁定查询面（AOE 目标段查询，无触发帧）
            HostBehavior::WudianJianbi => self.trigger_meta_wudian(),
            // 复仇：目录锚（取消逻辑在 ShaResolveMachine 内联）
            HostBehavior::WudianFuchou => self.trigger_meta_wudian(),
            // 良策·余获：锁定查询面（五谷机器内联查询，无触发帧）
            HostBehavior::WudianLiangceRemain => self.trigger_meta_wudian(),
            // 归汉：目录锚（limited 确认逻辑在 DyingSaveMachine 内联）
            HostBehavior::WudianGuihan => self.trigger_meta_wudian(),
            HostBehavior::SwdGTiandao => self.trigger_meta_swd(),
            // 魂身：自己使用/打出闪（上游 respond/useCard，frequent）
            HostBehavior::SwdHyunshen => self.trigger_meta_swd(),
            // 魂身·收束：自己准备阶段开始
            HostBehavior::SwdHyunshen2 => self.trigger_meta_swd(),
            // 待行：自己结束阶段（direct）
            HostBehavior::SwdDaixing => self.trigger_meta_swd(),
            // 待行·清甲：自己准备阶段开始（silent）
            HostBehavior::SwdDaixing2 => self.trigger_meta_swd(),
            // 待行·吸收：自己受伤后（silent）
            HostBehavior::SwdDaixing3 => self.trigger_meta_swd(),
            // 聚悉：主动技（无触发时机）
            HostBehavior::SwdJuxi => self.trigger_meta_swd(),
            // 聚悉·计数：任意角色弃牌后（global）
            HostBehavior::SwdJuxiCount => self.trigger_meta_swd(),
            // 破舟：自己受伤后
            HostBehavior::SwdPozhou => self.trigger_meta_swd(),
            // 破舟·封印：主动技（无触发时机）
            HostBehavior::SwdPozhou2 => self.trigger_meta_swd(),
            // 陷阱：主动技（无触发时机）
            HostBehavior::SwdSxianjing => self.trigger_meta_swd(),
            // 陷阱·触发：他人使用牌（目标含自己）
            HostBehavior::SwdSxianjingGain => self.trigger_meta_swd(),
            // 陷阱·取回：自己受伤后
            HostBehavior::SwdSxianjingDamage => self.trigger_meta_swd(),
            // 孤行：主动技（无触发时机）
            HostBehavior::SwdGuxing => self.trigger_meta_swd(),
            // 孤行·计数：自己以杀造成伤害后
            HostBehavior::SwdGuxing3 => self.trigger_meta_swd(),
            // 灵矢：锁定旗标（无触发时机；四条效果内联于距离/杀限次/
            // 直中/摸牌四臂，天弦同款）
            HostBehavior::SwdLingshi => self.trigger_meta_swd(),
            // 锦鳞：主动技（limited 一次性；无触发时机）
            HostBehavior::SwdJinlin => self.trigger_meta_swd(),
            // 禳星：自己受伤害结算后 / 失去体力后（silent forced）
            HostBehavior::HlssXiangxing => self.trigger_meta_common(),
            // 隐月：伤害前置窗（num+1 写回 + 记录伤害类型）
            HostBehavior::HlssYueyin => self.trigger_meta_common(),
            // 起风：准备阶段开始（Judge 阶段 = 上游 phaseZhunbei），可选
            HostBehavior::HlssFengqi => self.trigger_meta_common(),
            // 起风·闭 / 改命：锁定旗标（无触发时机；效果内联）
            HostBehavior::HlssFengqi2 => self.trigger_meta_common(),
            HostBehavior::HlssGaiming => self.trigger_meta_common(),
            // 潜龙：开局首阶段（phase_number==1）+ 获得牌 + 伤害两向
            HostBehavior::MbQianlong => self.trigger_meta_common(),
            // 涅槃：自己濒死（可选；出牌阶段面走 activate）
            HostBehavior::Niepan => self.trigger_meta_common(),
            // 清正：出牌阶段开始（可选）
            HostBehavior::MbcmQingzheng => self.trigger_meta_common(),
            // 烈弓：被动（杀机器内联，无触发时机）
            HostBehavior::FzLiegong => self.trigger_meta_common(),
            // 狂骨：自己造成伤害后（可选）
            HostBehavior::FzXinKuanggu => self.trigger_meta_common(),
            // 苦肉：出牌阶段主动（activate 分支；无触发时机）
            HostBehavior::Kurou => self.trigger_meta_common(),
            // 主亡忠继：任意角色死亡（锁定）
            HostBehavior::ZhuWanzhongJi => self.trigger_meta_common(),
            // 摸牌翻倍：被动（摸牌收束内联，无触发时机）
            HostBehavior::XiguaMopaiFanbei => self.trigger_meta_common(),
            // 血量翻倍：首次阶段开始（一次性；近似 gameStart）
            HostBehavior::XiguaXueliangFanbei => self.trigger_meta_common(),
            // 门神：自己结束阶段（可选）
            HostBehavior::YxsMenshen => self.trigger_meta_common(),
            // 门神·清除：任意阶段开始 + 任意角色死亡（锁定）
            HostBehavior::YxsMenshen3 => self.trigger_meta_common(),
            // 门神·标记持有：被动（无触发时机）
            HostBehavior::YxsMenshen2 => self.trigger_meta_common(),
            // 反击：自己受到伤害后（可选）
            HostBehavior::YxsFanji => self.trigger_meta_common(),
            // 反击·红杀直伤：被动（杀机器内联，无触发时机）
            HostBehavior::YxsFanji2 => self.trigger_meta_common(),
            // 灾变：自己出牌阶段开始（锁定）
            HostBehavior::Zaibian => self.trigger_meta_common(),
            // 誓师：自己击杀角色后（锁定）
            HostBehavior::Shishi => self.trigger_meta_common(),
            // 循梦：自己造成伤害（**伤害前置窗**；上游 damageBegin 同窗）
            HostBehavior::Xunmeng => self.trigger_meta_common(),
            // 集舍：出牌阶段主动（activate 分支；无触发时机）
            HostBehavior::Jishe => self.trigger_meta_common(),
            // 诈降：被动（mod/摸牌/直伤面内联，无触发时机）
            HostBehavior::SbZhaXiang => self.trigger_meta_common(),
            // 每步：任意角色出牌阶段开始（可选）
            HostBehavior::NewMeibu => self.trigger_meta_common(),
            // 每步·距离：被动（无触发时机）
            HostBehavior::NewMeibuRange => self.trigger_meta_common(),
            // 止息：自己使用牌（锁定；firstDo）
            HostBehavior::NewZhixi => self.trigger_meta_common(),
            // 撷芳：全局死亡结束 / 摸牌阶段结束（锁定）
            HostBehavior::Xiefang => self.trigger_meta_common(),
            // 让节：自己受到伤害后（可选）
            HostBehavior::Rangjie => self.trigger_meta_common(),
            // 蛊惑：出牌阶段主动（activate 分支；无触发时机）
            HostBehavior::Reguhuo => self.trigger_meta_common(),
            // 攻心：出牌阶段主动（activate 分支；无触发时机）
            HostBehavior::Gongxin => self.trigger_meta_common(),
            // 魔化：自己进入濒死（锁定）
            HostBehavior::Mohua2 => self.trigger_meta_common(),
            // 修罗：自己准备阶段开始（可选）
            HostBehavior::Xiuluo => self.trigger_meta_common(),
            // 灭道·弃：自己弃牌阶段开始（锁定）
            HostBehavior::Miedao => self.trigger_meta_common(),
            // 魔焰：出牌阶段主动（activate 分支；无触发时机）
            HostBehavior::Moyan => self.trigger_meta_common(),
            // 疾风：自己使用多目标单目标牌（结算窗；锁定）
            HostBehavior::Jifeng => self.trigger_meta_common(),
            // 昆仑镜：阶段收束（锁定记录快照）+ 阶段开始（体力低于快照值
            // 时**机器内确认门**回滚——上游两面可选性不同，故不用
            // TriggerMeta.optional）
            HostBehavior::Kunlunjing => self.trigger_meta_common(),
            // 神弓：被动修饰（无触发时机；响应窗内联候选）
            HostBehavior::Shengong => self.trigger_meta_common(),
            // 轩辕剑·伤增：装备者造成伤害前（锁定）
            HostBehavior::XuanyuanjianDamage => self.trigger_meta_common(),
            // 轩辕剑·反噬：装备者造成伤害后（锁定）
            HostBehavior::XuanyuanjianBacklash => self.trigger_meta_common(),
            // 轩辕：自己结束阶段（限定；可选）
            HostBehavior::Xuanyuan => self.trigger_meta_common(),
            // 燎原：自己使用【杀】（结算窗；可选）
            HostBehavior::Liaoyuan => self.trigger_meta_common(),
            // 改造：装备前置窗（DamageBefore 类；filter 检查 Equipped）
            HostBehavior::Gaizao => self.trigger_meta_common(),
            // 天书：出牌阶段主动（activate 分支；无触发时机）
            HostBehavior::SwdTianshu => self.trigger_meta_swd(),
            // 李香龙：自己的杀被闪避后（可选）
            HostBehavior::Lxianglong => self.trigger_meta_common(),
            // 移花：成为单目标牌的目标（使用结算窗；可选）
            HostBehavior::Yihua => self.trigger_meta_common(),
            // 贞烈：成为他人【杀】/锦囊目标（使用结算窗；可选）
            HostBehavior::Zhenlie => self.trigger_meta_common(),
            // 天伦：判定机改判窗驱动（无触发时机）
            HostBehavior::Tianlun => self.trigger_meta_common(),
            // 脉轮：任意阶段开始（可选）
            HostBehavior::Mailun => self.trigger_meta_common(),
            // 极略：出牌阶段主动（activate 分支；无触发时机）
            HostBehavior::Jilve => self.trigger_meta_common(),
            // 革新印：出牌阶段主动（activate 分支；无触发时机）
            HostBehavior::Gxianyin => self.trigger_meta_common(),
            // 冰封·封：自己翻回正面时（自移除）
            HostBehavior::Bingfeng2 => self.trigger_meta_common(),
            // 冰封：出牌阶段主动（activate 分支；无触发时机）
            HostBehavior::Bingfeng => self.trigger_meta_common(),
            // 此桥邪：自己失去装备牌后（频繁技——恒发动，无确认门）
            HostBehavior::Cyqiaoxie => self.trigger_meta_common(),
            // 魂殉志 / 殉：无触发时机（主动面 + 内联臂）
            HostBehavior::Hxunzhi => self.trigger_meta_common(),
            HostBehavior::Hxunzhi2 => self.trigger_meta_common(),
            // 护天：自己阶段结束（可选）
            HostBehavior::Hutian => self.trigger_meta_common(),
            // 天香：自己受到伤害前（可选）
            HostBehavior::Tianxiang => self.trigger_meta_common(),
            // 天香·受：自己受到伤害结算后（运行时授予）
            HostBehavior::Tianxiang2 => self.trigger_meta_common(),
            // 颂词：出牌阶段主动（activate 分支；无触发时机）
            HostBehavior::Songci => self.trigger_meta_common(),
            // 诛害：任意角色结束阶段开始（可选）
            HostBehavior::SwdZhuhai => self.trigger_meta_swd(),
            // 林韵：出牌阶段主动（activate 分支；无触发时机）
            HostBehavior::SwdLinyun => self.trigger_meta_swd(),
            // 集控：自己阶段开始 或 自己失去手牌（可用：每回合一次）
            HostBehavior::SwdJikong => self.trigger_meta_swd(),
            // 魅惑：自己失去装备牌后（可选）
            HostBehavior::SwdMeihuo => self.trigger_meta_swd(),
            // 守印：任意角色濒死（可选）
            HostBehavior::SwdShouyin => self.trigger_meta_swd(),
            // 义山：自己受伤结算后（可选；上游 notemp 直发，原生可选窗）
            HostBehavior::SwdYishan => self.trigger_meta_swd(),
            // 入梦：任意角色出牌阶段开始前（可选）
            HostBehavior::SwdRumeng => self.trigger_meta_swd(),
            // 神焰：伤害前置窗（+1 并记录蔓延目标）+ 伤害结算后（蔓延）
            HostBehavior::SwdShenyan => self.trigger_meta_swd(),
            // 灵武：无触发时机（效果内联于阶段收束）
            HostBehavior::SwdLingwu => self.trigger_meta_swd(),
            // 浴火 / 神威：无触发时机（浴火由涅槃收束内联；神威为 kernel 臂）
            HostBehavior::Yuhuo => self.trigger_meta_common(),
            HostBehavior::Shenwei => self.trigger_meta_common(),
            // 卫统：开局首阶段（登记面）
            HostBehavior::MbWeitong => self.trigger_meta_common(),
            // 隐月·判：翻面前置取消（不能被翻面；判定不生效由 kernel 内联）
            HostBehavior::HlssYueyin2 => self.trigger_meta_common(),
            // 锦鳞·退：持有者阶段开始（silent）
            HostBehavior::SwdJinlin2 => self.trigger_meta_swd(),
            HostBehavior::SwdShehun => self.trigger_meta_swd(),
            HostBehavior::SwdNlianji => self.trigger_meta_swd(),
            // 铁壁：成为杀目标时（CardUsed 帧等价承载，登记差异：上游
            // target shaBefore，原生以 CardUsed+targets 等价承载）
            HostBehavior::SwdTiebi => self.trigger_meta_swd(),
            // 大同：任意玩家阶段结束（global；frequent 自动无门）
            HostBehavior::SwdDatong => self.trigger_meta_swd(),
            // 承剑：其他角色受伤结算后（global；确认门=optional）
            HostBehavior::SwdChengjian => self.trigger_meta_swd(),
            // 缚斧：伤害前置窗（num 直写）
            HostBehavior::SwdJufu => self.trigger_meta_swd(),
            // 花镜：自己造成伤害结算后（frequent 自动无门）
            HostBehavior::SwdHuajing => self.trigger_meta_swd(),
            // 护盾：自己造成伤害结算后（forced 无门）
            HostBehavior::SwdHudun => self.trigger_meta_swd(),
            // 破星：伤害前置窗（num 直写）
            HostBehavior::SwdPoxing => self.trigger_meta_swd(),
            // 流虹：使用杀（frequent 自动无门）
            HostBehavior::SwdLiuhong => self.trigger_meta_swd(),
            // 墨剑：自己使用杀（可选确认门；上游 shaBegin）
            HostBehavior::SwdMojian => self.trigger_meta_swd(),
            // 道法：其他角色受伤结算后（global；确认门=optional）
            HostBehavior::SwdDaofa => self.trigger_meta_swd(),
            // 玉鳞：受伤前置窗口（可选；弃装备牌免伤）
            HostBehavior::SwdYulin => self.trigger_meta_swd(),
            // 凝霜：成为黑牌目标（可选；上游 useCardToBegin）
            HostBehavior::SwdNingshuang => self.trigger_meta_swd(),
            // 工神：其他角色用基本/锦囊牌（可选；上游 global useCard）
            HostBehavior::SwdGongshen => self.trigger_meta_swd(),
            // 平恤：无触发面（computed_distance 字段消费）
            HostBehavior::SwdPingxu => self.trigger_meta_swd(),
            // 还灵：自己阶段结束（可选确认门；上游 player phaseEnd）
            HostBehavior::SwdHuanling => self.trigger_meta_swd(),
            // 断月：出牌主动技（ActivateSkill 通道，usable:1 走 limits）
            HostBehavior::SwdDuanyue => self.trigger_meta_swd(),
            // 屠阵：自己造成伤害结算后（可选；上游 source damageAfter）
            HostBehavior::SwdTuzhen => self.trigger_meta_swd(),
            // 落梅：使用或打出（频率技；上游 useCard+respond 双事件）
            HostBehavior::SwdLuomei => self.trigger_meta_swd(),
            // 星点：出牌主动技（ActivateSkill 通道，usable:1 走 limits）
            HostBehavior::SwdXingdian => self.trigger_meta_swd(),
            // 协律：弃牌阶段结束=End 帧入场（可选；上游 phaseDiscardEnd）
            HostBehavior::SwdXielv => self.trigger_meta_swd(),
            // 雪煌：出牌主动技（ActivateSkill 通道，limited bool 门）
            HostBehavior::SwdXuehuang => self.trigger_meta_swd(),
            // 还侠：无触发面（PlayCard 转化臂+kernel 跟踪钩承载）
            HostBehavior::SwdHuanxia => self.trigger_meta_swd(),
            // 破月：无触发面（PlayCard 臂+kernel 计数/闪询问钩承载）
            HostBehavior::SwdPoyue => self.trigger_meta_swd(),
            // 收纳：出牌主动技（usable:1 走 limits）
            HostBehavior::SwdLianyaohuShouna => self.trigger_meta_swd(),
            // 炼化：出牌主动技（usable:1 走 limits）
            HostBehavior::SwdLianyaohuLianhua => self.trigger_meta_swd(),
            // 神农鼎：出牌主动技（usable:1 走 limits）
            HostBehavior::SwdShennongdingCard => self.trigger_meta_swd(),
            // 昆仑镜/东皇钟：出牌主动技/阶段末触发（东皇钟 direct forced）
            HostBehavior::SwdKunlunjingcCard => self.trigger_meta_swd(),
            HostBehavior::SwdDonghuangzhongCard => self.trigger_meta_swd(),
            // 鬼蛇旗/仙露回：trick 卡牌机（元数据不进触发计划）
            HostBehavior::GuisheqiCard => self.trigger_meta_common(),
            HostBehavior::XianluhuiCard => self.trigger_meta_common(),
            // 青龙之圭/黄麟之琮：准备阶段（Judge 帧入场 forced）
            HostBehavior::SwdQinglongzhiguiHeld => self.trigger_meta_swd(),
            HostBehavior::SwdHuanglinzhicongHeld => self.trigger_meta_swd(),
            HostBehavior::SwdCangchizhibiHeld => self.trigger_meta_swd(),
            // 朱雀之璋：受伤结算后 forced
            HostBehavior::SwdZhuquezhizhangHeld => self.trigger_meta_swd(),
            // 白兽之琥：弃牌阶段结束 direct（蝶舞口径）
            HostBehavior::SwdBaishouzhihuHeld => self.trigger_meta_swd(),
            // 玉蝉换形：自己用牌结算后 silent forced
            HostBehavior::SwdYuchanSwap => self.trigger_meta_swd(),
            // 疾影：锁定 mod 无触发面（sha_range_free 链查询）
            HostBehavior::SwdJiyingHeld => self.trigger_meta_swd(),
            // 沐风：结束阶段确认门（frequent 上游→确认近似）
            HostBehavior::SwdMufengHeld => self.trigger_meta_swd(),
            // 望尘：弃牌阶段结束确认门（Discard→End 入场近似）
            HostBehavior::SwdWangchenHeld => self.trigger_meta_swd(),
            // 神草结：锦囊伤害前置窗口（确认门；上游 chooseToUse）
            HostBehavior::SwdShencaojieGuard => self.trigger_meta_swd(),
            // 玄武之璜：自己造成伤害结算后 forced
            HostBehavior::SwdXuanwuzhihuangHeld => self.trigger_meta_swd(),
            // 逆攻积攒：受伤结算后 forced
            HostBehavior::SwdNigongAccumulate => self.trigger_meta_swd(),
            // 逆攻发动：出牌主动技（标记门在 activate）
            HostBehavior::SwdNigongStrike => self.trigger_meta_swd(),
            // 四非珍面：出牌主动技（usable:1 走 limits）
            HostBehavior::SwdSifeizhenmianCard => self.trigger_meta_swd(),
            // 空洞印：死亡前置窗口（forced；上游 dieBefore cancel 同口径）
            HostBehavior::SwdKongdongyinGuard => self.trigger_meta_swd(),
            // 女娲石：他人濒死（确认门；上游 global dying priority 6）
            HostBehavior::SwdNvwashiGuard => self.trigger_meta_swd(),
            // 壶境：准备阶段（Judge 帧入场 forced；上游 phaseBegin 锁定）
            HostBehavior::SwdHujing => self.trigger_meta_swd(),
            // 朱羽：横置者受伤前置窗口（可选；上游 global damageBegin）
            HostBehavior::SwdZhuyu => self.trigger_meta_swd(),
            // 分舵：出牌主动技（ActivateSkill 通道，每回合限一次走 limits）
            HostBehavior::SwdJiefen => self.trigger_meta_swd(),
            // 神秘果窗口：出牌阶段用牌结算后（确认门=optional——上游
            // chooseToUse 即确认）
            HostBehavior::SwdShenmiguoWindow => self.trigger_meta_swd(),
            // 鼠槌：出牌主动技（ActivateSkill 通道，usable:1 走 limits）
            HostBehavior::SwdShuchuiUse => self.trigger_meta_swd(),
            // 连舞：锁定 marker（engine max_targets 链+kernel 红杀 directHit
            // 臂查询），无触发面
            HostBehavior::SwdLianwu => self.trigger_meta_swd(),
            // 浴尘：回合外用牌/响应结算后（双事件；确认门=optional）
            HostBehavior::SwdYuchen => self.trigger_meta_swd(),
            // 挟雷：使用/打出杀结算后（双事件；确认门=optional）
            HostBehavior::SwdXielei => self.trigger_meta_swd(),
            // 对诗：出牌主动技（ActivateSkill 通道，usable:1 走 limits）
            HostBehavior::SwdDuishi => self.trigger_meta_swd(),
            // 连打：杀结算完毕后（确认门=optional）
            HostBehavior::SwdLianda => self.trigger_meta_swd(),
            // 隳延：造成伤害结算后（确认门=optional）
            HostBehavior::SwdSuiyan => self.trigger_meta_swd(),
            // 断行：装备牌入装结算后（确认门=optional）
            HostBehavior::SwdDuanxing => self.trigger_meta_swd(),
            // 破浪：造成伤害结算后（确认门=optional）
            HostBehavior::SwdPolang => self.trigger_meta_swd(),
            // 诱引：其他角色弃牌结算后（frequent 锁定式=非 optional）
            HostBehavior::SwdYouyin => self.trigger_meta_swd(),
            // 湛卢：出牌主动技（ActivateSkill 通道，usable:1 走 limits）
            HostBehavior::SwdZhanlu => self.trigger_meta_swd(),
            // 苏生：死亡前置窗口（Timing::First 同空洞印；确认门=
            // optional；每回合限一次走 limits——OptionalGate 消费 once_used）
            HostBehavior::SwdSusheng => self.trigger_meta_swd(),
            // 相惠：出牌主动技（ActivateSkill 通道，usable:1 走 limits）
            HostBehavior::SwdXianghui => self.trigger_meta_swd(),
            // 挥琪：受伤结算后（确认门=optional）
            HostBehavior::SwdHuiqi => self.trigger_meta_swd(),
            // 万钧：锁定 marker（engine PlayCard viewAs 链查询），无触发面
            HostBehavior::SwdWanjun => self.trigger_meta_swd(),
            // 森罗：出牌主动技（ActivateSkill 通道，usable:1 走 limits）
            HostBehavior::SwdSenluo => self.trigger_meta_swd(),
            // 火扑：锁定 marker（engine PlayCard viewAs 链查询），无触发面
            HostBehavior::SwdHuopu => self.trigger_meta_swd(),
            // 夜影：锁定 marker（engine PlayCard viewAs 链查询），无触发面
            HostBehavior::SwdYeying => self.trigger_meta_swd(),
            // 归影：锁定 marker（engine PlayCard viewAs 链查询），无触发面
            HostBehavior::SwdGuiying => self.trigger_meta_swd(),
            // 凤泽：锁定 marker（engine PlayCard viewAs 链查询），无触发面
            HostBehavior::SwdFengze => self.trigger_meta_swd(),
            // 玲月：锁定（sha 机器内联臂查询），无触发面
            HostBehavior::SwdLingyue => self.trigger_meta_swd(),
            // 暗莲映：其他/自己弃牌结算后（frequent 锁定式=非 optional）
            HostBehavior::SwdAnlianying => self.trigger_meta_swd(),
            // 疾风：锁定 marker（engine max_targets 链+state computed_distance
            // 链查询），无触发面
            HostBehavior::SwdLjifeng => self.trigger_meta_swd(),
            // 断义：出牌主动技（ActivateSkill 通道，usable:1 走 limits）
            HostBehavior::SwdDuanyi => self.trigger_meta_swd(),
            // 乐跃：任意角色回复结算后（确认门=optional）
            HostBehavior::SwdYinyue => self.trigger_meta_swd(),
            // 灵心：自己阶段结束（frequent 锁定式=非 optional）
            HostBehavior::SwdLingxin => self.trigger_meta_swd(),
            // 渔人：锁定（sha 机器内联臂查询），无触发面
            HostBehavior::SwdYueren => self.trigger_meta_swd(),
            // 幻剑：锁定 marker（engine PlayCard viewAs 链查询），无触发面
            HostBehavior::SwdHuanjian => self.trigger_meta_swd(),
            // 虚影：伤害前置窗口（锁定取消+失去体力——Boss 绝情同窗）
            HostBehavior::SwdXuying => self.trigger_meta_swd(),
            // 乖戾：双事件（DamageBefore +1 与 Damage 后弃牌同机分流）
            HostBehavior::SwdGuaili => self.trigger_meta_swd(),
            // 无邪：锁定 marker（engine 延时校验臂查询），无触发面
            HostBehavior::SwdWuxie => self.trigger_meta_swd(),
            // 观虎：杀伤害结算后（确认门=optional）
            HostBehavior::SwdGuanhu => self.trigger_meta_swd(),
            // 敷衍·护甲面：受伤结算后（确认门=optional）
            HostBehavior::SwdFuyan => self.trigger_meta_swd(),
            // 凌波·恒爱面：闪响应/使用结算后（双事件 frequent=非 optional）
            HostBehavior::SwdHlingbo => self.trigger_meta_swd(),
            // 血裔·Miles 面：伤害前置窗口（锁定取消+失去体力）
            HostBehavior::SwdMilesXueyi => self.trigger_meta_swd(),
            // 鸣凤：出牌主动技（ActivateSkill 通道，usable:1 走 limits）
            HostBehavior::SwdFengming => self.trigger_meta_swd(),
            // 小莫雨·山小小面：造成伤害结算后（锁定强制；每回合一次走
            // limits once_per_turn——机器臂消费 inst.once_used）
            HostBehavior::SwdXiaomoyu => self.trigger_meta_swd(),
            // 冰箭：出牌主动技（ActivateSkill 通道，usable:1 走 limits）
            HostBehavior::SwdBingjian => self.trigger_meta_swd(),
            // 神鼎·麦星面：出牌主动技（ActivateSkill 通道，usable:1 走 limits）
            HostBehavior::SwdShending => self.trigger_meta_swd(),
            // 鬼眼：出牌主动技（ActivateSkill 通道，usable:1 走 limits；
            // guiyan2 濒死复活 defer）
            HostBehavior::SwdGuiyan => self.trigger_meta_swd(),
            // 袭西·凤钰面：自己出牌阶段开始（direct 直发窗口）
            HostBehavior::SwdShangxi => self.trigger_meta_swd(),
            // 花间·剑仙面：出牌阶段结束（direct 直发窗口）
            HostBehavior::SwdHuajian => self.trigger_meta_swd(),
            // 星坠·慕月面：出牌主动技（ActivateSkill 通道，usable:1 走 limits）
            HostBehavior::SwdXingzhui => self.trigger_meta_swd(),
            // 焰羽·少微面：出牌主动技（ActivateSkill 通道，usable:1 走 limits）
            HostBehavior::SwdHuangyu => self.trigger_meta_swd(),
            // 摄引：出牌主动技（ActivateSkill 通道，无 limits——上游循环面）
            HostBehavior::SwdXianyin => self.trigger_meta_swd(),
            // 破茧·齐面：自己弃牌后手牌空（锁定 frequent=非 optional）
            HostBehavior::SwdPojian => self.trigger_meta_swd(),
            // 横刀·慧彦面：回合外用牌/响应结算后（双事件；确认门=optional
            // ——上游 check 态度门近似）
            HostBehavior::SwdHwendao => self.trigger_meta_swd(),
            // 凌锋：自己阶段结束（frequent 锁定式=非 optional）
            HostBehavior::SwdLingfeng => self.trigger_meta_swd(),
            // 天火·秦壬面：出牌主动技（ActivateSkill 通道，usable:1 走 limits）
            HostBehavior::SwdTianhuo => self.trigger_meta_swd(),
            // 运筹·姬亮面：出牌主动技（ActivateSkill 通道，usable:1 走 limits）
            HostBehavior::SwdYunchou => self.trigger_meta_swd(),
            // 破晓·张烈面：锁定 marker（viewAs 链+距离链+目标上限链三查询）
            HostBehavior::SwdPoxiao => self.trigger_meta_swd(),
            // 疾风·司空宇面：锁定 marker（engine 杀距离校验臂查询），无触发面
            HostBehavior::SwdSliufeng => self.trigger_meta_swd(),
            // 乱击·多鹏面：出牌主动技（viewAs 双卡链+usable:1 走 limits）
            HostBehavior::SwdLuanji => self.trigger_meta_swd(),
            // 聒噪·疾鹏面：自己以外的任意角色受到伤害结算后（global；
            // 距离与手牌候选在机器步求取；锁定=非 optional）
            HostBehavior::SwdGuozao => self.trigger_meta_swd(),
            // 英姿·刷新面：自己摸牌阶段开始时额外摸 1（上游 phaseDrawBegin2
            // num++ 同口径；手牌上限=体力上限在 state.hand_limit 查询）
            HostBehavior::SwdReyingzi => self.trigger_meta_swd(),
            // 愚钝·福面：锁定 marker（PlayCard 禁锦囊臂+双卡 viewAs 臂查询），
            // 无触发面
            HostBehavior::SwdYudun => self.trigger_meta_swd(),
            // 占星·肯迪面：出牌主动技（usable:1 走 limits）
            HostBehavior::SwdZhanxing => self.trigger_meta_swd(),
            // 乐学·久悠面：自己回合开始（授技在机器步按当前回合分流——
            // 自己回合授{制衡/集智}、他人回合授{遗计/急救/鬼道/反馈}；
            // 回合结束清候选在 PhaseEnd 面另入 filter 分流）
            HostBehavior::SwdLexue => self.trigger_meta_swd(),
            // 枕流·临月面：出牌主动技（usable:1 走 limits）
            HostBehavior::SwdZhenjiu => self.trigger_meta_swd(),
            // 枕流触发面：自己准备阶段开始（off_court 红牌入手+回 1）
            HostBehavior::SwdZhenjiu2 => self.trigger_meta_swd(),
            // 麻醉·临月面：出牌主动技（usable:1 走 limits）
            HostBehavior::SwdLmazui => self.trigger_meta_swd(),
            // 麻醉触发面：伤害前置（off_court 黑牌返还+降 1）
            HostBehavior::SwdLmazui2 => self.trigger_meta_swd(),
            // 激昂·凤天凌面：判定为「成为目标」面——原生经 CardUsed+targets
            // 表达（frequent=非 optional）
            HostBehavior::SwdJiang => self.trigger_meta_swd(),
            // 圣手·淳于恒面：锁定 marker（engine PlayCard viewAs 链查询）
            HostBehavior::SwdShengshou => self.trigger_meta_swd(),
            // 活弹·舒莞儿面：出牌主动技（usable:1 走 limits）
            HostBehavior::SwdHuodan => self.trigger_meta_swd(),
            // 义绝：出牌主动技（usable:1 走 limits）
            HostBehavior::SwdYijue => self.trigger_meta_swd(),
            // 荡平：自己造成伤害结算后（origin=source damageAfter 同口径；
            // 每回合一次走 limits）
            HostBehavior::SwdDangping => self.trigger_meta_swd(),
            // 幻形：自己准备阶段开始（direct 非 optional）
            HostBehavior::SwdHuanxing => self.trigger_meta_swd(),
            // 幻形·移除面：自己受伤结算后（forced）
            HostBehavior::SwdHuanxing2 => self.trigger_meta_swd(),
            // 命符·姜子牙面：出牌主动技（usable:1 走 limits）
            HostBehavior::SwdMingfu => self.trigger_meta_swd(),
            // 仙匠·车芸面：使用牌指定唯一目标后（CardUsed；frequent 面=
            // direct optional 确认门）
            HostBehavior::SwdCyxianjiang => self.trigger_meta_swd(),
            // 玄影·端蒙面：自己受伤结算后（damageEnd 反击面——已横置且
            // 来源存活可反击；sha 面经 ActivateSkill）
            HostBehavior::SwdXuanying => self.trigger_meta_swd(),
            HostBehavior::SwdXuanyingDamage => self.trigger_meta_swd(),
            // 烈刃：自己杀造成伤害结算后（每触发与目标拼点，optional）
            HostBehavior::SwdLieren => self.trigger_meta_swd(),
            // 极谋·姬良面：自己受伤结算后（回合外门在 filter；frequent=非
            // optional）
            HostBehavior::SwdJqimou => self.trigger_meta_swd(),
            // 剑击·少微面：出牌主动技（usable:1 走 limits）
            HostBehavior::SwdJianji => self.trigger_meta_swd(),
            // 头弹·麦星面：出牌主动技（usable:1 走 limits）
            HostBehavior::SwdToudan => self.trigger_meta_swd(),
            // 黑火：出牌主动技（ActivateSkill 通道，usable:1 走 limits）
            HostBehavior::SwdHeihuo => self.trigger_meta_swd(),
            // 真威：其他角色响应结算后（确认门=optional）
            HostBehavior::SwdFzhenwei => self.trigger_meta_swd(),
            // 遁行：成为其他角色卡牌目标（确认门=optional；CardUsed 帧
            // 目标侧）
            HostBehavior::SwdDunxing => self.trigger_meta_swd(),
            // 孵鸟：出牌主动技（ActivateSkill 通道，usable:1 走 limits）
            HostBehavior::SwdFuniao => self.trigger_meta_swd(),
            // 凝弦：自己受伤结算后（确认门=optional）
            HostBehavior::SwdNingxian => self.trigger_meta_swd(),
            // 妖童：锁定 marker（engine viewAs 链查询），无触发面
            HostBehavior::SwdYaotong => self.trigger_meta_swd(),
            // 拷贝栏：锁定（Draw 内联钩），无触发面
            HostBehavior::SwdKbolan => self.trigger_meta_swd(),
            // 奥剑：出牌主动技（ActivateSkill 通道，usable:1 走 limits）
            HostBehavior::SwdAojian => self.trigger_meta_swd(),
            // 化金：出牌主动技（ActivateSkill 通道，usable:1 走 limits）
            HostBehavior::SwdHuajin => self.trigger_meta_swd(),
            // 化金触发面：伤害前置 +1；准备阶段开始自清（extra PhaseBegin）
            HostBehavior::SwdHuajin2 => self.trigger_meta_swd(),
            // 明镜：自己受到伤害前置（无防具 + 1/3）
            HostBehavior::SwdMinjing => self.trigger_meta_swd(),
            // 兰芷：使用梅花牌后（上游含 check 可放弃→原生恒发动近似）
            HostBehavior::SwdLanzhi => self.trigger_meta_swd(),
            // 热血：任意角色阶段开始（global；目标在 machine() 提取）
            HostBehavior::SwdRexue => self.trigger_meta_swd(),
            // 幻音：他人对你使用牌（上游 useCardToBefore target，priority 5.9）
            HostBehavior::SwdHuanyin => self.trigger_meta_swd(),
            // 灵弦：自己使用或打出牌（上游 respond/useCard；非自己回合）
            HostBehavior::SwdLingxian => self.trigger_meta_swd(),
            // 青木：死亡前置 First（上游 dieBegin hide）
            HostBehavior::BossQingmu2 => self.trigger_meta_common(),
            // 青木·转阶：死亡前置 Normal priority -10（上游 dieAfter）
            HostBehavior::BossQingmu2x => self.trigger_meta_common(),
            HostBehavior::BossQingmu3 => self.trigger_meta_common(),
            HostBehavior::BossQingmu3x => self.trigger_meta_common(),
            // 白芒/赤炎/玄麟 2/3 + 后援3/4：同青木 dieBegin First
            HostBehavior::BossBaimang2
            | HostBehavior::BossChiyan2
            | HostBehavior::BossXuanlin2
            | HostBehavior::BossBaimang3
            | HostBehavior::BossChiyan3
            | HostBehavior::BossXuanlin3
            | HostBehavior::BossBianshen3
            | HostBehavior::BossBianshen4 => self.trigger_meta_common(),
            // 白芒/赤炎/玄麟 2x/3x + 后援3/4·转阶：同青木·转阶 dieAfter Normal -10
            HostBehavior::BossBaimang2x
            | HostBehavior::BossChiyan2x
            | HostBehavior::BossXuanlin2x
            | HostBehavior::BossBaimang3x
            | HostBehavior::BossChiyan3x
            | HostBehavior::BossXuanlin3x
            | HostBehavior::BossBianshen3x
            | HostBehavior::BossBianshen4x => self.trigger_meta_common(),
            // 还人：任意阶段结束即开局（原生无 gameStart 帧）
            HostBehavior::BossHuanren => self.trigger_meta_common(),
            // 还人·转关：同隐藏链 dieAfter
            HostBehavior::BossHuanrenDie => self.trigger_meta_common(),
            // 青翼/赤翼：准备阶段锚 = 判定阶段开始
            HostBehavior::BossQingyi | HostBehavior::BossChiyi => self.trigger_meta_common(),
            // 赤仪2：伤害前置 +1
            HostBehavior::BossChiyiMark => self.trigger_meta_common(),
            // 白翼弃牌：第 5 轮判定阶段开始（roundStart 近似）
            HostBehavior::BossBaiyi => self.trigger_meta_common(),
            HostBehavior::BossBaiyiThunder => self.trigger_meta_common(),
            // 玄咒：出牌主动技（ActivateSkill 通道，usable:1 走 limits）
            HostBehavior::SwdXuanzhou => self.trigger_meta_swd(),
            // 因果：他人伤害前置（确认门=optional；priority 6 同上游）
            HostBehavior::SwdYinguo => self.trigger_meta_swd(),
            // 唤魂：任意角色濒死（确认门=optional；priority 6 同上游）
            HostBehavior::SwdHuanhun => self.trigger_meta_swd(),
            // 倾城：自己结束阶段（frequent 锁定式=非 optional）
            HostBehavior::SwdLqingcheng => self.trigger_meta_swd(),
            // 狩猎：可选（Sha 机器内联臂承载——filter/machine 无触发面）
            HostBehavior::SwdShoulie => self.trigger_meta_swd(),
            // 不死：自己濒死（锁定强制；priority 7 先于鬼眼复活面/救援）
            HostBehavior::SwdBusi => self.trigger_meta_swd(),
            // 破阵：自己受伤结算后（确认门=optional）
            HostBehavior::SwdPozhen => self.trigger_meta_swd(),
            // 鬼眼·复活面：自己濒死（锁定强制；priority 6 同上游先于救援）
            HostBehavior::SwdGuiyan2 => self.trigger_meta_swd(),
            // 悲歌：任意角色受伤结算后（global；确认门=optional）
            HostBehavior::YunchouBeige => self.trigger_meta_common(),
            // 复仇·清算：自己回合结束（标记逐个结算）
            HostBehavior::WudianFuchou2 => self.trigger_meta_wudian(),
            // 蝉缘·变体：死亡结算后（来源减上限）
            HostBehavior::YunchouDiychanyuan => self.trigger_meta_common(),
            HostBehavior::YunchouLuweiyan2 => self.trigger_meta_common(),
            // 蛊惑·变体：回合（判定阶段）开始
            HostBehavior::YunchouDiyguhuo => self.trigger_meta_common(),
            // 狡黠：成为牌目标（CardUsed+targets，frequent 自动）
            HostBehavior::YunchouJiaoxia => self.trigger_meta_common(),
            HostBehavior::YunchouHonglian => self.trigger_meta_common(),
            HostBehavior::TargetDiscardEquipOrDamage => self.trigger_meta_common(),
            HostBehavior::ShengkeCheck => self.trigger_meta_common(),
            HostBehavior::ChangeCardNature => self.trigger_meta_common(),
            HostBehavior::MaxRangeWeaponDamage => self.trigger_meta_common(),
            // 末焱：判定阶段开始（一轮一次）+ 伤害前置增幅
            HostBehavior::D3Moyan => self.trigger_meta_common(),
            // 穿云 / 天陨（结束阶段开始）+ 侠行（准备阶段开始 / 濒死时）
            HostBehavior::BossChuanyun
            | HostBehavior::BossTianyun
            | HostBehavior::DddXiaxing => self.trigger_meta_common(),
            HostBehavior::WangmeiZhikeCard => self.trigger_meta_common(),
            HostBehavior::SijiDiscardDraw => self.trigger_meta_common(),
            HostBehavior::SanjianWeaponDamage => self.trigger_meta_common(),
            HostBehavior::MoshouImmunity => self.trigger_meta_common(),
            HostBehavior::ShuigongCard => self.trigger_meta_common(),
            HostBehavior::FudichouxinCompare => self.trigger_meta_common(),
            HostBehavior::ChenhuodajieCard => self.trigger_meta_common(),
            HostBehavior::TouliangSwap => self.trigger_meta_common(),
            HostBehavior::LinghunzhihuoCard => self.trigger_meta_common(),
            HostBehavior::ZhaomingdanCard => self.trigger_meta_common(),
            HostBehavior::JihuocardDraw => self.trigger_meta_common(),
            HostBehavior::GeanguanhuoCompare => self.trigger_meta_common(),
            HostBehavior::CaochuanCard => self.trigger_meta_common(),
            HostBehavior::ShezhanqunruCompare => self.trigger_meta_common(),
            HostBehavior::GujianShujinsanCard => self.trigger_meta_common(),
            HostBehavior::GujianMianliCard => self.trigger_meta_common(),
            HostBehavior::GujianFeibiaoCard => self.trigger_meta_common(),
            HostBehavior::JinlianzhuCancel => self.trigger_meta_common(),
            HostBehavior::GujianFoodTanhuadong => self.trigger_meta_common(),
            HostBehavior::GujianFoodYougeng => self.trigger_meta_common(),
            HostBehavior::GujianFoodGrant => self.trigger_meta_common(),
            HostBehavior::GujianFoodQingtuan => self.trigger_meta_common(),
            HostBehavior::GujianFoodYuanbaorou => self.trigger_meta_common(),
            HostBehavior::GujianFoodGudonggeng => self.trigger_meta_common(),
            HostBehavior::GujianFoodMapodoufu => self.trigger_meta_common(),
            HostBehavior::GujianFoodChunbing => self.trigger_meta_common(),
            HostBehavior::GujianFoodLiyutang => self.trigger_meta_common(),
            HostBehavior::GujianFoodMolicha => self.trigger_meta_common(),
            HostBehavior::GujianFoodXiajiao => self.trigger_meta_common(),
            HostBehavior::GujianFoodMizhilianou => self.trigger_meta_common(),
            HostBehavior::GujianFoodLuyugeng => self.trigger_meta_common(),
            HostBehavior::JiguanBingpoCard => self.trigger_meta_common(),
            HostBehavior::JiguanQiankunCard => self.trigger_meta_common(),
            HostBehavior::JiguanLongxuCard => self.trigger_meta_common(),
            HostBehavior::JiguanShenhuoCard => self.trigger_meta_common(),
            HostBehavior::GujianFoodZiyangdan => self.trigger_meta_common(),
            HostBehavior::GujianYunvyuanshen => self.trigger_meta_common(),
            HostBehavior::GujianFoodShihuifenBan => self.trigger_meta_common(),
            HostBehavior::GujianShihuifenCard => self.trigger_meta_common(),
            HostBehavior::GujianLiufengsanCard => self.trigger_meta_common(),
            HostBehavior::JiguanShatangCard => self.trigger_meta_common(),
            HostBehavior::JiguanHeilongCard => self.trigger_meta_common(),
            HostBehavior::JiguanLiutougeCard => self.trigger_meta_common(),
            HostBehavior::GujianMutouEquipSkill => self.trigger_meta_common(),
            HostBehavior::JiguanDinvCard => self.trigger_meta_common(),
            HostBehavior::GujianYuhengEquipSkill => self.trigger_meta_common(),
            HostBehavior::HearthDunpaiCard => self.trigger_meta_common(),
            HostBehavior::HearthZhiliaoboCard => self.trigger_meta_common(),
            HostBehavior::HearthYuansuCard => self.trigger_meta_common(),
            HostBehavior::HearthXingjieCard => self.trigger_meta_common(),
            HostBehavior::HearthChuansongCard => self.trigger_meta_common(),
            HostBehavior::HearthTansheCard => self.trigger_meta_common(),
            HostBehavior::HearthShijieshuCard => self.trigger_meta_common(),
            HostBehavior::GwentYanzhiCard => self.trigger_meta_common(),
            HostBehavior::GwentAozuCard => self.trigger_meta_common(),
            HostBehavior::GwentWenyiCard => self.trigger_meta_common(),
            HostBehavior::GwentZhihuanCard => self.trigger_meta_common(),
            HostBehavior::GwentBaoxueCard => self.trigger_meta_common(),
            HostBehavior::GwentQinpenState => self.trigger_meta_common(),
            HostBehavior::GwentBirinState => self.trigger_meta_common(),
            HostBehavior::GwentCiguState => self.trigger_meta_common(),
            HostBehavior::GwentWeatherGrant => self.trigger_meta_common(),
            HostBehavior::GwentKunenState => self.trigger_meta_common(),
            HostBehavior::GwentKunenCard => self.trigger_meta_common(),
            HostBehavior::GwentShanbengCard => self.trigger_meta_common(),
            HostBehavior::GwentPoxiaoCard => self.trigger_meta_common(),
            HostBehavior::GwentFengyinState => self.trigger_meta_common(),
            // 机关凤封印：拥有者下一阶段开始解除（上游 addTempSkill
            // {player:'phaseBegin'} 同口径）
            HostBehavior::SwdFengyinState => self.trigger_meta_swd(),
            // 机关怨待获得：拥有者结束阶段强制获得置牌
            HostBehavior::SwdJiguanyuanGain => self.trigger_meta_swd(),
            HostBehavior::SwdJiguanfengCard => self.trigger_meta_swd(),
            HostBehavior::SwdJiguanyuanCard => self.trigger_meta_swd(),
            // 机关妖术防重标记：纯状态，无触发
            HostBehavior::SwdJiguanyaoshuCard => self.trigger_meta_swd(),
            HostBehavior::SwdJiguanyaoState => self.trigger_meta_swd(),
            HostBehavior::SwdJiguanshuCard => self.trigger_meta_swd(),
            HostBehavior::SwdTianfuzhenCard => self.trigger_meta_swd(),
            HostBehavior::SwdQixingzhenCard => self.trigger_meta_swd(),
            HostBehavior::SwdNiaoxiangzhenCard => self.trigger_meta_swd(),
            HostBehavior::SwdChangshezhenCard => self.trigger_meta_swd(),
            HostBehavior::SwdPozhenjueCard => self.trigger_meta_swd(),
            // 机关妖术补偿：弃牌事件（含装备牌时触发，登记近似差异）
            HostBehavior::SwdJiguanyaoGain => self.trigger_meta_swd(),
            HostBehavior::SwdXuanfengzhirenCard => self.trigger_meta_swd(),
            HostBehavior::SwdZhongxinghujiaCard => self.trigger_meta_swd(),
            HostBehavior::SwdXingtigaizaoCard => self.trigger_meta_swd(),
            HostBehavior::SwdShijianhuisuCard => self.trigger_meta_swd(),
            HostBehavior::SwdJinjilengdongCard => self.trigger_meta_swd(),
            HostBehavior::SwdShengxiuhaojiaoCard => self.trigger_meta_swd(),
            HostBehavior::SwdYinmilichangCard => self.trigger_meta_swd(),
            HostBehavior::GwentZumoCard => self.trigger_meta_common(),
            HostBehavior::HuanleMonkeyCard => self.trigger_meta_common(),
            HostBehavior::HuanleXuelunyangCard => self.trigger_meta_common(),
            HostBehavior::HuanleMianjuCard => self.trigger_meta_common(),
            HostBehavior::ZhangbaSha => self.trigger_meta_common(),
            HostBehavior::SwdLiuxinghuoyuCard => self.trigger_meta_swd(),
            HostBehavior::SwdShentouEquipSkill => self.trigger_meta_swd(),
            HostBehavior::SwdYufuluEquipSkill => self.trigger_meta_swd(),
            HostBehavior::SwdYangpijuanCard => self.trigger_meta_swd(),
            HostBehavior::SwdLangeguaiyiEquipSkill => self.trigger_meta_swd(),
            HostBehavior::MtgLongshanfengLand => self.trigger_meta_common(),
            HostBehavior::MtgHaidaoLand => self.trigger_meta_common(),
            HostBehavior::MtgShuimomuxueLand => self.trigger_meta_common(),
            HostBehavior::MtgFeixuLand => self.trigger_meta_common(),
            HostBehavior::MtgYixialanLand => self.trigger_meta_common(),
            HostBehavior::MtgYixialanRecast => self.trigger_meta_common(),
            HostBehavior::MtgShamolvzhouLand => self.trigger_meta_common(),
            HostBehavior::MtgLinzhongjiantaLand => self.trigger_meta_common(),
            HostBehavior::MtgLinzhongjiantaStrike => self.trigger_meta_common(),
            HostBehavior::MtgBingheyaosaiLand => self.trigger_meta_common(),
            HostBehavior::MtgYoulinLand => self.trigger_meta_common(),
            HostBehavior::MtgYoulinDiscard => self.trigger_meta_common(),
            HostBehavior::MtgLindixiliuLand => self.trigger_meta_common(),
            HostBehavior::WudianCiqiu => self.trigger_meta_wudian(),
            HostBehavior::WudianShuaiyan => self.trigger_meta_wudian(),
            HostBehavior::WudianZhucheng => self.trigger_meta_wudian(),
            HostBehavior::WudianZhuchengGuard => self.trigger_meta_wudian(),
            HostBehavior::WudianKangyin => self.trigger_meta_wudian(),
            HostBehavior::WudianChezhen => self.trigger_meta_wudian(),
            HostBehavior::WudianJuedao => self.trigger_meta_wudian(),
            HostBehavior::WudianGeju => self.trigger_meta_wudian(),
            HostBehavior::WudianTuji => self.trigger_meta_wudian(),
            HostBehavior::WudianMujing => self.trigger_meta_wudian(),
            HostBehavior::WudianGuangshu => self.trigger_meta_wudian(),
            HostBehavior::WudianGuangshuHeart => self.trigger_meta_wudian(),
            HostBehavior::WudianGuangshuDiamond => self.trigger_meta_wudian(),
            HostBehavior::WudianGuangshuClub => self.trigger_meta_wudian(),
            HostBehavior::WudianGuangshuSpade => self.trigger_meta_wudian(),
            HostBehavior::WudianZiyu => self.trigger_meta_wudian(),
            HostBehavior::WudianShouhu => self.trigger_meta_wudian(),
            HostBehavior::WudianDuwen => self.trigger_meta_wudian(),
            HostBehavior::WudianPaotai => self.trigger_meta_wudian(),
            HostBehavior::WudianPaotaiFire => self.trigger_meta_wudian(),
            HostBehavior::WudianBaoleiSwitch => self.trigger_meta_wudian(),
            HostBehavior::WudianZhenchaDraw => self.trigger_meta_wudian(),
            HostBehavior::WudianYuedong => self.trigger_meta_wudian(),
            HostBehavior::WudianKuoyin => self.trigger_meta_wudian(),
            HostBehavior::WudianHuhuan => self.trigger_meta_wudian(),
            HostBehavior::WudianJidong => self.trigger_meta_wudian(),
            HostBehavior::WudianBaoxue => self.trigger_meta_wudian(),
            HostBehavior::WudianLiangou => self.trigger_meta_wudian(),
            HostBehavior::WudianXiyang => self.trigger_meta_wudian(),
            HostBehavior::WudianFeiren => self.trigger_meta_wudian(),
            HostBehavior::WudianFeidan => self.trigger_meta_wudian(),
            HostBehavior::WudianZhiyuan => self.trigger_meta_wudian(),
            HostBehavior::WudianShenqiang => self.trigger_meta_wudian(),
            HostBehavior::WudianShihuo => self.trigger_meta_wudian(),
            HostBehavior::WudianMengji => self.trigger_meta_wudian(),
            HostBehavior::WudianYinbo => self.trigger_meta_wudian(),
            HostBehavior::WudianQianggu => self.trigger_meta_wudian(),
            HostBehavior::WudianQiangguGuard => self.trigger_meta_wudian(),
            HostBehavior::WudianFeitiao => self.trigger_meta_wudian(),
            HostBehavior::WudianPingzhang => self.trigger_meta_wudian(),
            HostBehavior::WudianOwliyong => self.trigger_meta_wudian(),
            HostBehavior::WudianYinshen => self.trigger_meta_wudian(),
            HostBehavior::WudianXiandan => self.trigger_meta_wudian(),
            HostBehavior::WudianJijia => self.trigger_meta_wudian(),
            HostBehavior::XianjianLingxue => self.trigger_meta_xianjian(),
            HostBehavior::XianjianBinxin => self.trigger_meta_xianjian(),
            HostBehavior::XianjianHuimeng => self.trigger_meta_xianjian(),
            HostBehavior::XianjianTianshe => self.trigger_meta_xianjian(),
            HostBehavior::XianjianTianshe2 => self.trigger_meta_xianjian(),
            HostBehavior::XianjianFanling => self.trigger_meta_xianjian(),
            HostBehavior::WudianLongxiang => self.trigger_meta_wudian(),
            HostBehavior::XianjianGuiyuan => self.trigger_meta_xianjian(),
            HostBehavior::XianjianTannang => self.trigger_meta_xianjian(),
            HostBehavior::XianjianFeixu => self.trigger_meta_xianjian(),
            HostBehavior::XianjianFeng => self.trigger_meta_xianjian(),
            HostBehavior::XianjianYa => self.trigger_meta_xianjian(),
            HostBehavior::XianjianSong => self.trigger_meta_xianjian(),
            HostBehavior::XianjianXjyufeng => self.trigger_meta_xianjian(),
            HostBehavior::XianjianXuanyan2 => self.trigger_meta_xianjian(),
            HostBehavior::XianjianXuanyan3 => self.trigger_meta_xianjian(),
            HostBehavior::XianjianNingbin => self.trigger_meta_xianjian(),
            HostBehavior::XianjianXuanyan => self.trigger_meta_xianjian(),
            HostBehavior::XianjianDiewu => self.trigger_meta_xianjian(),
            HostBehavior::XianjianLingyu => self.trigger_meta_xianjian(),
            HostBehavior::WudianLueying => self.trigger_meta_wudian(),
            HostBehavior::XianjianAnwugu => self.trigger_meta_xianjian(),
            HostBehavior::XianjianAnwugu2 => self.trigger_meta_xianjian(),
            // 甜香去：伤害前置窗口（上游 source damageBefore check 询问式）
            HostBehavior::XianjianTxianqu => self.trigger_meta_xianjian(),
            HostBehavior::XianjianFenshi => self.trigger_meta_xianjian(),
            HostBehavior::XianjianLonghuo => self.trigger_meta_xianjian(),
            HostBehavior::XianjianSajin => self.trigger_meta_xianjian(),
            // 聚宝：global 弃牌（上游 discardAfter frequent=自动无确认）
            HostBehavior::XianjianJtjubao => self.trigger_meta_xianjian(),
            HostBehavior::XianjianYuexing => self.trigger_meta_xianjian(),
            HostBehavior::XianjianMinsha => self.trigger_meta_xianjian(),
            // 聚宝：global 弃牌（上游 discardAfter frequent=自动无确认）
            HostBehavior::XianjianLinghuo => self.trigger_meta_xianjian(),
            HostBehavior::XianjianGuijin => self.trigger_meta_xianjian(),
            HostBehavior::XianjianZhuyue => self.trigger_meta_xianjian(),
            HostBehavior::XianjianGuanri => self.trigger_meta_xianjian(),
            HostBehavior::XianjianShuiyun => self.trigger_meta_xianjian(),
            HostBehavior::XianjianShuiyun5 => self.trigger_meta_xianjian(),
            HostBehavior::XianjianWangyou => self.trigger_meta_xianjian(),
            HostBehavior::XianjianWangyou2 => self.trigger_meta_xianjian(),
            HostBehavior::XianjianLingquan => self.trigger_meta_xianjian(),
            HostBehavior::XianjianShenwu => self.trigger_meta_xianjian(),
            HostBehavior::XianjianChengxin => self.trigger_meta_xianjian(),
            HostBehavior::XianjianShenmu => self.trigger_meta_xianjian(),
            HostBehavior::XianjianDuci => self.trigger_meta_xianjian(),
            HostBehavior::XianjianRunxin => self.trigger_meta_xianjian(),
            HostBehavior::XianjianXjzhimeng => self.trigger_meta_xianjian(),
            HostBehavior::XianjianXjzhimeng2 => self.trigger_meta_xianjian(),
            HostBehavior::XianjianSheling => self.trigger_meta_xianjian(),
            HostBehavior::XianjianHushaCount => self.trigger_meta_xianjian(),
            HostBehavior::XianjianHusha => self.trigger_meta_xianjian(),
            HostBehavior::XianjianChangnian => self.trigger_meta_xianjian(),
            HostBehavior::XianjianChangnian2 => self.trigger_meta_xianjian(),
            HostBehavior::XianjianShiying => self.trigger_meta_xianjian(),
            HostBehavior::XianjianZhaoyao => self.trigger_meta_xianjian(),
            HostBehavior::XianjianLiguang => self.trigger_meta_xianjian(),
            HostBehavior::XianjianXuanmo => self.trigger_meta_xianjian(),
            HostBehavior::XianjianSheying => self.trigger_meta_xianjian(),
            HostBehavior::XianjianHuahu => self.trigger_meta_xianjian(),
            HostBehavior::XianjianXtanxi => self.trigger_meta_xianjian(),
            HostBehavior::XianjianFenglue => self.trigger_meta_xianjian(),
            HostBehavior::XianjianLiefeng => self.trigger_meta_xianjian(),
            HostBehavior::XianjianTianjian => self.trigger_meta_xianjian(),
            HostBehavior::XianjianPoyun => self.trigger_meta_xianjian(),
            HostBehavior::XianjianXshuangren => self.trigger_meta_xianjian(),
            HostBehavior::XianjianXshuangren2 => self.trigger_meta_xianjian(),
            HostBehavior::XianjianZhangmu => self.trigger_meta_xianjian(),
            HostBehavior::XianjianQianfang => self.trigger_meta_xianjian(),
            HostBehavior::XianjianXunying => self.trigger_meta_xianjian(),
            HostBehavior::XianjianTianxian => self.trigger_meta_xianjian(),
            HostBehavior::XianjianXjyingfeng => self.trigger_meta_xianjian(),
            HostBehavior::XianjianXiepan => self.trigger_meta_xianjian(),
            HostBehavior::XianjianLuanjian => self.trigger_meta_xianjian(),
            // 乱剑·增伤：杀伤害前置窗（上游 source damageBegin 同口径）
            HostBehavior::XianjianLuanjian2 => self.trigger_meta_xianjian(),
            HostBehavior::XianjianCtianfu => self.trigger_meta_xianjian(),
            // 天符·反噬：伤害结算后（上游 source damageAfter=Damage 帧
            // AfterTriggers）
            HostBehavior::XianjianCtianfu2 => self.trigger_meta_xianjian(),
            HostBehavior::XianjianCtianfu3 => self.trigger_meta_xianjian(),
            // swd 机关牌族（v189-v192）：卡牌机器（无懈窗包装，intrinsic 分派——无触发）
            HostBehavior::SwdZhiluxiaohuCard => self.trigger_meta_swd(),
            HostBehavior::SwdMujiarenCard => self.trigger_meta_swd(),
            HostBehavior::SwdGouhunluoCard => self.trigger_meta_swd(),
            // 勾魂锣·倒计时：来源回合开始（global phaseBegin 由 filter 定向）
            HostBehavior::SwdGouhunluoSub => self.trigger_meta_swd(),
            HostBehavior::SwdJiguantongCard => self.trigger_meta_swd(),
        };
        if let (Some(meta), Some((timing, priority, selectable))) = (&mut meta, self.source_order()) {
            meta.timing = timing;
            meta.priority = priority;
            meta.order_selectable = selectable;
        }
        meta
    }
}
