# complete-workflow-test - Cleanroom Test Project

This project uses the cleanroom testing framework for hermetic integration testing.

## Quick Start

```bash
# Run tests
clnrm run

# Validate configuration
clnrm validate tests/

# Show available plugins
clnrm plugins
```

## Project Structure

- `tests/` - Test configuration files
- `scenarios/` - Test scenario definitions
- `README.md` - This file

## Framework Self-Testing

This project demonstrates the cleanroom framework testing itself through the "eat your own dog food" principle.
