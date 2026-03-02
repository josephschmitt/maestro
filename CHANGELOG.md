# Changelog

## [0.2.0](https://github.com/josephschmitt/maestro/compare/maestro-v0.1.1...maestro-v0.2.0) (2026-03-02)


### Features

* rich prompt input with paste compression, drag-drop, history, and char count ([#48](https://github.com/josephschmitt/maestro/issues/48)) ([ad4116d](https://github.com/josephschmitt/maestro/commit/ad4116d48f902bd3fad7ff59fb2b0d51073f6f0d))
* streaming-optimized markdown rendering with syntax highlighting ([#46](https://github.com/josephschmitt/maestro/issues/46)) ([bcb3c62](https://github.com/josephschmitt/maestro/commit/bcb3c626df04f1795b56a0b010faa8be41ecc392))
* whimsical animated loading states with rotating verbs ([#47](https://github.com/josephschmitt/maestro/issues/47)) ([c03f4ba](https://github.com/josephschmitt/maestro/commit/c03f4ba46f875f9fbc918a37727aeb7c8ca9a82c))


### Bug Fixes

* **ci:** match Tauri's aarch64 DMG naming in homebrew workflow ([2b5bd68](https://github.com/josephschmitt/maestro/commit/2b5bd68686b612df88185eaa38c3e0ea0bc283e7))

## [Unreleased]

### Added

- **Rich prompt input** - Paste compression for long text into visual chips, file drag-and-drop onto message input, input history with arrow key navigation, and character/token count feedback. ([JJS-95](https://linear.app/josephschmitt/issue/JJS-95/oc-3-rich-prompt-input), [#48](https://github.com/josephschmitt/maestro/pull/48))

## [0.1.1](https://github.com/josephschmitt/maestro/compare/maestro-v0.1.0...maestro-v0.1.1) (2026-02-23)


### Bug Fixes

* **ci:** merge release-please into release workflow to fix trigger ([84c7fb3](https://github.com/josephschmitt/maestro/commit/84c7fb347ef72707aa6d2860602172db14187a23))

## [0.1.0](https://github.com/josephschmitt/maestro/compare/maestro-v0.0.1...maestro-v0.1.0) (2026-02-23)


### Features

* Add bearer token authentication for network mode (Phase 5a) ([#39](https://github.com/josephschmitt/maestro/issues/39)) ([e8afc94](https://github.com/josephschmitt/maestro/commit/e8afc940c3582d0c5905710b74d8c0764ee36b7c))
* Add HTTP API route dispatcher for browser app (Phase 2c) ([#34](https://github.com/josephschmitt/maestro/issues/34)) ([0b9b540](https://github.com/josephschmitt/maestro/commit/0b9b540443a542d358fe9fd8f1c89fd8dd771e40))
* Add HTTP server settings UI and connection indicator (Phase 5b) ([#40](https://github.com/josephschmitt/maestro/issues/40)) ([db014a0](https://github.com/josephschmitt/maestro/commit/db014a09fe8fe75da7942b7cd916d973db034014))
* Add Polish + Error Handling (Phase 22) ([#42](https://github.com/josephschmitt/maestro/issues/42)) ([a389f9f](https://github.com/josephschmitt/maestro/commit/a389f9f7d5e5117da5db17745caef8fb3b6a4751))
* add pre-push hooks and actionlint ([c269d7a](https://github.com/josephschmitt/maestro/commit/c269d7ad953db113a9c9b2d5d98e25f2321b0b98))
* Add Settings + Configuration UI (Phase 21) ([#41](https://github.com/josephschmitt/maestro/issues/41)) ([0ed6d35](https://github.com/josephschmitt/maestro/commit/0ed6d354398874d5f2339598dc4b126e80c85005))


### Bug Fixes

* **ci:** fix release workflow validation errors ([b4c1bf9](https://github.com/josephschmitt/maestro/commit/b4c1bf95f972e66f2640b584aaafcc4a2e8e224c))
* **ci:** use correct rust-toolchain action name ([1c6a29f](https://github.com/josephschmitt/maestro/commit/1c6a29fe48c0cb6324ad5ced9343211206d3e3e0))
* **lint:** resolve all lint errors for CI ([07ff30f](https://github.com/josephschmitt/maestro/commit/07ff30f606d5d780152d2d84e4abe6bd0c9ca57e))
