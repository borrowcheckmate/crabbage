# Agents

Instructions for automated contributors. Humans follow the same rules via CONTRIBUTING.md.

1. Read ARCHITECTURE.md first, then only the modules your task touches.
2. Work from an issue. If the task is not one, propose an issue instead of a large change.
3. Run `cargo xtask ci` before declaring work done. All checks must pass; do not weaken lints, thresholds or policy to get there.
4. Every behaviour change ships with tests. No manual verification steps.
5. Keep files under 500 lines. When a file grows a distinct concern, make it a submodule.
6. `mod.rs` only declares modules and re-exports.
7. No comments explaining code. Rename or extract until it reads clearly. `///` docs on public items.
8. No new dependencies without the checks in CONTRIBUTING.md → Dependencies.
9. No loose notes or documents. Update the existing documents or the issue.
10. No personal data: no real emails, names, home paths or tokens. Git identity must be a noreply address.
11. No attribution or advertising for tools in commits, PRs, code or docs.
12. PR descriptions: what, why, how tested. Nothing else.
13. ARCHITECTURE.md describes the code as it is, never plans. Update it in the same commit that changes the structure. Plans live in issues.
14. Use the `gh` CLI for everything on GitHub: issues, sub-issues, labels, milestones, PRs, reviews, CI runs and project boards.
