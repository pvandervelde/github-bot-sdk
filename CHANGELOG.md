# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.1] - 2026-07-10

### Bug Fixes

- **release-plz**: checkout merge commit SHA when tagging releases [c41158052250ec08eed61d31b31826019bbc09ad]
- **release-plz**: checkout merge commit SHA when tagging releases (#81) [2bd1160fbc9f7d15515d0faea0f412af5d173f65]


### Continuous Integration

- **deps**: update github actions [3b11ac1331c892b4b29a9d05b99029544e685622]
- **deps**: update github actions (#85) [75ec57e3f24a6c8574686db92a8dad47683f90ef]
- **deps**: update github actions to v7 [d9e127eaadb577be6bfa6f3e9a98d9ea8c2f005f]
- **deps**: update github actions to v7 (#96) [a98984b433568df518aaf569c49aa5119de28466]
- **deps**: update taiki-e/install-action digest to c7eb173 [b122f858356726b150c0380ca18aff7edc16e8e6]
- **deps**: update taiki-e/install-action digest to c7eb173 (#86) [3466d311bbe073fd45b4657c2b19481f77386b92]
- replace release-plz with release-regent [1a7568cef058e4f7db8eb04aa03b46394b2b80ed]
- replace release-plz with release-regent (#99) [a70ef158130c8db6b272c70026a4b86f6c94a68d]


### Chores

- **deps**: bump cmov from 0.5.3 to 0.5.4 [060fe1cf50827fd57ad97af7df1f951c0e7c31ca]
- **deps**: bump cmov from 0.5.3 to 0.5.4 (#93) [9fdbaba97cac9b281b8a5d717e51002f65c1adeb]
- **deps**: lock file maintenance [c190857b59a04037dce78aade7906344ace185db]
- **deps**: lock file maintenance [8ff7bbe54effb9809afa4bb48f91021a02cc91d6]
- **deps**: lock file maintenance (#83) [c1e9d8f96378807edca13a9e09e70627c5671f86]
- **deps**: lock file maintenance (#97) [d71dc4fabb64a8d3bdbe0fd141cf7237ffac78ef]
- **deps**: update rust crate anyhow to v1.0.103 [4a800b77d2eaf25e9cb479dd45a9b54040db773d]
- **deps**: update rust crate anyhow to v1.0.103 (#94) [e67ca79bc8f53837bf786d384ac6e93b2e51ed5e]
- **deps**: update rust crate bytes to v1.12.1 [ca09b4350d0d26e18d140c5d721810a4ae4bbf29]
- **deps**: update rust crate bytes to v1.12.1 (#92) [e2d8f7b876dfce70a65936624a327d43ad9778f0]
- **deps**: update rust crate chrono to v0.4.45 [3807827fb70e60e7c7cba5b2b8df7c06b95f3abb]
- **deps**: update rust crate chrono to v0.4.45 (#91) [decaf8540476f88527891d74bd34685510a807d6]
- **deps**: update rust crate jsonwebtoken to v10.4.0 [fb6f56384f6f6862aad6abcd39540ba33039b290]
- **deps**: update rust crate jsonwebtoken to v10.4.0 (#88) [8eb7dc7317239dcf11f5e7f6d38fe54657dd87d2]
- **deps**: update rust crate rand to v0.10.2 [f0a5620093006c3240421f29cb19e6f3cee96b1f]
- **deps**: update rust crate rand to v0.10.2 (#95) [5f2ec880d27a2d27fc967227a6074fa52a140a5b]
- **deps**: update rust crate reqwest to v0.13.4 [e82870684beafdf9a02c3d597fb8a6b73906658d]
- **deps**: update rust crate reqwest to v0.13.4 (#84) [4ba2423f7dca9346f660f6b07c4a864d48c4b333]
- **deps**: update rust crate serde_json to v1.0.150 [56e98dde872104c971ad4801651e2e058af90b95]
- **deps**: update rust crate serde_json to v1.0.150 (#89) [f99fa48f176d1ee6b8a0550c65387bf365076e55]
- **deps**: update rust crate tokio to v1.52.3 [90f89ce3a7caea0806627f6eab418eaf136400e8]
- **deps**: update rust crate tokio to v1.52.3 (#87) [30878ab628809c393bd987318dd3587031e8e1d3]
- **deps**: update rust crate uuid to v1.23.4 [21ca86e5502e45f73509ee6e28251e1b675806dd]
- **deps**: update rust crate uuid to v1.23.4 (#90) [e4dd7ededf6463db87d643360b04f59281fb169f]
- Adding the config for merge-warden [4d6de7c2801e08c160d3fdfc3b57a260cd15b1eb]
- Adding the config for merge-warden (#98) [ef17b260a03b7b364a2332c9ca15cab704fb33ab]
- release v0.2.0 (#36) [cc658e7c6e76a34a92a32e4c41dbf1236d96c440]
- release v0.2.1 [943a02960b279c9a20a48c014a5c6b546bfebc5c]
- release v0.2.1 (#82) [364d7e98f48963bfea082c0e40c4e8dab0be7098]

## [0.2.0] - 2026-04-22

### <!-- 1 -->🐛 Bug Fixes

- Update rustls-webpki to 0.103.13 to resolve RUSTSEC-2026-0104

- PullRequestsClient::set_milestone delegates to Issues API and re-fetches PR

- **config**: Add use_merge_commit=true so feat PR merges appear in changelog

- **config**: Remove body skip rule that drops commits with blank lines in body

- **ci**: Install typos before running release-plz to enable git-cliff changelog generation

- Update rand Rng import to RngExt for rand 0.10 compatibility

- **deps**: Update rust crate rand to 0.10.0

- **client**: Add Display and Default impls for workflow enum types

- **deps,tests**: Update rustls-webpki to fix CVEs, fix workflow test assertions

- **client**: Add workflow enum types and align spec with implementation

- **client**: Address PR review feedback on map_http_error migration

- **client**: Consolidate map_http_error, fix remove_label return type, update docs

- **changelog**: Replace remote.github template vars with REPO postprocessor

- **changelog**: Add remote.github section to cliff.toml

- **deps**: Upgrade rand 0.9.2 -> 0.9.3 to resolve RUSTSEC-2026-0097

- **client**: Remove unused import and fix infinite-loop in multi-page mock test

- **client**: Use map_error in PullRequestsClient::remove_label

- **client**: Unify add_labels error handling to use map_error

- **client**: Address PR review feedback

- Address PR review comments on spec docs

- **ci**: Trigger release workflow only on release PR merge

- **ci**: Split release-plz into separate release and PR workflows


### <!-- 2 -->🚜 Refactor

- **client**: Delegate IssuesClient::fetch_all to fetch_all_pages

- **client**: Eliminate duplicate pagination loops and fix label URL encoding

- **client**: Implement domain sub-client API pattern


### <!-- 3 -->📚 Documentation

- Update README examples to use domain sub-client API


### <!-- 6 -->🧪 Testing

- **client**: Fix remove_label test to assert Vec<Label> and add NotFound case

- **client**: Add per_page and auth header matchers to not_found comment mock

- **client**: Update issue_tests for fetch_all_pages delegation

- **webhook**: Raise large-payload timing threshold to 1s

- **client**: Add tests for new sub-client methods


### <!-- 7 -->⚙️ Miscellaneous Tasks

- **deps**: Update taiki-e/install-action digest to a2352fc

- **github**: Add PowerShell script to update repository label colors

- **deps**: Update taiki-e/install-action digest to 5939f33

- **deps**: Update taiki-e/install-action digest to eea29cf

- **deps**: Update anthropics/claude-code-action digest to 5fb8995

- **deps**: Update github actions

- Fix Rust formatting

- **deps**: Update github actions ([#20](https://github.com/pvandervelde/github-bot-sdk/issues/20))

- Fixing PR comments

- Fixing formatting


[0.2.0]: https://github.com/pvandervelde/github-bot-sdk/compare/0.1.0..0.2.0

<!-- generated by git-cliff -->

## [0.1.0] - 2026-01-24

### <!-- 0 -->⛰️  Features

- **auth,client,events,webhook,error**: Implement complete GitHub Bot SDK

### <!-- 3 -->📚 Documentation

- **interfaces**: Fix README to reference consolidated additional-operations spec

- **specs**: Update specifications

- **specs**: Complete constraints.md refactoring

- **specs**: Restructure architecture specifications

### <!-- 7 -->⚙️ Miscellaneous Tasks

- **config**: Modernize CI, dependencies, and release tooling ([#2](https://github.com/pvandervelde/github-bot-sdk/issues/2))

- Trying to fix the cargo deny workflow steps

- Addressing cargo clippy errors

- Update to cargo deny v3

- **config**: Fix repository URL in cliff.toml changelog generator

- **security**: Configure cargo-deny for security and license compliance

- **config**: Enhance Renovate configuration for automated dependency management

- Modernize GitHub Actions workflows and fix doc warning

<!-- generated by git-cliff -->

