# CI Integration Samples

This directory contains sample CI configurations for integrating port-kill cache management into your CI/CD pipelines.

## Available Samples

### 1. GitHub Actions (`github-actions.yml`)
- **Scheduled runs**: Daily at 2 AM UTC
- **Manual triggers**: Workflow dispatch with cache type selection
- **Features**:
  - Cache analysis with size thresholds
  - Conditional cleanup based on cache size
  - System diagnostics
  - PR comments with cache analysis results
  - Artifact upload for results

### 2. GitLab CI (`gitlab-ci.yml`)
- **Scheduled runs**: Daily cache cleanup
- **Manual triggers**: Web interface with cache type selection
- **Features**:
  - Multi-stage pipeline (analysis → cleanup → diagnostics)
  - Environment variables for configuration
  - Artifact management
  - Conditional execution based on cache size

### 3. CircleCI (`circleci-config.yml`)
- **Scheduled runs**: Daily at 2 AM UTC
- **Manual triggers**: Multiple workflow types
- **Features**:
  - Parameterized workflows
  - Multiple cleanup strategies (all, rust, js, python, java, npx, js-pm)
  - Workspace persistence
  - Artifact storage

## Usage

### GitHub Actions
1. Copy `github-actions.yml` to `.github/workflows/cache-management.yml`
2. Customize the schedule and thresholds as needed
3. The workflow will run automatically and can be triggered manually

### GitLab CI
1. Copy `gitlab-ci.yml` to your GitLab project as `.gitlab-ci.yml`
2. Configure the `CACHE_THRESHOLD` and `STALE_DAYS` variables
3. Set up scheduled pipelines in GitLab project settings

### CircleCI
1. Copy `circleci-config.yml` to `.circleci/config.yml`
2. Configure parameters in CircleCI project settings
3. Set up scheduled workflows in CircleCI dashboard

## Configuration Options

### Cache Types
- `all` - Clean all detected caches
- `rust` - Clean Rust caches (target/, ~/.cargo)
- `js` - Clean JavaScript/TypeScript caches (node_modules, .next, .vite, etc.)
- `python` - Clean Python caches (__pycache__, .venv, .pytest_cache)
- `java` - Clean Java caches (.gradle, build, ~/.m2)
- `npx` - Clean NPX caches with stale filtering
- `js-pm` - Clean JS package manager caches (npm, pnpm, yarn)

### Thresholds
- **Cache size threshold**: Default 1GB, configurable
- **Stale days**: Default 30 days for NPX packages
- **Disk usage warnings**: 80% and 90% thresholds

### Safety Features
- **Safe delete**: All cleanups use `--safe-delete` by default
- **Backup creation**: Automatic backup before cleanup
- **Restore capability**: `--restore-last` command available
- **Dry run**: Use `--dry-run` to preview changes

## Examples

### Clean only NPX caches older than 7 days
```bash
./port-kill-console cache --npx --clean --stale-days 7
```

### Clean all caches with system diagnostics
```bash
./port-kill-console cache --clean --doctor
```

### Analyze cache without cleaning
```bash
./port-kill-console cache --list --json
```

## Monitoring

All CI samples include:
- **Cache analysis**: Size, entry count, stale count
- **System diagnostics**: Disk usage, warnings, errors
- **Artifact storage**: Results saved for review
- **Conditional execution**: Only clean when thresholds exceeded

## Best Practices

1. **Start with analysis**: Always run `--list` before `--clean`
2. **Use safe delete**: Enable `--safe-delete` for production
3. **Set appropriate thresholds**: Don't clean too aggressively
4. **Monitor results**: Review diagnostics and artifacts
5. **Test locally**: Verify commands work before CI deployment
6. **Backup strategy**: Keep backups for critical caches
7. **Staged rollout**: Start with less critical cache types

## Troubleshooting

### Common Issues
- **Permission errors**: Ensure CI has write access to cache directories
- **Large cache sizes**: Adjust thresholds or use specific cache types
- **Build failures**: Check Rust toolchain and dependencies
- **Missing caches**: Verify cache detection is working correctly

### Debug Commands
```bash
# Check system health
./port-kill-console cache --doctor

# Analyze specific cache types
./port-kill-console cache --lang rust --list
./port-kill-console cache --npx --list

# Test cleanup without execution
./port-kill-console cache --clean --dry-run
```
