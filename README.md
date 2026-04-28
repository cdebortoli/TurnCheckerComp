# Turn Checker Companion

Turn Checker Companion is the desktop companion app for the Turn Checker iOS app.

It is designed for PC/Mac players of turn-based strategy games and board games. The desktop app manages the currently active game on the computer, while the iOS app communicates with it over local HTTP requests to synchronize checks, comments, turns, and related game state.

## Download

For normal use, download the app from the GitHub **Releases** page. Each official release provides ready-to-use archives for supported platforms, such as macOS and Windows.

Do not download random binaries from other sources.

## Why release builds are safe to use

Official release builds are produced by GitHub Actions using the workflow in this repository.

For each tagged release:

- GitHub checks out the repository source code.
- GitHub installs the configured Rust toolchain.
- The app is built in release mode with locked dependencies.
- macOS and Windows archives are generated automatically.
- A `SHA256SUMS.txt` file is published with the release so each archive can be verified.
- GitHub also provides the source code archive for the exact tag used to create the release.

This means a release can be traced back to the source code and workflow used to build it, and downloaded archives can be checked against their published SHA-256 hash.

## Verifying a download

After downloading a release archive, compare its SHA-256 hash with the value listed in `SHA256SUMS.txt` on the same release page.
