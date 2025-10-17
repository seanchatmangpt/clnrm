# Cursor Commands for Cleanroom Testing Framework

## Overview

This directory contains Cursor commands specifically designed for the cleanroom testing framework development team. These commands follow the project's core team best practices and help maintain FAANG-level code quality standards.

## Available Commands

### 🔍 Code Quality & Review
- **`code-review-checklist`** - Comprehensive code review checklist following FAANG-level standards and .cursorrules guidelines
- **`enforce-best-practices`** - Systematically enforce best practices, validate error handling, async patterns, and anti-pattern detection

### 🧪 Testing & Validation
- **`run-all-tests-and-fix`** - Execute full test suite and systematically fix failures following core team testing standards
- **`framework-self-test`** - Execute framework's comprehensive self-test suite (framework "eats its own dog food")

### 💻 Development Workflow
- **`dev-workflow`** - Standardized development workflow for efficient, high-quality development cycles

### 🏗️ Project Management
- **`project-maintenance`** - Comprehensive project maintenance procedures for long-term codebase health

### 📚 Documentation & Reporting
- **`generate-docs`** - Generate comprehensive documentation suite for APIs, usage, and architecture
- **`security-audit`** - Comprehensive security audit ensuring secure handling and protection against vulnerabilities

## Command Categories

### Quality Assurance (3 commands)
Focus on maintaining FAANG-level code quality through systematic validation and best practices enforcement.

### Development Workflow (2 commands)
Streamline development processes while ensuring quality gates and best practices are followed.

### Project Management (2 commands)
Maintain long-term project health through regular maintenance, monitoring, and improvement.

### Specialized Tools (1 command)
Security auditing to ensure robust protection and compliance with security standards.

## Core Principles

All commands follow these core team principles:

- **🎯 No False Positives** - Never return `Ok(())` without doing actual work
- **🚫 No unwrap()/expect()** - Proper error handling throughout
- **🔄 Dyn Compatibility** - All traits remain `dyn` compatible
- **🧪 Honest Testing** - Tests follow AAA patterns and are properly async
- **📦 Clean Architecture** - Proper module organization and imports
- **🔒 Security First** - Input validation and secure error handling

## Usage

Commands are triggered by typing `/` in the Cursor chat input, followed by the command name. For example:

- `/code-review-checklist` - Start a comprehensive code review
- `/run-all-tests-and-fix` - Execute and fix test suite
- `/enforce-best-practices` - Validate best practices compliance
- `/framework-self-test` - Run framework self-validation
- `/dev-workflow` - Follow standardized development process
- `/project-maintenance` - Perform project maintenance tasks
- `/generate-docs` - Generate comprehensive documentation
- `/security-audit` - Perform security assessment

## Integration with Project Standards

These commands are designed to work seamlessly with:

- **`.cursorrules`** - Core team best practices and standards
- **`scripts/`** directory - Existing automation and validation scripts
- **`crates/`** structure - Multi-crate architecture
- **`tests/`** organization - Comprehensive testing approach
- **`docs/`** structure - Documentation standards

## Maintenance

Commands should be regularly updated to:

- Reflect changes in `.cursorrules` and best practices
- Incorporate new project capabilities and features
- Address gaps identified through usage and feedback
- Maintain alignment with the project's evolving architecture

## Command Development Guidelines

When creating or modifying commands:

1. **Follow AAA Pattern** - Arrange, Act, Assert structure
2. **Use Proper Error Handling** - Never use `unwrap()` or `expect()`
3. **Maintain Dyn Compatibility** - No async trait methods
4. **Be Honest** - Use `unimplemented!()` for incomplete features
5. **Document Thoroughly** - Clear purpose, steps, and success criteria
6. **Test Commands** - Ensure they work as described

---

*These commands represent the accumulated wisdom and best practices of the core team, ensuring consistent high-quality development across the entire cleanroom testing framework project.*
