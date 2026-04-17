# oxo-call LangGraph 架构实施计划

## Phase 1: 核心架构重构（优先级：高）

### 1.1 Task Complexity Estimator（任务复杂度评估器）
- **文件**: `src/task_complexity.rs`
- **功能**: 分析用户输入，自动决定 Fast/Quality 模式
- **预计时间**: 2-3 小时

### 1.2 Task Normalizer（任务标准化器）
- **文件**: `src/task_normalizer.rs`
- **功能**: 将中文/模糊描述转换为标准化英文任务
- **预计时间**: 3-4 小时

### 1.3 Intelligent Doc Processor（智能文档处理器）
- **文件**: 重构 `src/doc_processor.rs`
- **功能**: 使用 LLM 智能清理文档，提取关键信息
- **预计时间**: 4-5 小时

### 1.4 Workflow Graph（工作流图）
- **文件**: `src/workflow_graph.rs`
- **功能**: DAG 编排多个 LLM 调用
- **预计时间**: 5-6 小时

## Phase 2: 高级功能（优先级：中）

### 2.1 Web Retrieval Module（URL 检索模块）
- **文件**: `src/web_retrieval.rs`
- **功能**: 从用户提及的 URL 获取最新文档
- **预计时间**: 3-4 小时

### 2.2 Mini-skill Evolution System（自进化系统）
- **文件**: `src/skill_evolution.rs`
- **功能**: 根据用户反馈改进 Mini-skill
- **预计时间**: 4-5 小时

### 2.3 Adaptive Config（自适应配置）
- **文件**: `src/adaptive_config.rs`
- **功能**: 动态调整工作流参数
- **预计时间**: 2-3 小时

### 2.4 Performance Optimization（性能优化）
- **功能**: 并行 LLM 调用、流式输出、缓存预热
- **预计时间**: 4-5 小时

## Phase 3: 测试与优化（优先级：高）

### 3.1 Comprehensive Benchmark
- **文件**: `oxo-call-test/bench/comprehensive_bench.sh`
- **功能**: 全面测试准确率、性能、鲁棒性
- **预计时间**: 3-4 小时

### 3.2 Real User Testing
- **功能**: 收集真实用户反馈
- **预计时间**: 持续进行

---

## 当前状态

- [x] Phase 1.1: Task Complexity Estimator ✅ (2026-04-17)
- [x] Phase 1.2: Task Normalizer ✅ (2026-04-17)
- [x] Phase 1.3: Intelligent Doc Processor ✅ (2026-04-17)
- [x] Phase 1.4: Workflow Graph ✅ (2026-04-17)
- [ ] Phase 2.x: 高级功能
- [ ] Phase 3.x: 测试与优化

---

## 开始实施

**Phase 1 全部完成！** ✅

**测试结果**: 10/12 通过 (83.33% 成功率)

下一步：集成到 runner，进行端到端测试
