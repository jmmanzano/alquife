# Agent notes

## Build and verification
- This is a single Rust binary crate; `Cargo.lock` is committed. Use `cargo build --release --locked` for a reproducible release build.
- Focused local checks: `cargo fmt -- --check`, `cargo check`, `cargo clippy`, and `cargo test`. There is no configured CI workflow in this checkout; run relevant checks locally.
- Linux builds need the native development packages described in `README.md` (pkg-config, OpenSSL, D-Bus). FFmpeg is an external runtime dependency, not bundled by the Rust build.

## Structure and runtime
- `src/main.rs` parses CLI options, initializes file logging, loads config, starts the update check, then runs `app::App`; `src/app/` owns state, event handling, and playback orchestration.
- `src/ui/` renders Ratatui pages; `src/audio/` contains FFmpeg and PipeWire integration; `src/subsonic/` implements the server API. Keep cross-platform code in mind: MPRIS is `cfg(unix)`, while terminal setup has Windows-specific branches.
- Config and runtime state paths are defined in `src/config/paths.rs`; avoid assuming paths from documentation without checking this source. Queue and UI state persist across runs.
- Themes and equalizer presets are seeded/loaded when the app starts. Follow their existing file formats and loading code when changing these features.

## Subsonic compatibility
- Server implementations differ despite sharing the Subsonic API. Preserve compatibility behavior in `src/subsonic/`, especially radio station calls, which were adjusted for Navidrome.

## OpenSpec
- Baseline behavior is documented in `openspec/specs/`; use the OpenCode `/opsx-propose`, `/opsx-apply`, and `/opsx-archive` workflows for future changes. Artifacts are written in Spanish; keep OpenSpec structural headings and keywords in English.
- Validate with `openspec validate --specs --strict --no-interactive`; run `openspec update` to refresh the generated OpenCode integration.

<!-- code-review-graph MCP tools -->
## MCP Tools: code-review-graph

**IMPORTANT: This project has a knowledge graph. ALWAYS use the
code-review-graph MCP tools BEFORE using Grep/Glob/Read to explore
the codebase.** The graph is faster, cheaper (fewer tokens), and gives
you structural context (callers, dependents, test coverage) that file
scanning cannot.

### When to use graph tools FIRST

- **Exploring code**: `semantic_search_nodes_tool` or `query_graph_tool` instead of Grep
- **Understanding impact**: `get_impact_radius_tool` instead of manually tracing imports
- **Code review**: `detect_changes_tool` + `get_review_context_tool` instead of reading entire files
- **Finding relationships**: `query_graph_tool` with callers_of/callees_of/imports_of/tests_for
- **Architecture questions**: `get_architecture_overview_tool` + `list_communities_tool`

Fall back to Grep/Glob/Read **only** when the graph doesn't cover what you need.

### Key Tools

| Tool | Use when |
| ------ | ---------- |
| `detect_changes_tool` | Reviewing code changes — gives risk-scored analysis |
| `get_review_context_tool` | Need source snippets for review — token-efficient |
| `get_impact_radius_tool` | Understanding blast radius of a change |
| `get_affected_flows_tool` | Finding which execution paths are impacted |
| `query_graph_tool` | Tracing callers, callees, imports, tests, dependencies |
| `semantic_search_nodes_tool` | Finding functions/classes by name or keyword |
| `get_architecture_overview_tool` | Understanding high-level codebase structure |
| `refactor_tool` | Planning renames, finding dead code |

### Workflow

1. The graph auto-updates on file changes (via hooks).
2. Use `detect_changes_tool` for code review.
3. Use `get_affected_flows_tool` to understand impact.
4. Use `query_graph_tool` pattern="tests_for" to check coverage.
