# Contributing to ContextFlow

Thanks for wanting to contribute. ContextFlow is an early-stage Windows
desktop product and we treat it like a real product from day one: every commit
should leave `main` releasable.

## Ground rules

1. **Windows only.** ContextFlow targets Windows 10/11. We don't accept PRs
   that add Linux or macOS code paths to the core engines. The UI may end up
   cross-platform incidentally, but that's not a goal.
2. **No placeholders in main.** A `// TODO` is fine for follow-up work tracked
   in an issue. A function that returns `unimplemented!()` is not — keep that
   work on a feature branch until it's real.
3. **Slices ship runnable.** Every PR that closes a roadmap milestone must
   include an updated acceptance test in [`docs/acceptance/`](./docs/acceptance/)
   and pass it on a clean Windows VM.

## Development setup

See the [Quickstart](./README.md#quickstart-development) in the README.

```powershell
pnpm install
cargo check --workspace
pnpm tauri dev
```

## Branch strategy

- **`main`** — stable. Protected. Only fast-forward merges from `dev` or hotfix branches.
- **`dev`** — integration. PRs target this branch by default.
- **`feature/<slice>-<short-name>`** — feature work. Branch from `dev`, rebase
  onto `dev` before opening a PR.
- **`fix/<short-name>`** — bugfixes that aren't tied to a slice.
- **`hotfix/<short-name>`** — critical fixes that branch from `main` and merge
  back to both `main` and `dev`.

Example branch names:

- `feature/slice1-foundation`
- `feature/slice2-whisper-cpp-provider`
- `feature/text-injection-uia`
- `fix/hotkey-double-registration`

## Conventional Commits

All commit messages follow [Conventional Commits](https://www.conventionalcommits.org/).
The CI pipeline enforces this on PRs.

Allowed types:

| Type     | Use for                                              |
|----------|------------------------------------------------------|
| `feat`   | New user-facing feature                              |
| `fix`    | Bugfix                                               |
| `perf`   | Performance improvement                              |
| `refactor` | Code restructuring with no behavior change         |
| `docs`   | Docs-only change                                     |
| `test`   | Adding or correcting tests                           |
| `build`  | Build system / toolchain change                      |
| `ci`     | CI configuration                                     |
| `chore`  | Repo plumbing, version bumps, dependency updates     |
| `style`  | Formatting only, no code change                      |

Scopes we use:

`audio`, `speech`, `injection`, `dictation`, `context`, `ai`, `hotkey`,
`settings`, `telemetry`, `ui`, `shell`, `installer`, `ci`, `repo`.

Examples:

```text
feat(audio): add cpal-based microphone capture pipeline
feat(injection): implement SendInput Unicode fallback
fix(hotkey): prevent duplicate key registration on hot reload
perf(transcription): reduce streaming latency by chunking on VAD silence
refactor(speech): extract SpeechProvider trait from WhisperCppProvider
docs(architecture): document the SpeechProvider plug-in contract
chore(repo): initialize contextflow workspace
```

Breaking changes use `!`:

```text
feat(speech)!: change SpeechProvider trait to async-stream returning Result
```

## Code style

- **Rust:** `cargo fmt` (config in `rustfmt.toml`) and `cargo clippy -D warnings`.
  We use `clippy::pedantic` selectively — see `Cargo.toml` lints table.
- **TypeScript/React:** ESLint + Prettier. Components are functional with hooks.
  No class components. Tailwind for styling, shadcn primitives over hand-rolled UI.
- **Imports:** grouped (std → external → workspace → local) with a blank line between groups.
- **Errors:** `thiserror` for typed errors in libraries, `anyhow` only at the
  binary boundary. Never `unwrap()` outside of tests and `main.rs` startup.

## Tests

- Unit tests live next to the code, in `#[cfg(test)] mod tests`.
- Integration tests live in `crates/<crate>/tests/`.
- E2E and acceptance tests live in `apps/desktop/tests/e2e/` and run via
  `pnpm test:e2e`. These drive the real app via WebDriver where possible.
- Performance budgets are asserted in `crates/<crate>/benches/` via `criterion`.

Run the full suite locally with `cargo test --workspace && pnpm test`.

## Pre-commit quality gates

We use [`lefthook`](https://github.com/evilmartians/lefthook) to run formatters
and fast linters on staged files. Install it once:

```powershell
cargo install lefthook
lefthook install
```

What runs on `git commit`:

- `cargo fmt --check` on staged `.rs` files
- `cargo clippy --workspace -- -D warnings`
- `cargo check --workspace`
- `pnpm lint --filter ...[HEAD]`
- `pnpm typecheck`

What runs on `git push`:

- `cargo test --workspace` (only when Rust files changed)

CI re-runs all of the above plus the full test suite, `tauri build`
validation, and security scans.

## Reporting bugs

Open an issue with:

1. Windows version (`winver`)
2. ContextFlow version (`Settings → About`)
3. Which app you were dictating into
4. Reproducible steps
5. The diagnostic log from `Settings → Diagnostics → Export logs`

## Filing security issues

Don't open public issues for security bugs. Email `security@contextflow.dev`.
See [`docs/security.md`](./docs/security.md) for the disclosure policy.
