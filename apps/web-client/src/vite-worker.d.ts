/// Vite `?worker` 导入的类型声明（构建期产出独立 worker chunk）。
declare module "*?worker" {
  const WorkerFactory: new () => Worker;
  export default WorkerFactory;
}
