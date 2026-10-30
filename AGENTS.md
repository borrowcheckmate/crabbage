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

## New project from this template

Follow this once, right after creating a repository from the template. Do not write product code until every phase is done. The last step deletes this section.

### 1. Set up the repository

1. Ask the user for the project name (kebab-case), a one-line description and the GitHub owner.
2. Set the git identity for this clone to the owner's `ID+username@users.noreply.github.com` address, then run `cargo xtask hooks`.
3. Run `cargo xtask rename <name>`.
4. Update the metadata: `repository` in `Cargo.toml`, the crate's `description`, `keywords` and `categories`, the `repository_owner` check in `.github/workflows/release-plz.yml`, and the README title and description.
5. Remove the rename task: `xtask/src/rename.rs`, every `rename` / `Rename` reference in xtask (`main.rs`, `cli.rs`, `task/action.rs`, `task/plan.rs`, `task/run.rs`) and its row in ARCHITECTURE.md.
6. Run `cargo xtask ci`, commit as `chore: set up project from template`, push, and confirm CI is green.

The greeting example stays for now so the build, tests and coverage keep working. Removing it is part of the first issue.

### 2. Plan the project with the user

1. Interview the user before proposing anything: the problem, who uses it, the interface (CLI, library, service), inputs and outputs, constraints (platforms, performance, dependencies), what is out of scope, and what a first useful release looks like.
2. Propose a plan: the architecture (crates, modules, main types and data flow), the external crates needed and why std is not enough, and an ordered list of milestones, each one a usable increment.
3. Iterate until the user explicitly agrees. Ask instead of assuming whenever something is unclear.

The agreed design goes into the issues in the next phase, not into ARCHITECTURE.md or a separate document.

### 3. Turn the plan into issues

1. Create one issue per milestone as an epic with `gh issue create`, and its sub-issues with `gh issue create --parent <epic>`. Each epic carries its part of the design: modules, main types, data flow and chosen crates. Sub-issues follow the task template: the goal, where to look (the epic, then the relevant ARCHITECTURE.md section), and acceptance criteria that each map to a test.
2. Make the first issue replace the greeting example with the project's first real feature, rewrite the crate section of ARCHITECTURE.md to match, and update the README install and usage.
3. Use milestones for releases (`gh api repos/{owner}/{repo}/milestones -f title=...`, then `gh issue create --milestone`) and labels for the type of work (`gh label create`).
4. Ask the user whether they want a GitHub Project board as a kanban view over these issues. The issues stay the source of truth. Creating one needs `gh auth refresh -s project`, then `gh project create` and `gh project item-add`.
5. Show the user the resulting issue list and adjust it until they are happy.

### 4. Finish

1. Delete the "Use this template" section in README.md and this section of AGENTS.md.
2. Check for leftovers: `git grep -n -i template` should only match intended text.
3. Commit as `docs: remove template bootstrap instructions` and push.

From here on, work issue by issue using the rules above.
