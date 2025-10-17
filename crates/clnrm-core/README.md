# Cleanroom Test Project

This project uses the cleanroom testing framework for hermetic integration testing.

## Quick Start

```bash
# Run tests
clnrm run tests/

# Validate configuration
clnrm validate tests/

# Show available plugins
clnrm plugins
```

## Project Structure

- `tests/` - Test definition files (.clnrm.toml)
- `scenarios/` - Test scenario definitions
- `cleanroom.toml` - Optional framework configuration
- `README.md` - This file

## Framework Self-Testing

This project demonstrates the cleanroom framework testing itself through the "eat your own dog food" principle.
