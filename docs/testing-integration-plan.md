# Integration Test Plan (v0.2.0)

## Overview

This document outlines the integration test strategy for features that require real GitHub API access or GraphQL support. These tests are deferred from v0.1.0 to v0.2.0.

## Deferred Tests

### GitHub Projects V2 (15 tests)

**Reason for Deferral**: GitHub Projects V2 primarily uses GraphQL API, not REST API. The REST endpoints described in the spec may not exist or are in beta.

**Current Status**: 
- Project operations are stubbed with `unimplemented!()` 
- 15 tests are ignored in `src/client/project_tests.rs`
- 2 serialization tests pass (type validation only)

**Required for v0.2.0**:

1. **GraphQL API Support**
   - Implement GraphQL client for Projects V2
   - Add GraphQL query builder or use existing library
   - Handle GraphQL-specific error responses

2. **Test Organization Setup**
   - GitHub organization with Projects V2 enabled
   - Test projects with known structure
   - Test issues/PRs for adding to projects

3. **GitHub App Permissions**
   - `organization_projects: read` - List and read projects
   - `organization_projects: write` - Add/remove items, modify projects
   - `contents: read` - Access repository content for node IDs

4. **Integration Test Implementation**
   - Real API calls to test organization
   - Cleanup after each test (remove items, reset state)
   - Rate limiting consideration (use retries)
   - Environment variables for credentials

**Test Coverage Needed**:

- `test_list_organization_projects` - List org projects via GraphQL
- `test_list_user_projects` - List user projects via GraphQL
- `test_get_project_organization` - Get org project details
- `test_get_project_user` - Get user project details
- `test_get_project_not_found` - Handle 404 errors
- `test_add_item_to_project` - Add issue/PR to project
- `test_add_item_already_in_project` - Handle duplicate items
- `test_remove_item_from_project` - Remove item from project
- `test_remove_item_not_found` - Handle missing items
- `test_project_not_found` - Handle missing projects
- `test_organization_not_found` - Handle missing organizations
- `test_forbidden_access` - Handle permission errors
- 3 serialization tests for GraphQL response structures

## Alternative Approach

If GraphQL support is too complex for v0.2.0:

**Option A**: Remove project operations entirely
- Remove `src/client/project.rs` and `src/client/project_tests.rs`
- Document that users should use GitHub's GraphQL API directly
- Focus SDK on REST API operations only

**Option B**: Basic REST API support (if endpoints exist)
- Research actual GitHub REST API for Projects V2
- Implement only operations with stable REST endpoints
- Document GraphQL requirement for advanced features
- Keep integration tests minimal

## Test Environment Requirements

### For v0.2.0 Integration Tests

1. **GitHub App Registration**
   - Create test GitHub App
   - Generate private key for JWT signing
   - Configure webhook URL (for webhook tests)

2. **Test Organization**
   - Dedicated test organization (e.g., `github-bot-sdk-testing`)
   - Multiple repositories for different test scenarios
   - Projects V2 enabled with test projects

3. **CI/CD Integration**
   - Store App ID and private key as secrets
   - Run integration tests on schedule (not every commit)
   - Use separate job for integration tests
   - Allow manual triggering

4. **Cleanup Strategy**
   - Each test creates uniquely named resources
   - Tests clean up after themselves
   - Periodic manual cleanup for failed tests
   - Consider dedicated cleanup job

## Timeline

- **v0.1.0**: Ship without Projects V2 support (current)
- **v0.2.0**: Add GraphQL support and integration tests
- **v0.3.0**: Full Projects V2 feature parity with GraphQL

## Notes

- Projects V2 is still evolving (beta features exist)
- GraphQL API is more stable than REST for Projects V2
- Consider using GitHub's official GraphQL Rust client
- Integration tests should be opt-in during development
- Unit tests with mocks remain primary test strategy
