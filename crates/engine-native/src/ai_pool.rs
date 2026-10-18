//! NC10：**长驻有界规划工作池**（替代逐调用 `std::thread::scope` 的线程创建）。
//!
//! 方案 §NC10 要求：「将逐调用创建线程并同步等待的原型收敛为产品宿主中的长驻有界
//! 工作队列，限制并发、内存和排队，支持取消、降级与过期结果拒绝」。
//!
//! 设计（std-only，无新依赖）：
//! - **长驻**：`PlannerPool::new(workers, queue_cap)` 一次创建 `workers` 个工作线程，
//!   复用至 `shutdown`；不再每次调用创建线程。
//! - **有界**：任务通道容量 `queue_cap`；提交用 `try_send`，队列满**立即返回 `Err`**
//!   由调用方**降级**（例如退回单座位串行规划），不阻塞规则主通道。
//! - **取消**：沿用 `BoundedPlanner.cancel`（节点边界检查）；池不额外阻塞取消。
//! - **过期拒绝**：产物 `Plan.revision` 携带规划时修订号，提交侧按修订号拒绝
//!   （既有机制，池不改语义）。
//! - **内存**：任务为 `FnOnce + Send` 闭包，需 `'static` → 每个座位任务**持有一份
//!   引擎克隆**（1.3–1.5 ms / 数百 KB，见 NC12 基线）。这是本设计的**明确代价**，
//!   与「共享 `&Engine` + scoped 线程」相比换取长驻与安全；登记为待办（可改为
//!   `Arc` 共享状态或专用工作线程持有引擎）。

use crate::ai::{BoundedPlanner, Plan};
use crate::Engine;
use std::sync::mpsc::{sync_channel, SyncSender};
use std::sync::Mutex;

/// 池已满（调用方应降级）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PoolFull;

type Job = Box<dyn FnOnce() + Send + 'static>;

/// 长驻有界工作池。
pub struct PlannerPool {
    tx: Option<SyncSender<Job>>,
    workers: usize,
    queue_cap: usize,
    handles: Mutex<Vec<std::thread::JoinHandle<()>>>,
}

impl PlannerPool {
    /// 创建池：`workers` 个长驻线程 + 容量 `queue_cap` 的任务队列。
    pub fn new(workers: usize, queue_cap: usize) -> Self {
        let workers = workers.max(1);
        let queue_cap = queue_cap.max(1);
        let (tx, rx) = sync_channel::<Job>(queue_cap);
        let rx = std::sync::Arc::new(Mutex::new(rx));
        let mut handles = Vec::with_capacity(workers);
        for _ in 0..workers {
            let rx = rx.clone();
            handles.push(std::thread::spawn(move || loop {
                let job = {
                    let guard = rx.lock().expect("pool rx");
                    guard.recv()
                };
                match job {
                    Ok(job) => job(),
                    Err(_) => break, // 发送端关闭 → 退出
                }
            }));
        }
        Self {
            tx: Some(tx),
            workers,
            queue_cap,
            handles: Mutex::new(handles),
        }
    }

    /// 工作线程数。
    pub fn workers(&self) -> usize {
        self.workers
    }

    /// 队列容量。
    pub fn queue_cap(&self) -> usize {
        self.queue_cap
    }

    /// 提交一个任务（非阻塞）：队列满返回 `Err(PoolFull)`，调用方据此降级。
    pub fn try_submit<F: FnOnce() + Send + 'static>(&self, f: F) -> Result<(), PoolFull> {
        self.tx
            .as_ref()
            .expect("pool open")
            .try_send(Box::new(f))
            .map_err(|_| PoolFull)
    }

    /// 规划若干座位：并发受池限制，队列满则**降级为串行**（同线程内规划），
    /// 不阻塞、不 panic。
    ///
    /// 返回 `(plans, degraded)`：`degraded = true` 表示本次发生降级。
    pub fn plan_all(
        &self,
        engine: &Engine,
        seats: &[u8],
        planner: &BoundedPlanner,
    ) -> (Vec<Plan>, bool) {
        if seats.is_empty() {
            return (Vec::new(), false);
        }
        let (result_tx, result_rx) = std::sync::mpsc::channel::<(usize, Plan)>();
        let mut submitted = 0usize;
        for (idx, seat) in seats.iter().enumerate() {
            let seat = *seat;
            let sim = engine.clone(); // 任务需 'static：每座位持一份克隆（见模块注释）
            let planner = planner.clone();
            let tx = result_tx.clone();
            match self.try_submit(move || {
                let plan = sim.plan_action(seat, &planner);
                let _ = tx.send((idx, plan));
            }) {
                Ok(()) => submitted += 1,
                Err(PoolFull) => break,
            }
        }
        drop(result_tx);
        let mut out: Vec<Option<Plan>> = vec![None; seats.len()];
        let mut got = 0usize;
        for _ in 0..submitted {
            if let Ok((idx, plan)) = result_rx.recv() {
                out[idx] = Some(plan);
                got += 1;
            }
        }
        let mut degraded = got < seats.len();
        // 降级：未提交的座位在本线程串行补齐
        for (idx, seat) in seats.iter().enumerate() {
            if out[idx].is_none() {
                out[idx] = Some(engine.plan_action(*seat, planner));
                degraded = true;
            }
        }
        (out.into_iter().flatten().collect(), degraded)
    }

    /// 关闭池（等待工作线程退出）。
    pub fn shutdown(mut self) {
        self.close_and_join();
    }

    fn close_and_join(&mut self) {
        self.tx = None; // 关闭通道 → 工作线程退出
        if let Ok(mut handles) = self.handles.lock() {
            for h in handles.drain(..) {
                let _ = h.join();
            }
        }
    }
}

impl Drop for PlannerPool {
    fn drop(&mut self) {
        self.close_and_join();
    }
}
