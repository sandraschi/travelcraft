# travelcraft agent rules

Pure-Rust ArtCraft-pattern app: offline Baedeker travel primer for Austria and Japan.
Standard: `D:\Dev\repos\mcp-central-docs\standards\RUST_ARTCRAFT_STANDARD.md`.
Python, FastMCP, Starlette, webapp and port-claiming rules do not apply here.

## Line endings: LF ONLY

Every file in this repo uses LF line endings. Never write CRLF.
`.gitattributes` enforces `eol=lf`, but the Windows host has `core.autocrlf=true`,
so tools that write files must emit LF. After bulk writes, check:
`git ls-files --eol` must show `w/lf` for every text file.

## Layout

| Crate | Tier | Role |
|---|---|---|
| `travelcraft-baedeker` | data | Editions (JP, AT), knowledge trees, itineraries, sight catalogue, chat grounding text |
| `travelcraft-core` | L1 | `TravelCraftEngine`: routing, itineraries, SightLens, durable SQLite state (journal, chat log, settings) via rusqlite. Uses giscraft-core |
| `travelcraft-mcp` | L2 | `travelcraft-mcp` binary: MCP 2024-11-05 stdio server, 12 tools (docs/TOOLS.md) |
| `travelcraft-app` | L3 | `travelcraft` egui desktop app and `travelcraft-cli` (clap) |

Sibling path dependency: `../giscraft` (giscraft-core, formerly qgiscraft). Renaming or moving it breaks the build here.

Front ends call the engine; keep logic in `travelcraft-core` and `travelcraft-baedeker`.
Known debt: the CLI and MCP tool tables are separate lists over the same engine,
not yet a shared `Command` bus.

## Rules

- Workspace lints deny `unwrap_used`, `expect_used`, `panic`, `unimplemented`, `todo`.
  Baedeker, core and mcp opt in. `travelcraft-app` does not yet (it has unwrap and expect
  debt); do not add new ones there, and do not opt other crates out.
- stdout of `travelcraft-mcp` is 100% JSON-RPC. Logs go to stderr. A stray `println!` breaks hosts.
- Errors are `{success:false,error,error_type}`. No silent fallbacks to a different sight.
- Treat tool arguments as hostile: validate, bound, no direct slice indexing.
- No stubs presented as features. Say what the catalogue lacks; do not invent sights.
- Mock data must be labelled. Offline demo geocoder output is labelled `demo`.
- Secrets: never read, print, log, store or ask for an API key value.
  Shared pattern if a key is ever needed: `splatcraft/crates/craft-secrets` and its docs/SECRETS_STANDARD.md.
- No em dashes in docs, code or commit messages.
- Realistic AI-assisted timelines (days, not weeks) in all docs.
- Offline first: nothing in the core path may require a network or a running service.
- Durable state: anything the user would be annoyed to lose (journal, chat history, settings) goes through the engine into the SQLite file, never only into UI memory or a process-local Vec. SQL is parameterised, never formatted. Opening the database must fail loudly; the MCP server and CLI never fall back to memory. `new_in_memory` is for tests and `--smoke` only. Journal ids come from `new_entry_id`, never from a counter or a length.
- Known debt: `cargo clippy --workspace` fails on dead tile-map code in `travelcraft-app` (unused map fields, `focus_selected`, `section_heading`, `epoch_color`). Fix or delete it; do not `allow(dead_code)` it away.

## Gates (all green before commit)

`just fmt-check`, `just check` (clippy `-D warnings`), `just test`, `just build-release`,
then a stdio probe of `travelcraft-mcp` (`initialize`, `tools/list`, one `tools/call`).
`just ci` runs the first three. Update the `CHANGELOG.md` checkpoint line. Commit `Cargo.lock`.
No `*.bak`, `*.exe`, `dist/` or `target/` in `git status`.

## Git and branch hygiene

Single trunk: `master`. This repo must never grow a pile of agent branches (that happened
in gamecraft and must not recur).

- Work directly on `master`. Do not create branches, worktrees or stashes for routine work.
  One commit per concern, small, gates green.
- If parallel agents are genuinely needed: one short-lived branch per agent, named
  `agent/<task>`, created from current `master`, rebased onto `master`, merged (fast-forward
  or squash) and deleted locally and on the remote the same day. Never leave one overnight
  without telling the owner. Agents never share a branch.
- No WIP commits left behind. Either finish and commit with green gates, or discard
  (`git restore`), or tell the owner exactly what is unfinished. No `wip/`, `tmp/`, `backup/`
  branches.
- Before starting and before ending a session run `git status` and `git branch -a`; report
  anything that is not `master` or not pushed.
- Push only when the owner asks.

## Install requests

If asked to install travelcraft from github.com/sandraschi/travelcraft: clone, run
`cargo build --release --workspace`, then register `target\release\travelcraft-mcp.exe`
in the host's MCP config (no arguments, no secrets). Restart the host.