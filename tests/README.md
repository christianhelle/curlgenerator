# Smoke Tests

This directory contains comprehensive smoke tests for the cURL Request Generator, including a variety of OpenAPI specifications to ensure broad compatibility.

## Test Structure

```
tests/
├── openapi/              # OpenAPI specification files
│   ├── v2.0/            # Swagger/OpenAPI 2.0 specs
│   ├── v3.0/            # OpenAPI 3.0 specs
│   └── v3.1/            # OpenAPI 3.1 specs
├── smoke-tests.ps1      # PowerShell test runner
├── smoke-tests.sh       # Bash test runner
└── README.md            # This file
```

## OpenAPI Test Specifications

### OpenAPI v2.0 (Swagger)
- `api-with-examples` - API with example values
- `petstore` - Classic Swagger Petstore
- `petstore-expanded` - Extended Petstore with more operations
- `petstore-minimal` - Minimal Petstore example
- `petstore-simple` - Simplified Petstore
- `petstore-with-external-docs` - Petstore with external documentation
- `uber` - Uber API specification

### OpenAPI v3.0
- `api-with-examples` - API with example values
- `callback-example` - Demonstrates callback functionality
- `hubspot-events` - HubSpot Events API
- `hubspot-webhooks` - HubSpot Webhooks API
- `link-example` - Demonstrates link functionality
- `petstore` - OpenAPI 3.0 Petstore
- `petstore-expanded` - Extended Petstore
- `uspto` - USPTO API specification

### OpenAPI v3.1
- `non-oauth-scopes` - Non-OAuth security scopes
- `webhook-example` - Webhook implementation example

## Running Tests

### PowerShell (Windows/Linux/macOS)

```powershell
# Run with default binary location
.\tests\smoke-tests.ps1

# Run with custom binary
.\tests\smoke-tests.ps1 -Binary "C:\path\to\curlgenerator.exe"
```

### Bash (Linux/macOS)

```bash
# Make script executable
chmod +x tests/smoke-tests.sh

# Run with default binary location
./tests/smoke-tests.sh

# Run with custom binary
./tests/smoke-tests.sh /path/to/curlgenerator
```

## What Gets Tested

For each OpenAPI specification (both JSON and YAML formats):

1. ✅ **PowerShell Script Generation**
   - Generates `.ps1` files
   - Verifies files are created
   - Checks exit code

2. ✅ **Bash Script Generation**
   - Generates `.sh` files with `--bash` flag
   - Verifies files are created
   - Checks exit code

3. ✅ **File Count Validation**
   - Ensures scripts are actually generated
   - Reports number of files created

## Test Output

### Successful Run Example

```
╔════════════════════════════════════════════════════════════╗
║          cURL Generator - Smoke Test Suite                ║
╚════════════════════════════════════════════════════════════╝

Binary: ./target/release/curlgenerator
Test Date: 2024-11-01 12:00:00

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  OpenAPI v2.0
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  ✓ v2.0/petstore.json (14 PS, 14 SH)
  ✓ v2.0/petstore.yaml (14 PS, 14 SH)
  ...

╔════════════════════════════════════════════════════════════╗
║                      Test Summary                          ║
╚════════════════════════════════════════════════════════════╝

Total Tests:    46
Passed:         46
Failed:         0
Skipped:        0
Duration:       45.23s

✓ All tests passed!
```

### Failed Test Example

```
  ✗ v3.0/callback-example.json (PowerShell)
  ✗ v3.0/callback-example.yaml (Bash - no files)
```

## Adding New Tests

To add a new test specification:

1. Place the OpenAPI file in the appropriate version directory:
   ```
   tests/openapi/v3.0/my-new-api.json
   tests/openapi/v3.0/my-new-api.yaml
   ```

2. Update the test script to include the new spec:
   
   **PowerShell (`smoke-tests.ps1`):**
   ```powershell
   "v3.0" = @(
       "existing-spec",
       "my-new-api"  # Add this line
   )
   ```
   
   **Bash (`smoke-tests.sh`):**
   ```bash
   for spec in existing-spec my-new-api; do
   ```

3. Run the tests to verify

## CI/CD Integration

These tests are automatically run in the GitHub Actions workflow:

- **Trigger**: Push to main, pull requests
- **Platforms**: Ubuntu, Windows, macOS
- **Schedule**: Weekly (Sunday)

See `.github/workflows/smoke-tests.yml` for details.

## Test Coverage

Current test coverage:
- **OpenAPI v2.0**: 7 specifications × 2 formats = 14 tests
- **OpenAPI v3.0**: 8 specifications × 2 formats = 16 tests
- **OpenAPI v3.1**: 2 specifications × 2 formats = 4 tests
- **Total**: 34 test cases

Each test case validates both PowerShell and Bash generation, effectively doubling coverage.

## Troubleshooting

### Tests Fail to Find Binary

**Issue**: `Binary not found: ./target/release/curlgenerator`

**Solution**: Build the release binary first:
```bash
cargo build --release
```

### Permission Denied (Linux/macOS)

**Issue**: `Permission denied: ./tests/smoke-tests.sh`

**Solution**: Make the script executable:
```bash
chmod +x tests/smoke-tests.sh
```

### Test Timeout

**Issue**: Tests take too long or hang

**Solution**: 
- Check network connectivity if loading from URLs
- Verify OpenAPI files are not corrupted
- Check system resources

### Specific Spec Fails

**Issue**: One particular specification fails

**Solution**:
1. Run generator manually on that spec:
   ```bash
   ./target/release/curlgenerator tests/openapi/v3.0/failing-spec.json --output ./debug
   ```
2. Check the error message
3. Verify the OpenAPI spec is valid
4. Use `--skip-validation` if needed

## Performance Benchmarks

Typical execution times (on a modern system):
- **Small specs** (< 10 operations): ~0.5s each
- **Medium specs** (10-50 operations): ~1-2s each
- **Large specs** (> 50 operations): ~3-5s each
- **Full test suite**: ~45-60s

## Contributing

When adding new OpenAPI specifications:
1. Prefer real-world APIs over synthetic examples
2. Include both JSON and YAML formats when possible
3. Add comments explaining any special features tested
4. Ensure specs are valid according to their version
5. Test both PowerShell and Bash generation

## References

- [OpenAPI Specification](https://swagger.io/specification/)
- [OpenAPI Examples](https://github.com/OAI/OpenAPI-Specification/tree/main/examples)
- [Swagger Petstore](https://petstore.swagger.io/)
