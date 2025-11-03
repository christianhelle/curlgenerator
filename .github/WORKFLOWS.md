# GitHub Workflows Documentation

This document describes all GitHub Actions workflows configured for this project.

## Workflows Overview

| Workflow | Trigger | Purpose | Badge |
|----------|---------|---------|-------|
| Build | Push, PR | Build & test on all platforms | [![Build](https://github.com/christianhelle/curlgenerator-rs/actions/workflows/build.yml/badge.svg)](https://github.com/christianhelle/curlgenerator-rs/actions/workflows/build.yml) |
| Smoke Tests | Push, PR, Schedule | End-to-end functional tests | [![Smoke Tests](https://github.com/christianhelle/curlgenerator-rs/actions/workflows/smoke-tests.yml/badge.svg)](https://github.com/christianhelle/curlgenerator-rs/actions/workflows/smoke-tests.yml) |
| Release | Tag push | Create releases with binaries | [![Release](https://github.com/christianhelle/curlgenerator-rs/actions/workflows/release.yml/badge.svg)](https://github.com/christianhelle/curlgenerator-rs/actions/workflows/release.yml) |
| Security Audit | Daily, Cargo changes | Security vulnerability scanning | [![Security Audit](https://github.com/christianhelle/curlgenerator-rs/actions/workflows/security.yml/badge.svg)](https://github.com/christianhelle/curlgenerator-rs/actions/workflows/security.yml) |
| Dependencies | Weekly | Automated dependency updates | [![Dependency Update](https://github.com/christianhelle/curlgenerator-rs/actions/workflows/dependencies.yml/badge.svg)](https://github.com/christianhelle/curlgenerator-rs/actions/workflows/dependencies.yml) |
| Code Coverage | Push, PR | Track code coverage metrics | [![Code Coverage](https://github.com/christianhelle/curlgenerator-rs/actions/workflows/coverage.yml/badge.svg)](https://github.com/christianhelle/curlgenerator-rs/actions/workflows/coverage.yml) |

## 1. Build Workflow

**File:** `.github/workflows/build.yml`

### Triggers
- Push to `main` branch
- Pull requests to `main`
- Manual dispatch

### Jobs

#### Lint Job
- Runs on Ubuntu
- Checks code formatting with `cargo fmt`
- Runs clippy for code quality
- Fails on warnings

#### Build & Test Job
- Runs on: Ubuntu, Windows, macOS
- Builds release binaries
- Runs all tests
- Uploads binaries as artifacts (7 day retention)

### Caching
- Cargo registry
- Cargo index
- Build artifacts

## 2. Release Workflow

**File:** `.github/workflows/release.yml`

### Triggers
- Git tags starting with `v*` (e.g., `v0.1.0`)
- Manual dispatch

### Jobs

#### Create Release
- Creates GitHub release from tag
- Extracts version from tag name

#### Build Release
Builds for multiple platforms:
- Linux x86_64
- Linux ARM64 (aarch64)
- Windows x86_64
- macOS x86_64
- macOS ARM64 (Apple Silicon)

### Artifacts
- Stripped binaries
- `.tar.gz` for Unix platforms
- `.zip` for Windows
- Uploaded as release assets

## 3. Smoke Tests Workflow

**File:** `.github/workflows/smoke-tests.yml`

### Triggers
- Push to `main`
- Pull requests
- Weekly schedule (Sunday)
- Manual dispatch

### Tests

1. **Version Command Test**
   - Verifies `--version` flag works

2. **Help Command Test**
   - Verifies `--help` flag works

3. **PowerShell Script Generation**
   - Tests generation from Petstore v3 API
   - Verifies script files are created

4. **Bash Script Generation**
   - Tests bash output format
   - Verifies script files are created

5. **Custom Base URL Test**
   - Tests `--base-url` option
   - Verifies URL appears in scripts

6. **Authorization Header Test**
   - Tests `--authorization-header` option
   - Verifies header appears in scripts

### Platform Coverage
- Ubuntu (Linux)
- Windows
- macOS

## 4. Security Audit Workflow

**File:** `.github/workflows/security.yml`

### Triggers
- Daily at midnight (UTC)
- Changes to `Cargo.toml` or `Cargo.lock`
- Manual dispatch

### Checks

1. **Security Vulnerabilities**
   - Uses `cargo audit`
   - Scans for known CVEs in dependencies

2. **Outdated Dependencies**
   - Uses `cargo outdated`
   - Reports outdated crates
   - Runs as informational (continues on error)

## 5. Dependency Update Workflow

**File:** `.github/workflows/dependencies.yml`

### Triggers
- Weekly schedule (Monday)
- Manual dispatch

### Process
1. Updates all dependencies with `cargo update`
2. Runs full test suite
3. Creates pull request if successful
4. Auto-deletes branch after merge

### PR Details
- Title: `chore: Update Cargo dependencies`
- Branch: `update-dependencies`
- Includes test results in description

## 6. Code Coverage Workflow

**File:** `.github/workflows/coverage.yml`

### Triggers
- Push to `main`
- Pull requests
- Manual dispatch

### Process
1. Installs `cargo-tarpaulin`
2. Generates coverage report
3. Uploads to Codecov
4. Continues on failure (non-blocking)

### Configuration
- Timeout: 120 seconds
- Format: XML for Codecov
- Workspace coverage
- All features enabled

## Workflow Best Practices

### Caching Strategy
All workflows use aggressive caching:
- Cargo registry cached by `Cargo.lock` hash
- Cargo index cached by `Cargo.lock` hash
- Build artifacts cached by `Cargo.lock` hash

This reduces build times significantly after the first run.

### Fail-Fast Strategy
- Build workflow: `fail-fast: false` - all platforms complete
- Release workflow: `fail-fast: false` - all platforms release
- Smoke tests: `fail-fast: false` - see all failures

### Security
- No secrets in code
- GITHUB_TOKEN used for API access
- CODECOV_TOKEN required for coverage upload
- All workflows run in isolated environments

## Creating a Release

To create a new release:

```bash
# Create and push a tag
git tag -a v0.2.0 -m "Release v0.2.0"
git push origin v0.2.0
```

This will:
1. Trigger the release workflow
2. Build binaries for all platforms
3. Create a GitHub release
4. Upload all binaries as release assets

## Manual Workflow Execution

All workflows support manual dispatch:

1. Go to Actions tab on GitHub
2. Select desired workflow
3. Click "Run workflow"
4. Select branch (usually `main`)
5. Click "Run workflow" button

## Monitoring Workflows

### Status Badges
Add these badges to your README:

```markdown
[![Build](https://github.com/christianhelle/curlgenerator-rs/actions/workflows/build.yml/badge.svg)](https://github.com/christianhelle/curlgenerator-rs/actions/workflows/build.yml)
[![Smoke Tests](https://github.com/christianhelle/curlgenerator-rs/actions/workflows/smoke-tests.yml/badge.svg)](https://github.com/christianhelle/curlgenerator-rs/actions/workflows/smoke-tests.yml)
[![Security Audit](https://github.com/christianhelle/curlgenerator-rs/actions/workflows/security.yml/badge.svg)](https://github.com/christianhelle/curlgenerator-rs/actions/workflows/security.yml)
```

### Email Notifications
GitHub automatically sends notifications for:
- Workflow failures on your commits
- Workflow failures on your PRs
- Completed workflow runs on watched repos

## Troubleshooting

### Build Failures

**Check:**
1. Cargo.lock is committed
2. All dependencies are compatible
3. Tests pass locally
4. Code formatting is correct (`cargo fmt`)
5. No clippy warnings (`cargo clippy`)

### Release Failures

**Common Issues:**
1. Tag format incorrect (must start with `v`)
2. GITHUB_TOKEN permissions insufficient
3. Cross-compilation tools not installed
4. Target not supported on runner

### Coverage Upload Failures

**Solutions:**
1. Add `CODECOV_TOKEN` to repository secrets
2. Check Codecov project is configured
3. Verify coverage format is correct

## Continuous Improvement

### Planned Enhancements
- [ ] Add benchmarking workflow
- [ ] Add performance regression tests
- [ ] Add Docker image builds
- [ ] Add crates.io publishing
- [ ] Add homebrew tap updates
- [ ] Add Windows installer builds

### Metrics to Track
- Build times across platforms
- Test execution time
- Binary sizes
- Coverage percentage
- Security vulnerabilities
- Dependency freshness

## Resources

- [GitHub Actions Documentation](https://docs.github.com/en/actions)
- [Rust CI/CD Best Practices](https://doc.rust-lang.org/cargo/guide/continuous-integration.html)
- [cargo-audit](https://github.com/RustSec/rustsec/tree/main/cargo-audit)
- [cargo-tarpaulin](https://github.com/xd009642/tarpaulin)
- [Codecov](https://about.codecov.io/)
