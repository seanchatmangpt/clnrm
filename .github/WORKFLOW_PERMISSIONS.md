# GitHub Actions Workflow Permissions Reference

This document defines the minimal required permissions for each workflow type in this repository.

## Permission Matrix

### Workflows That Read Only (No Permissions Needed)
- unit-tests.yml
- fast-tests.yml
- fuzz.yml
- performance.yml
- performance-regression.yml
- contract-tests.yml
- integration-tests.yml
- lib-*.yml (library check workflows)
- best-practices.yml
- schema-validation.yml
- telemetry-validation.yml
- weaver-validation.yml
- weaver-live-check-tests.yml
- weaver-refactor-validation.yml

**Permission Block:**
```yaml
permissions:
  contents: read
```

### Workflows That Create Releases & Write to Repo
- publish-crates.yml
- release.yml
- homebrew-release.yml
- pages.yml

**Permission Block:**
```yaml
permissions:
  contents: write
  pull-requests: read
  id-token: write  # For OIDC token (crates.io)
```

### Workflows That Manage GitHub Pages
- pages.yml
- documentation.yml

**Permission Block:**
```yaml
permissions:
  contents: read
  pages: write
  id-token: write
```

### Workflows That Comment on PRs
- (None currently, but add if added in future)

**Permission Block:**
```yaml
permissions:
  contents: read
  pull-requests: write
```

## Implementation

Each workflow should have a `permissions` section at the top level (not inside jobs):

```yaml
name: My Workflow
on: [push, pull_request]

# Explicit permissions (REQUIRED FOR SECURITY)
permissions:
  contents: read

jobs:
  my-job:
    runs-on: ubuntu-latest
    steps:
      # ...
```

## Security Implications

- **Default behavior (no permissions specified)**: Full access to secrets and repository
- **Explicit minimal permissions**: Following principle of least privilege
- **GitHub's OIDC tokens** (`id-token: write`): Only needed for OIDC flows (e.g., crates.io publishing)

## Audit Checklist

- [ ] All 29 workflows have explicit `permissions` section
- [ ] No workflow has `permissions: write-all`
- [ ] No workflow requests `secrets: inherit` unnecessarily
- [ ] OIDC workflows only request `id-token: write` (not full access)
