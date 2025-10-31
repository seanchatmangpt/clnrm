# README.md Claims Extraction

## Version Claim
**Line 3**: `version-1.0.0`
**Actual**: `clnrm 1.0.0` ✅

## CLI Commands to Verify

### Core Commands
1. `clnrm init` - Line 71
2. `clnrm run` - Line 78
3. `clnrm validate tests/` - Line 90
4. `clnrm plugins` - Line 98
5. `clnrm --version` - Line 406 (VERIFIED ✅)
6. `clnrm --help` - Line 407 (VERIFIED ✅)
7. `clnrm template <type>` - Line 39, 195
8. `clnrm template otel` - Line 195, 411
9. `clnrm self-test` - Line 25, 362, 412
10. `clnrm services status` - Line 34
11. `clnrm services logs` - Line 35
12. `clnrm services restart` - Line 36
13. `clnrm dev --watch` - Line 10, 413
14. `clnrm dry-run` - Line 11, 414
15. `clnrm fmt` - Line 12, 415
16. `clnrm analyze test.toml traces.json` - Line 333

### Template Commands
- `clnrm template otel > my-test.clnrm.toml` - Line 195

### Installation Commands
- `brew tap seanchatmangpt/clnrm` - Line 429
- `brew install clnrm` - Line 430
- `cargo install clnrm` - Line 438

### Verification Commands (Line 493)
```bash
clnrm init && clnrm run && clnrm validate tests/
clnrm self-test
clnrm plugins
```

## Feature Claims

### Plugins (Line 28-32)
- GenericContainerPlugin
- SurrealDbPlugin
- NetworkToolsPlugin

### Templates (Line 39-43)
- Default Template
- Database Template
- API Template

### Output Examples

#### Container Execution (Line 350-358)
```
$ clnrm run
🚀 Executing test: basic_test
📋 Step 1: hello_world
🔧 Executing: echo Hello from cleanroom!
📤 Output: Hello from cleanroom!
✅ Output matches expected regex
✅ Step 'hello_world' completed successfully
🎉 Test 'basic_test' completed successfully!
```

#### Self-Test (Line 362-368)
```
$ clnrm self-test
Framework Self-Test Results:
Total Tests: 5
Passed: 5
Failed: 0
✅ All framework functionality validated
```

#### Plugins List (Line 372-377)
```
$ clnrm plugins
📦 Available Service Plugins:
✅ generic_container (alpine, ubuntu, debian)
✅ surreal_db (database integration)
✅ network_tools (curl, wget, netcat)
```

## Performance Metrics (Line 532-535)
- ✅ First green: <60s
- ✅ Hot reload latency: <3s
- ✅ Dry-run validation: <1s for 10 files
- ✅ Cache operations: <100ms

## File Generation Claims (Line 73)
`clnrm init` should generate:
- tests/basic.clnrm.toml
- README.md
- scenarios/

## Documentation References
- docs/PRD-v1.md - Line 204
- docs/CLI_GUIDE.md - Line 205, 476
- docs/TOML_REFERENCE.md - Line 206, 477
- docs/TERA_TEMPLATES.md - Line 207
- docs/v1.0/MIGRATION_GUIDE.md - Line 208
- docs/FAKE_GREEN_DETECTION_USER_GUIDE.md - Line 341
- docs/FAKE_GREEN_DETECTION_DEV_GUIDE.md - Line 342
- docs/FAKE_GREEN_TOML_SCHEMA.md - Line 343
- docs/CLI_ANALYZE_REFERENCE.md - Line 344
- docs/V1.0_ARCHITECTURE.md - Line 524

## Badge Claims (Line 3-5)
- version-1.0.0-blue
- build-passing-green
- license-MIT-blue
