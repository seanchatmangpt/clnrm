# Mutation Testing Reports

## Directory Structure

```
mutation-reports/
├── rust/              # cargo-mutants reports
│   ├── baseline.json
│   ├── report_*.json
│   └── log_*.txt
├── typescript/        # Stryker reports
│   ├── optimus-prime_*.log
│   └── case-study_*.log
└── comprehensive_report_*.md  # Combined reports
```

## Report Types

### Rust Reports (cargo-mutants)
- **JSON Reports**: Machine-readable mutation results
- **Log Files**: Detailed execution logs
- **Baseline Files**: Reference scores for comparison

### TypeScript Reports (Stryker)
- **HTML Reports**: Interactive visualization
- **JSON Reports**: Structured data
- **Log Files**: Test execution details

### Comprehensive Reports
- Combined analysis of all components
- Recommendations for improvements
- Trend analysis over time

## Usage

```bash
# Generate reports
./scripts/run-mutation-tests.sh

# View latest report
cat comprehensive_report_*.md | tail -1 | xargs cat

# Compare with baseline
diff baseline.json report_latest.json
```

## Baseline Management

Keep baseline files in git for:
- Regression detection
- Trend tracking
- Quality gates

Update baseline after significant improvements:
```bash
cp report_latest.json baseline.json
git add baseline.json
git commit -m "Update mutation testing baseline"
```
