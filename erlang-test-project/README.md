# Cleanroom + Erlang Integration Test

This project demonstrates how Cleanroom can test applications written in Erlang, proving that the framework works across different programming language ecosystems.

## 🎯 Purpose

This test validates that Cleanroom's "eat your own dog food" philosophy works for applications in any language, not just Rust. It shows that:

- ✅ Cleanroom can test containerized applications regardless of implementation language
- ✅ HTTP API testing works across language boundaries
- ✅ Hermetic testing is possible for multi-language applications
- ✅ Cross-language integration testing is supported

## 🏗️ Architecture

### Erlang Application (`src/simple_erlang.erl`)
- Simple HTTP server written in Erlang
- Provides multiple endpoints for testing:
  - `/` - Basic hello world
  - `/hello` - Function call response
  - `/add/2/3` - Arithmetic operation
  - `/json` - JSON response
  - `/health` - Health check endpoint
- Compiled and run in Alpine Linux container

### Cleanroom Test (`erlang-integration-test.toml`)
- TOML configuration for testing the Erlang application
- Tests HTTP endpoints via curl commands
- Validates JSON responses and arithmetic operations
- Demonstrates cross-language API testing

## 🚀 Usage

### 1. Build the Erlang Docker Image
```bash
cd erlang-test-project
docker build -t erlang-test:latest .
```

### 2. Run the Cleanroom Test
```bash
# From project root
clnrm run erlang-test-project/erlang-integration-test.toml
```

### 3. Run the Demo Script
```bash
cd erlang-test-project
./test-erlang-with-cleanroom.sh
```

## 📋 What This Proves

### Framework Capabilities
- ✅ **Language Agnostic**: Works with Erlang, Python, Java, Go, etc.
- ✅ **Container-Based Testing**: Tests any containerized application
- ✅ **HTTP API Testing**: Validates REST endpoints regardless of backend
- ✅ **Cross-Language Integration**: Tests multi-language service architectures

### "Eat Your Own Dog Food" Validation
- ✅ **Framework tests non-Rust applications**: Proves universality
- ✅ **Real HTTP testing**: No mocked responses
- ✅ **Actual container execution**: Tests real Erlang runtime
- ✅ **Cross-ecosystem validation**: Works beyond Rust ecosystem

## 🎉 Test Results Expected

When run successfully, this test will validate:

```
📋 Step: test_root_endpoint
✅ Erlang Test Server - Hello World!

📋 Step: test_hello_endpoint
✅ Hello from Erlang!

📋 Step: test_math_endpoint
✅ 5

📋 Step: test_json_endpoint
✅ JSON response with message field

📋 Step: test_health_endpoint
✅ {"status":"healthy"}

✅ All assertions passed
🎉 Test 'erlang_integration_test' PASSED
```

## 🔧 Technical Details

### Dockerfile Features
- Uses `erlang:25-alpine` base image
- Compiles Erlang source code
- Exposes port 8080
- Runs Erlang application directly

### Cleanroom Configuration Features
- Tests HTTP endpoints via curl
- Validates JSON responses
- Tests arithmetic operations
- Validates service health endpoints
- Demonstrates hermetic testing of Erlang applications

## 🌟 Real-World Applications

This pattern enables testing of:
- **Erlang Microservices**: Cowboy, Yaws, or custom HTTP servers
- **Multi-Language Systems**: Erlang backend with React frontend
- **Legacy System Integration**: Testing existing Erlang applications
- **Cross-Platform APIs**: Any HTTP API regardless of implementation

## 📚 Integration with Main Examples

This Erlang test complements the main Cleanroom examples by demonstrating:

- **Cross-Language Testing**: Shows framework works beyond Rust
- **HTTP API Validation**: Universal endpoint testing
- **Container Agnosticism**: Tests any containerized application
- **Ecosystem Independence**: Works with Erlang/OTP, Python, Java, etc.

## 🎯 Validation Criteria

This test validates that Cleanroom successfully:
1. ✅ Starts Erlang application in container
2. ✅ Tests HTTP endpoints via curl commands
3. ✅ Validates JSON response formats
4. ✅ Tests arithmetic operations
5. ✅ Validates service health endpoints
6. ✅ Demonstrates hermetic isolation for Erlang apps
7. ✅ Provides clear error messages for Erlang-specific issues

## 🚀 Next Steps

To extend this pattern:
1. Add more Erlang-specific test scenarios
2. Test Erlang clustering or distributed features
3. Integrate with Erlang testing frameworks (Common Test, EUnit)
4. Test Erlang applications with databases (Mnesia, PostgreSQL)
5. Validate OTP application structure and releases

This demonstrates that Cleanroom is a **universal testing framework** that works across programming language boundaries, making it suitable for complex, multi-language architectures.
