# AI Agent Implementation Guide (v1.8.0+)

**Feature Version**: v1.8.0 - v2.0.0+
**Status**: Architecture & Implementation Guide
**Last Updated**: 2025-11-18

---

## Agent Implementation Framework

### Base Agent Trait

```rust
// crates/clnrm-agents/src/agent.rs

#[async_trait]
pub trait Agent: Send + Sync + Debug {
    // Lifecycle
    async fn initialize(&self) -> Result<()>;
    async fn shutdown(&self) -> Result<()>;

    // Work execution
    async fn execute_task(&self, task: Task) -> Result<TaskResult>;

    // State management
    fn get_state(&self) -> AgentState;
    fn set_state(&self, state: AgentState) -> Result<()>;

    // Communication
    async fn send_message(&self, recipient: AgentId, msg: Message) -> Result<()>;
    async fn receive_message(&self) -> Result<Option<Message>>;
    async fn broadcast(&self, msg: Message) -> Result<()>;

    // Health
    async fn heartbeat(&self) -> Result<HeartbeatResponse>;
    fn get_metrics(&self) -> AgentMetrics;

    // Discovery
    async fn discover_peers(&self, role: AgentRole) -> Result<Vec<AgentId>>;
    async fn register_capability(&self, cap: Capability) -> Result<()>;
}

pub struct BaseAgent {
    id: AgentId,
    role: AgentRole,
    state: Arc<RwLock<AgentState>>,
    peer_registry: Arc<DashMap<AgentId, PeerInfo>>,
    message_queue: Arc<SegQueue<Message>>,
    metrics: Arc<AtomicMetrics>,
    logger: Arc<dyn Logger>,
}

impl BaseAgent {
    pub async fn new(role: AgentRole) -> Result<Self> {
        let id = AgentId::new();
        info!("Creating agent {} with role {:?}", id, role);

        Ok(Self {
            id,
            role,
            state: Arc::new(RwLock::new(AgentState::Initializing)),
            peer_registry: Arc::new(DashMap::new()),
            message_queue: Arc::new(SegQueue::new()),
            metrics: Arc::new(AtomicMetrics::new()),
            logger: create_logger(&id),
        })
    }
}
```

---

## Specific Agent Implementations

### 1. Test Discovery Agent

```rust
// crates/clnrm-agents/src/agents/discovery.rs

#[derive(Debug)]
pub struct TestDiscoveryAgent {
    base: BaseAgent,
    repo_scanner: Arc<RepositoryScanner>,
    semantic_analyzer: Arc<SemanticAnalyzer>,
    pattern_engine: Arc<PatternMatcher>,
    cache: Arc<DiscoveryCache>,
}

impl TestDiscoveryAgent {
    pub async fn new() -> Result<Self> {
        let base = BaseAgent::new(AgentRole::TestDiscoveryAgent).await?;
        Ok(Self {
            base,
            repo_scanner: Arc::new(RepositoryScanner::new()),
            semantic_analyzer: Arc::new(SemanticAnalyzer::new()),
            pattern_engine: Arc::new(PatternMatcher::new()),
            cache: Arc::new(DiscoveryCache::new()),
        })
    }

    pub async fn discover_tests(&self, path: &Path) -> Result<Vec<TestSuite>> {
        // Check cache first
        if let Some(cached) = self.cache.get(path).await {
            return Ok(cached);
        }

        // Phase 1: File-based discovery
        let file_tests = self.discover_file_based(path).await?;

        // Phase 2: Configuration-based discovery
        let config_tests = self.discover_config_based(path).await?;

        // Phase 3: Semantic analysis
        let semantic_tests = self.discover_semantic(path).await?;

        // Merge and deduplicate
        let mut all_tests = [file_tests, config_tests, semantic_tests].concat();
        all_tests.sort_by_key(|t| t.id.clone());
        all_tests.dedup_by_key(|t| t.id.clone());

        // Cache result
        self.cache.set(path, all_tests.clone()).await;

        self.base.metrics.record("tests_discovered", all_tests.len() as u64);

        Ok(all_tests)
    }

    async fn discover_file_based(&self, path: &Path) -> Result<Vec<TestSuite>> {
        // Detect test files by pattern:
        // - pytest: test_*.py, *_test.py
        // - cargo: tests/, benches/
        // - nodejs: __tests__/, *.test.js
        // - etc.

        let patterns = vec![
            "**/*_test.{rs,py,js,go,java}",
            "**/test_*.{rs,py,js,go,java}",
            "**/tests/**/*.{rs,py,js,go,java}",
            "**/__tests__/**/*.{js,ts}",
        ];

        let mut tests = Vec::new();
        for pattern in patterns {
            let files = self.repo_scanner.glob(path, pattern).await?;
            for file in files {
                let suite = self.parse_test_file(&file).await?;
                tests.push(suite);
            }
        }

        Ok(tests)
    }

    async fn discover_semantic(&self, path: &Path) -> Result<Vec<TestSuite>> {
        // Use LLM to understand code intent and suggest tests
        // This identifies implicit requirements that don't have tests yet

        let source_files = self.repo_scanner
            .glob(path, "**/*.{rs,py,js,go}")
            .await?;

        let mut implicit_tests = Vec::new();

        for file in source_files.take(10).await {  // Sample first 10 files
            let content = tokio::fs::read_to_string(&file).await?;

            // Analyze code structure
            let analysis = self.semantic_analyzer.analyze(&content).await?;

            // Suggest test scenarios
            let scenarios = analysis.extract_test_scenarios();

            implicit_tests.extend(scenarios);
        }

        Ok(implicit_tests)
    }

    async fn parse_test_file(&self, path: &Path) -> Result<TestSuite> {
        let content = tokio::fs::read_to_string(path).await?;

        // Detect language
        let lang = self.detect_language(path)?;

        // Parse tests based on language
        let tests = match lang {
            Language::Rust => self.parse_rust_tests(&content)?,
            Language::Python => self.parse_python_tests(&content)?,
            Language::JavaScript => self.parse_js_tests(&content)?,
            _ => vec![],
        };

        Ok(TestSuite {
            id: TestSuiteId::new(),
            source_file: path.to_path_buf(),
            language: lang,
            tests,
        })
    }
}

#[async_trait]
impl Agent for TestDiscoveryAgent {
    async fn execute_task(&self, task: Task) -> Result<TaskResult> {
        match task.task_type {
            TaskType::DiscoverTests { path } => {
                let tests = self.discover_tests(&path).await?;
                Ok(TaskResult::TestsDiscovered(tests))
            }
            _ => Err(CleanroomError::invalid_operation(
                "Unknown task type for TestDiscoveryAgent"
            ))
        }
    }

    // ... other trait methods
}
```

---

### 2. Test Generator Agent

```rust
// crates/clnrm-agents/src/agents/generator.rs

#[derive(Debug)]
pub struct TestGeneratorAgent {
    base: BaseAgent,
    llm_client: Arc<LLMClient>,
    template_library: Arc<TemplateLibrary>,
    validator: Arc<TestValidator>,
}

impl TestGeneratorAgent {
    pub async fn new(llm_api_key: String) -> Result<Self> {
        let base = BaseAgent::new(AgentRole::TestGeneratorAgent).await?;

        Ok(Self {
            base,
            llm_client: Arc::new(LLMClient::new(llm_api_key).await?),
            template_library: Arc::new(TemplateLibrary::load().await?),
            validator: Arc::new(TestValidator::new()),
        })
    }

    pub async fn generate_from_description(&self, description: &str) -> Result<Vec<Test>> {
        info!("Generating tests from description: {}", description);

        // Step 1: Parse intent using NLP
        let intent = self.parse_intent(description).await?;

        // Step 2: Generate test code via LLM
        let generated_code = self.llm_client
            .generate_test_code(&intent)
            .await?;

        // Step 3: Apply code generation templates
        let refined_code = self.template_library
            .apply_templates(&generated_code, &intent)
            .await?;

        // Step 4: Validate generated tests
        let tests = self.validator
            .validate(&refined_code)
            .await?;

        self.base.metrics.record("tests_generated", tests.len() as u64);

        Ok(tests)
    }

    pub async fn generate_edge_cases(&self, test: &Test) -> Result<Vec<Test>> {
        info!("Generating edge cases for test: {}", test.id);

        // Use LLM to identify edge cases
        let edge_cases = self.llm_client
            .identify_edge_cases(test)
            .await?;

        let mut generated = Vec::new();

        for edge_case in edge_cases {
            // Generate test for each edge case
            let code = self.llm_client
                .generate_test_code(&edge_case)
                .await?;

            // Validate
            if let Ok(test) = self.validator.validate(&code).await {
                generated.push(test);
            }
        }

        self.base.metrics.record("edge_cases_generated", generated.len() as u64);

        Ok(generated)
    }

    pub async fn generate_property_tests(&self, test: &Test) -> Result<Vec<Test>> {
        // Generate property-based tests (quickcheck, proptest)
        // These tests verify properties that should hold for all inputs

        let properties = self.extract_properties(test).await?;

        let mut property_tests = Vec::new();

        for property in properties {
            let code = self.generate_property_test_code(&property).await?;

            if let Ok(test) = self.validator.validate(&code).await {
                property_tests.push(test);
            }
        }

        Ok(property_tests)
    }

    async fn parse_intent(&self, description: &str) -> Result<TestIntent> {
        // Use LLM to parse test description into structured intent

        let prompt = format!(
            "Parse the following test description into structured intent:\n\n{}",
            description
        );

        let response = self.llm_client.query(&prompt).await?;

        // Extract structured data from response
        let intent = TestIntent::parse_from_response(&response)?;

        Ok(intent)
    }

    async fn generate_test_code(&self, intent: &TestIntent) -> Result<String> {
        // Multi-turn conversation with LLM to refine test code

        let prompt = self.template_library
            .get_test_generation_prompt(&intent)
            .await?;

        let mut code = self.llm_client.query(&prompt).await?;

        // Iteratively improve via feedback loop
        for _ in 0..3 {
            let validation = self.validator.validate(&code).await;

            if validation.is_ok() {
                break;
            }

            // Ask LLM to fix issues
            let fix_prompt = format!(
                "The generated test code has issues:\n\n{}\n\nPlease fix it:\n\n{}",
                validation.err().unwrap(),
                code
            );

            code = self.llm_client.query(&fix_prompt).await?;
        }

        Ok(code)
    }
}

#[async_trait]
impl Agent for TestGeneratorAgent {
    async fn execute_task(&self, task: Task) -> Result<TaskResult> {
        match task.task_type {
            TaskType::GenerateFromDescription { description } => {
                let tests = self.generate_from_description(&description).await?;
                Ok(TaskResult::TestsGenerated(tests))
            }
            TaskType::GenerateEdgeCases { test } => {
                let edge_cases = self.generate_edge_cases(&test).await?;
                Ok(TaskResult::EdgeCasesGenerated(edge_cases))
            }
            _ => Err(CleanroomError::invalid_operation(
                "Unknown task type for TestGeneratorAgent"
            ))
        }
    }

    // ... other trait methods
}
```

---

### 3. Executor Agent

```rust
// crates/clnrm-agents/src/agents/executor.rs

#[derive(Debug)]
pub struct ExecutorAgent {
    base: BaseAgent,
    container_pool: Arc<MultiImagePoolManager>,
    test_runner: Arc<TestRunner>,
    metrics: Arc<ExecutorMetrics>,
}

impl ExecutorAgent {
    pub async fn new(container_pool: Arc<MultiImagePoolManager>) -> Result<Self> {
        let base = BaseAgent::new(AgentRole::ExecutorAgent).await?;

        Ok(Self {
            base,
            container_pool,
            test_runner: Arc::new(TestRunner::new()),
            metrics: Arc::new(ExecutorMetrics::new()),
        })
    }

    pub async fn execute_test(&self, test: &Test) -> Result<TestResult> {
        info!("Executing test: {}", test.id);

        let start = Instant::now();

        // Step 1: Acquire container
        let (pool, container) = self.container_pool
            .acquire(&test.image)
            .await?;

        // Step 2: Setup environment
        self.setup_environment(&container, test).await?;

        // Step 3: Run test
        let result = self.test_runner
            .run_test(&container, test)
            .await?;

        // Step 4: Collect metrics
        let duration = start.elapsed();

        // Step 5: Release container
        self.container_pool
            .release(&test.image, container)
            .await?;

        self.metrics.record_execution(&result, duration);

        Ok(result)
    }

    pub async fn execute_tests_parallel(&self, tests: &[Test], parallelism: usize) -> Result<Vec<TestResult>> {
        info!("Executing {} tests with parallelism {}", tests.len(), parallelism);

        let mut results = Vec::new();
        let semaphore = Arc::new(Semaphore::new(parallelism));

        let tasks: Vec<_> = tests.iter().map(|test| {
            let sem = semaphore.clone();
            let agent = self.clone();
            let test = test.clone();

            tokio::spawn(async move {
                let _permit = sem.acquire().await.unwrap();
                agent.execute_test(&test).await
            })
        }).collect();

        for task in tasks {
            match task.await {
                Ok(Ok(result)) => results.push(result),
                Ok(Err(e)) => warn!("Test execution failed: {}", e),
                Err(e) => warn!("Task execution failed: {}", e),
            }
        }

        Ok(results)
    }

    async fn setup_environment(&self, container: &PooledContainer, test: &Test) -> Result<()> {
        // Install dependencies
        for dep in &test.dependencies {
            let cmd = Cmd::new("pip")
                .args(&["install", dep]);
            container.backend().run_cmd(cmd)?;
        }

        // Set environment variables
        for (key, value) in &test.env_vars {
            let cmd = Cmd::new("export")
                .arg(&format!("{}={}", key, value));
            container.backend().run_cmd(cmd)?;
        }

        Ok(())
    }
}

#[derive(Clone)]
pub struct ExecutorMetrics {
    tests_executed: Arc<AtomicU64>,
    tests_passed: Arc<AtomicU64>,
    tests_failed: Arc<AtomicU64>,
    total_duration_ms: Arc<AtomicU64>,
    avg_duration_ms: Arc<AtomicU64>,
}

impl ExecutorMetrics {
    pub fn new() -> Self {
        Self {
            tests_executed: Arc::new(AtomicU64::new(0)),
            tests_passed: Arc::new(AtomicU64::new(0)),
            tests_failed: Arc::new(AtomicU64::new(0)),
            total_duration_ms: Arc::new(AtomicU64::new(0)),
            avg_duration_ms: Arc::new(AtomicU64::new(0)),
        }
    }

    pub fn record_execution(&self, result: &TestResult, duration: Duration) {
        let duration_ms = duration.as_millis() as u64;

        self.tests_executed.fetch_add(1, Ordering::Relaxed);
        self.total_duration_ms.fetch_add(duration_ms, Ordering::Relaxed);

        if result.passed {
            self.tests_passed.fetch_add(1, Ordering::Relaxed);
        } else {
            self.tests_failed.fetch_add(1, Ordering::Relaxed);
        }

        let total = self.tests_executed.load(Ordering::Relaxed);
        let avg = self.total_duration_ms.load(Ordering::Relaxed) / total;
        self.avg_duration_ms.store(avg, Ordering::Relaxed);
    }
}

#[async_trait]
impl Agent for ExecutorAgent {
    async fn execute_task(&self, task: Task) -> Result<TaskResult> {
        match task.task_type {
            TaskType::ExecuteTest { test } => {
                let result = self.execute_test(&test).await?;
                Ok(TaskResult::TestExecuted(result))
            }
            TaskType::ExecuteTests { tests, parallelism } => {
                let results = self.execute_tests_parallel(&tests, parallelism).await?;
                Ok(TaskResult::TestsExecuted(results))
            }
            _ => Err(CleanroomError::invalid_operation(
                "Unknown task type for ExecutorAgent"
            ))
        }
    }

    // ... other trait methods
}
```

---

### 4. Optimizer Agent

```rust
// crates/clnrm-agents/src/agents/optimizer.rs

#[derive(Debug)]
pub struct OptimizerAgent {
    base: BaseAgent,
    history: Arc<TestHistory>,
    ml_model: Arc<OptimizationModel>,
}

impl OptimizerAgent {
    pub async fn new() -> Result<Self> {
        let base = BaseAgent::new(AgentRole::OptimizerAgent).await?;

        Ok(Self {
            base,
            history: Arc::new(TestHistory::load().await?),
            ml_model: Arc::new(OptimizationModel::load().await?),
        })
    }

    pub async fn select_critical_tests(&self, tests: &[Test]) -> Result<Vec<Test>> {
        info!("Selecting critical tests from {} candidates", tests.len());

        // Score each test by importance
        let mut scored_tests: Vec<_> = tests.iter()
            .map(|t| (t.clone(), self.score_test(t)))
            .collect();

        // Sort by score (descending)
        scored_tests.sort_by(|a, b| {
            b.1.partial_cmp(&a.1)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Select top 20% (Pareto principle)
        let cutoff = (tests.len() / 5).max(1);
        let critical: Vec<_> = scored_tests
            .into_iter()
            .take(cutoff)
            .map(|(t, _)| t)
            .collect();

        Ok(critical)
    }

    pub async fn prioritize_tests_by_failure_risk(&self, tests: &[Test]) -> Result<Vec<Test>> {
        info!("Prioritizing {} tests by failure risk", tests.len());

        // Use ML model to predict failure probability
        let mut tests_with_risk: Vec<_> = tests.iter()
            .map(|t| (t.clone(), self.predict_failure_risk(t)))
            .collect();

        // Sort by risk (descending)
        tests_with_risk.sort_by(|a, b| {
            b.1.partial_cmp(&a.1)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        Ok(tests_with_risk.into_iter().map(|(t, _)| t).collect())
    }

    pub async fn recommend_test_coverage(&self, codebase: &Path) -> Result<Vec<TestRecommendation>> {
        info!("Analyzing codebase for coverage gaps: {}", codebase.display());

        // Analyze code coverage
        let coverage = self.analyze_coverage(codebase).await?;

        // Identify gaps
        let gaps = self.identify_coverage_gaps(&coverage).await?;

        // Generate recommendations
        let recommendations = self.generate_recommendations(&gaps).await?;

        Ok(recommendations)
    }

    fn score_test(&self, test: &Test) -> f64 {
        // Score based on:
        // 1. Historical failure rate (higher = more important)
        // 2. Complexity (higher = more important)
        // 3. Code coverage impact (higher = more important)

        let failure_rate = self.history.get_failure_rate(&test.id)
            .unwrap_or(0.0);
        let complexity = self.estimate_complexity(test);
        let coverage_impact = self.estimate_coverage_impact(test);

        // Weighted average
        (failure_rate * 0.5) + (complexity * 0.3) + (coverage_impact * 0.2)
    }

    fn predict_failure_risk(&self, test: &Test) -> f64 {
        // Use ML model to predict failure probability

        let features = test.to_features();
        self.ml_model.predict(&features)
    }

    async fn analyze_coverage(&self, codebase: &Path) -> Result<CodeCoverage> {
        // Run code coverage analysis (e.g., llvm-cov)
        // Return coverage report
        unimplemented!()
    }

    async fn identify_coverage_gaps(&self, coverage: &CodeCoverage) -> Result<Vec<CoverageGap>> {
        // Find uncovered functions, branches, paths
        unimplemented!()
    }

    async fn generate_recommendations(&self, gaps: &[CoverageGap]) -> Result<Vec<TestRecommendation>> {
        // Convert gaps into test recommendations
        unimplemented!()
    }
}

#[async_trait]
impl Agent for OptimizerAgent {
    async fn execute_task(&self, task: Task) -> Result<TaskResult> {
        match task.task_type {
            TaskType::SelectCriticalTests { tests } => {
                let critical = self.select_critical_tests(&tests).await?;
                Ok(TaskResult::CriticalTestsSelected(critical))
            }
            TaskType::PrioritizeTests { tests } => {
                let prioritized = self.prioritize_tests_by_failure_risk(&tests).await?;
                Ok(TaskResult::TestsPrioritized(prioritized))
            }
            _ => Err(CleanroomError::invalid_operation(
                "Unknown task type for OptimizerAgent"
            ))
        }
    }

    // ... other trait methods
}
```

---

## Agent Pool Integration

```rust
// crates/clnrm-agents/src/pool.rs

pub struct AgentPool<T: Agent> {
    agents: Arc<SegQueue<T>>,
    active: Arc<DashMap<AgentId, T>>,
    config: PoolConfig,
}

impl<T: Agent> AgentPool<T> {
    pub async fn acquire(&self) -> Result<T> {
        // Get idle agent or spawn new
        if let Some(agent) = self.agents.pop() {
            return Ok(agent);
        }

        // Spawn new agent
        let agent = T::new().await?;
        Ok(agent)
    }

    pub async fn release(&self, agent: T) -> Result<()> {
        self.agents.push(agent);
        Ok(())
    }
}
```

---

## Orchestration Patterns

### Chaining Pattern

```rust
// Sequentially pass results between agents
pub async fn discovery_then_generation(
    repo_path: &Path,
) -> Result<Vec<Test>> {
    // Step 1: Discover tests
    let discovery_agent = TestDiscoveryAgent::new().await?;
    let discovered = discovery_agent.discover_tests(repo_path).await?;

    // Step 2: Generate additional tests
    let generator_agent = TestGeneratorAgent::new(api_key).await?;
    let mut generated = Vec::new();

    for test in discovered {
        let edge_cases = generator_agent.generate_edge_cases(&test).await?;
        generated.extend(edge_cases);
    }

    Ok(generated)
}
```

### Parallel Execution Pattern

```rust
// Execute agents in parallel
pub async fn parallel_execution(
    tests: Vec<Test>,
) -> Result<Vec<TestResult>> {
    let mut results = Vec::new();
    let mut tasks = JoinSet::new();

    for test in tests {
        tasks.spawn(async move {
            let executor = ExecutorAgent::new(pool.clone()).await?;
            executor.execute_test(&test).await
        });
    }

    while let Some(result) = tasks.join_next().await {
        results.push(result??);
    }

    Ok(results)
}
```

### Fan-Out/Fan-In Pattern

```rust
// Fan out to multiple agents, collect results
pub async fn fan_out_optimization(
    tests: Vec<Test>,
) -> Result<Vec<Test>> {
    let optimizers: Vec<_> = (0..5)
        .map(|_| OptimizerAgent::new())
        .collect::<Result<Vec<_>>>()?;

    let chunk_size = tests.len() / optimizers.len();
    let mut tasks = JoinSet::new();

    for (i, optimizer) in optimizers.iter().enumerate() {
        let chunk = tests[i * chunk_size..(i + 1) * chunk_size].to_vec();
        let opt = optimizer.clone();

        tasks.spawn(async move {
            opt.select_critical_tests(&chunk).await
        });
    }

    let mut results = Vec::new();
    while let Some(chunk) = tasks.join_next().await {
        results.extend(chunk??);
    }

    Ok(results)
}
```

---

## Testing Agent Implementations

### Unit Test Pattern

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_discovery_finds_tests() -> Result<()> {
        let agent = TestDiscoveryAgent::new().await?;
        let tests = agent.discover_tests(Path::new("tests/")).await?;

        assert!(!tests.is_empty());
        Ok(())
    }

    #[tokio::test]
    async fn test_generator_creates_valid_tests() -> Result<()> {
        let agent = TestGeneratorAgent::new(api_key).await?;
        let tests = agent
            .generate_from_description("Test sorting algorithm")
            .await?;

        for test in tests {
            assert!(!test.code.is_empty());
            assert!(test.code.contains("assert") || test.code.contains("expect"));
        }

        Ok(())
    }
}
```

---

## References

- gRPC: Distributed Communication
- Protocol Buffers: Efficient Serialization
- OpenTelemetry: Agent Observability
- async-trait: Async Trait Methods

---

**Version**: 1.0
**Last Updated**: 2025-11-18
