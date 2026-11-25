//! 37 来源（46 扩展包）八维仓内签署补全测试入口。
//!
//! 覆盖原八项在仓内可验证的全部维度：
//! - ① 签名验真、安装/幂等、停用/重启持久化/启用、升级/回滚、卸载/重启持久化；
//! - ② 真实包配置与资产解析、负例拒错；
//! - ③ 依赖声明、宿主版本门禁、被依赖包停用/卸载保护、缺失依赖拒绝；
//! - ④ 多包同店共存与同局建局/推进/牌守恒/反向加载序；
//! - ⑤ `ContentLock` 对局保存恢复、错版拒恢复、升级与物理卸载后旧锁无损恢复；
//! - ⑥/⑦/⑧ 配合 `nc12_bench`、跨端/桌面烟测与规则基准差异登记完成逐来源复核。

#[path = "../src/native.rs"]
mod native;
#[path = "../src/native_room_store.rs"]
mod native_room_store;
#[path = "../src/native_view.rs"]
mod native_view;
#[path = "../src/persist.rs"]
mod persist;
#[path = "../src/room.rs"]
mod room;
#[path = "../src/room_native.rs"]
mod room_native;
#[path = "../src/room_native_actor.rs"]
mod room_native_actor;
#[path = "../src/session.rs"]
mod session;

#[path = "up37_eight_dim_signoff/harness.rs"]
mod harness;
#[path = "up37_eight_dim_signoff/wave1.rs"]
mod wave1;
#[path = "up37_eight_dim_signoff/wave2.rs"]
mod wave2;
