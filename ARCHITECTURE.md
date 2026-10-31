# Architecture

Read this first. It tells you where things live so you only need to open the files relevant to your task. It describes the code as it is; planned work lives in issues.

## Layout

```
crates/crabbage-domain/   pure recipe rules, no IO
crates/crabbage-store/    SQLite persistence
crates/crabbage-import/   recipe import from web pages, photos and Mealie
crates/crabbage/          the Leptos app: library (SSR shell and hydration) + server binary
xtask/                    repository automation, invoked as `cargo xtask`
.githooks/                pre-commit and commit-msg hooks, both call xtask
.github/                  CI, release, dependabot, issue and PR templates
.devcontainer/            development container
Dockerfile                image build: native server build, WASM site, scratch runtime
```

## crates/crabbage-domain, crates/crabbage-import

Their responsibilities:

| Crate | Responsibility |
| --- | --- |
| `crabbage-domain` | Pure recipe rules. No IO |
| `crabbage-import` | Recipe import from web pages, photos and Mealie |

`crabbage-domain` modules:

| Module | Responsibility |
| --- | --- |
| `src/membership.rs` | `Role` (`Admin`, `Member`) and `Membership`, derived from a login's groups by `Membership::from_groups`: admin if the admin group is present, households are the groups with the household prefix and a name after it. `has_household_features` is true only with an active household |
| `tests/membership.rs` | Role, household matching, deduplication and the feature check |
| `src/quantity/mod.rs` | Declares the quantity submodules and re-exports `Dimension`, `Unit`, `Quantity` and `ImperialUnit` |
| `src/quantity/unit.rs` | `Dimension` (`Mass`, `Volume`, `Count`, `Unitless`) and `Unit`: `ALL` (in form order), `from_symbol`, `dimension`, `factor` to the base unit (g, ml or piece; `None` for unitless), `symbol` |
| `src/quantity/amount.rs` | `Quantity` (finite, non-negative amount and unit, `new` returns `None` otherwise): `convert_to` within a dimension, `checked_add` (same unit adds as it is, mixed units add in the base unit), `display` (g below 1000 g else kg; ml below 1000 ml else l; tsp and tbsp kept; count and unitless unchanged) |
| `src/quantity/imperial.rs` | `ImperialUnit` (oz, lb, cup, fl oz): `to_metric` gives the displayed metric quantity |
| `tests/quantity.rs` | Conversion within and across dimensions, spoons, display, addition, imperial input, unitless amounts and rejection of negative or non-finite amounts |
| `src/scaling.rs` | `Servings` (at least one, `new` returns `None` for zero) and `scale`: multiplies by the servings ratio, displays, then rounds per unit (g and ml by size, kg and l to 0.05, spoons to 0.25, pieces to 0.5). Non-zero amounts never round to zero; equal servings and unitless amounts are returned unchanged |
| `tests/scaling.rs` | Ratio scaling, per-unit rounding, the smallest step for tiny amounts, display boundary after rounding, identity at equal servings, unitless amounts, zero servings and overflow |
| `src/shopping.rs` | `Item` (`Ingredient` by id, or `Text`) and `Line` (item and amounts). `merge(items)`: same-ingredient items share a line at the first one's position, text items never merge, each amount is added to the first `checked_add`-able amount on its line or appended, `None` amounts add nothing |
| `tests/shopping.rs` | Merging to the best unit, amounts in different dimensions kept apart in order, spoons added in the base unit, text items never merged, first-seen order, items without amounts, unitless amounts kept apart unless equal |
| `src/durations.rs` | `detect_durations(text)`: the durations a step's text names, in order, as `std::time::Duration`. English and Dutch amounts (`25 minutes`, `1,5 uur`, `anderhalf uur`, `een half uur`), short units and whole words only, a range gives its upper bound, and a smaller unit after a larger one (`1 hour 30 minutes`, `1 uur en 30 minuten`) makes one duration. Zero amounts are left out |
| `tests/durations.rs` | Every form of the durations in both languages, case, whole-word units, numbers without units, combinations and the ones that stay apart |
| `src/permissions.rs` | `can_manage_catalog(role)`: true only for `Role::Admin`. `can_edit_recipe(role, user, owner, editors)`: admin, owner or a listed editor. `can_delete_recipe` and `can_manage_editors(role, user, owner)`: admin or owner. Anyone signed in may create a recipe |
| `tests/permissions.rs` | Admins manage the catalog, members do not; owner, listed editor, admin and other member for each recipe rule |
| `src/planner.rs` | `plan(days, candidates, history, lookback_weeks, rng)`: chooses a recipe in each requested category, excludes recipes planned within the lookback, chooses randomly among remaining recipes, falls back to the least recently planned recipe (never planned first, ties by id), and preserves rest and empty categories as `None`. Planning dates are ordered integer day numbers |
| `src/dates.rs` | Converts Gregorian dates to and from integer day numbers and parses and formats ISO dates, with leap-year validation |
| `tests/planner.rs` | Category membership, lookback exclusion, least-recent fallback, rest days, empty categories and no repeats when enough candidates exist |
| `tests/unit_symbols.rs` | `Unit::from_symbol` finds every unit by its symbol and nothing else |

## crates/crabbage-import

Parses free-text ingredient lines, matches their names to catalogue ingredients, reads recipes from web pages and completes them with AI when needed. Depends on `crabbage-domain` for `Unit`, `Quantity` and `ImperialUnit`. The web part fetches with `reqwest` (rustls, no default features, the same version and TLS stack as `openidconnect`). JSON-LD is read from the page's `application/ld+json` script tags without an HTML parser. The AI client posts a recipe's text and photos to an OpenAI-compatible chat completions endpoint (`reqwest`, the key as a bearer token) and reads the JSON draft from the first answer, checking it before use.

| Module | Responsibility |
| --- | --- |
| `src/lib.rs` | Crate doc and the `ai`, `lines`, `matching`, `mealie`, `pipeline` and `web` modules |
| `src/lines/mod.rs` | Declares the line modules, re-exports `parse_line` and `ParsedLine`, and re-exports `measure_of` for the crate |
| `src/lines/line.rs` | `parse_line(text)`: collapses whitespace, reads the amount, unit, name and note. `ParsedLine` (`amount`, `amount_max`, `unit`, `name`, `note`, `original`, `parsed`). A line is parsed when its name has a letter or digit; otherwise it keeps its collapsed text, with no amount, unit or note. A comma or parenthesis starts the note |
| `src/lines/numbers.rs` | Amounts: integers, decimals with a comma or dot, `1/2`, `1 1/2`, unicode fractions (`½ ¼ ¾ ⅓ ⅔ ⅛`, also glued as `1½`), and ranges with `-`, `–`, `tot`, `to`, `à` |
| `src/lines/units.rs` | `Measure` and the English and Dutch unit table. Domain units map directly; `dl` and `cl` scale to ml; `cup`, `oz`, `lb` and `fl oz` convert to metric through `ImperialUnit`. Units without a domain counterpart (`clove`, `teen`, `blik`) stay in the name. `measure_of(symbol)` (crate-only) finds the measure of a whole symbol, ignoring case, for the AI draft check and the merge. Drops a `.` and a following `of` or `of the` after a unit |
| `src/lines/words.rs` | `strip`: whole-word, case-insensitive match at the start of a text |
| `tests/lines/main.rs`, `tests/lines/table.rs` | Table tests for amounts, units, names and notes in both languages, unparsed lines and whitespace |
| `src/matching.rs` | `normalise` (lowercase, trimmed, whitespace collapsed, Latin-1 diacritics folded to ASCII), `Candidate` (id, name, aliases), `match_ingredient` (first normalised name match, then first alias match, empty input matches nothing), `match_all` (`Matches`: one link per input in order, and the original spellings of unmatched names, once per normalised name, in first-seen order) |
| `src/mealie/mapping.rs` | Pure Mealie mapping: foods resolve to existing ingredients or deduplicated pending-review creation requests; recipes map servings, image id, rating, categories/tags, instructions and ingredient lines using the shared matching and unit conversion rules; category and tag names resolve against existing categories or become deduplicated new names |
| `tests/matching.rs` | Case, whitespace and diacritics, exact name and alias matches, names beating aliases, no partial matches, empty input, unmatched names in order and once |
| `src/web/mod.rs` | Declares the web submodules and re-exports `import`, `extract`, `fetch`, `fetch_bytes`, `Fetched`, `Limits`, `Draft`, `ImportError` and `is_public` |
| `src/web/language.rs` | `html_lang(html)` (crate-only): the `lang` attribute of the `<html` tag, the name matched without case, the value with or without quotes, empty values as none. `xml:lang` is not read |
| `src/web/draft.rs` | `Draft`: title, servings, `language`, image, ingredient lines, steps, and the prep, cook and total times as `Duration`. `language` is the recipe's `inLanguage` string, else the page's `html` `lang`, else none |
| `src/web/error.rs` | `ImportError` (`NotHttp`, `PrivateAddress`, `Timeout`, `TooLarge`, `Status(u16)`, `Network(text)`, `NoRecipe`) with English messages and `std::error::Error` |
| `src/web/jsonld.rs` | `extract(html)`: the first schema.org `Recipe` in the `application/ld+json` scripts, searched in top-level objects and arrays and in `@graph`. Scripts that are not JSON are skipped and a recipe needs a name. Servings are the first integer of the yield, ingredients are trimmed non-empty lines, steps come from text, name, string lines or `HowToSection` `itemListElement`s flattened in order |
| `src/web/duration.rs` | `parse`: ISO 8601 durations of days, hours, minutes and whole seconds (`PT1H30M`, `P0DT20M`, `PT90S`); anything else is `None` |
| `src/web/fetch.rs` | `Limits` (timeout 10 s, 5 MiB, `allow_private` false by default) and `fetch(url, limits)` and `fetch_bytes(url, limits)` (the raw body as `Fetched`: bytes and the declared `Content-Type`, with the same checks; `fetch` reads it as UTF-8 text): http and https only, every resolved address checked with `address::is_public`, the connection pinned to the checked address, no proxy, redirects followed by hand (at most five) with the same checks, body size checked by `Content-Length` and while reading, non-success status as `Status` |
| `src/web/address.rs` | `is_public(ip)`: false for loopback, unspecified, private, link-local, CGNAT, broadcast, documentation, multicast and unique-local addresses, and for IPv4 mapped into IPv6 |
| `src/web/import.rs` | `import(url, limits)`: `fetch` then `extract` |
| `tests/web/main.rs` | Declares the web test modules |
| `tests/web/extract.rs` | Recipe fields from the fixtures, `@graph`, type lists, sections, string instructions, yield, image and duration forms, invalid scripts, and pages without a recipe |
| `tests/web/fetch.rs` | Loopback server: body, redirect, declared and streamed size limits, timeout, 404, `import`, refused schemes and private addresses, and `fetch_bytes` with its content type |
| `tests/web/address.rs` | Public addresses allowed and private, special and mapped addresses refused |
| `tests/web/errors.rs` | Display text of every `ImportError` variant |
| `tests/web/language.rs` | The language from `inLanguage`, from the `html` `lang` (quotes, case, other attributes, `xml:lang` ignored), and none when neither is given |
| `src/mealie/mod.rs` | Declares and re-exports the Mealie client, errors and API models |
| `src/mealie/client.rs` | `MealieClient` validates an HTTP(S) base URL, sends the API token as a bearer token, reads paginated recipe summaries, foods, units, categories and tags, fetches a recipe by slug, and downloads its original image up to 5 MiB |
| `src/mealie/model.rs` | Serde models for Mealie recipe summaries, recipes, ingredients, foods, units, categories, tags and instructions; optional fields default when absent or null |
| `src/mealie/error.rs` | `MealieError` reports invalid URLs, refused tokens, missing resources, non-Mealie responses, timeouts, network failures, other statuses and oversized images without exposing the token |
| `tests/mealie/main.rs`, `tests/mealie/client.rs` | Loopback API server: pagination for every collection, bearer token, recipe model, image download, refused token, invalid URL and non-Mealie responses |
| `tests/fixtures/mealie/recipe.json` | Invented Mealie v1 recipe response used to test deserialization |
| `tests/fixtures/web/*.html` | Small invented pages: plain, `@graph`, type list, sections, string instructions, article only, invalid script before a recipe, and no JSON-LD |
| `src/ai/mod.rs` | Declares the AI submodules and re-exports `AiClient`, `AiDraft`, `AiLine`, `AiStep`, `parse`, `AiError`, `Image` and `Prompt` |
| `src/ai/client.rs` | `AiClient` (base URL, optional API key, model, timeout 60 s; `with_timeout`). `complete(prompt)` posts to `{base}/chat/completions` with bearer auth only when a key is set, and reads `choices[0].message.content` through `parse`. Timeouts, non-success statuses, connection failures and answers without a message map to `AiError`. `Debug` prints the key as `<redacted>` |
| `src/ai/request.rs` | `Prompt` (instructions, text, `Image`s) and `Image` (media type, bytes). Builds the chat body: JSON object response format, a system message with the instructions and the shape of the draft, and a user message with the text part (left out when empty) and one `data:` URL image part per photo |
| `src/ai/draft.rs` | `AiDraft`, `AiLine`, `AiStep` (serde) and `parse(content)`: strips a ```json fence, reads the draft and checks it: non-blank title, lines and steps present, names and step texts non-blank, amounts not negative, units known to `Unit::from_symbol` or to the line parser's unit table (`measure_of`, so `cup` is accepted and `handful` is not), step line indexes existing. Failures are `AiError::Invalid` with the reason |
| `src/ai/error.rs` | `AiError` (`Timeout`, `Status(u16)`, `Network(text)`, `Invalid(reason)`) with user-facing messages (401, 403 and 429 have their own text) and `std::error::Error` |
| `tests/ai/main.rs` | Declares the AI test modules and the loopback mock server, which records raw requests and answers with a canned status and body, or never answers |
| `tests/ai/client.rs` | Request path with and without a trailing slash on the base, bearer auth with and without a key, body parts and the image data URL, the text part left out, a valid answer, and timeouts |
| `tests/ai/parse.rs` | A full valid draft, fenced content, optional fields, every invalid case with its reason, and the line parser's units (`cup`, `oz`, `fl oz`, `dl`) accepted |
| `tests/ai/errors.rs` | Display text of every `AiError`, statuses 401, 429 and 500, unreachable service, answers without a message, and the API key absent from errors and `Debug` output for each error case |
| `src/pipeline/mod.rs` | Declares the pipeline submodules and re-exports `from_draft`, `needs_ai`, `merge`, `complete`, `Imported`, `ImportedLine` and `ImportedStep` |
| `src/pipeline/imported.rs` | `Imported` (title, servings, image, lines, steps, `unmatched`, `ai_error`), `ImportedLine` (amount, domain unit, name, note, `ingredient_id`, `original`) and `ImportedStep` (text, sorted `line_positions`). Units are always metric or domain units |
| `src/pipeline/deterministic.rs` | `from_draft(draft, candidates)`: each line parsed with `parse_line`, matched by name with `match_all` (unparsed lines are unmatched by their text), and each step linked to the parsed lines whose normalised names it contains as whole words (a letter or digit on either side blocks the match) |
| `src/pipeline/decide.rs` | `needs_ai(imported, page_language, instance_language)`: true for a blank title, missing servings, no lines or steps, unmatched lines, or a page language whose primary subtag differs from the instance's. An unknown page language alone does not trigger it |
| `src/pipeline/merge.rs` | `merge(ai, base, candidates)`: the AI's title, lines and steps; servings from the AI or else the page; the page's photo. Each line's unit is converted to metric (`Unit::from_symbol` first, else `measure_of` and `Measure::quantity`; a measure without an amount is dropped). The AI's `ingredient_id` is kept only when it is a candidate's id, else the name is matched again. Step positions are sorted and deduplicated, and `unmatched` is recomputed from the lines without an id |
| `src/pipeline/photos.rs` | `from_photos(images, candidates, client, instance_language)`: one AI request with the shared instructions (`run.rs`'s `instructions`, plus reading the recipe from the photos) and every photo, merged onto an empty reading (no photo); an `AiError` is returned |
| `src/pipeline/run.rs` | `complete(draft, candidates, ai, instance_language)`: `from_draft`, then, when an AI client is configured and `needs_ai`, one `complete` call with a text-only `Prompt` (the instructions name the language code, the allowed unit symbols and the zero-based line indexes), merged on success. A failure keeps the reading with `ai_error` set to the error's text |
| `tests/pipeline/main.rs` | Shared fixtures: a draft, candidates, a complete `Imported` |
| `tests/pipeline/from_draft.rs` | Matched, alias-matched, unmatched and unparsed lines, unmatched names once, step links by whole word in English and Dutch, unparsed lines linking to nothing, and a name named twice in a step linked once |
| `tests/pipeline/needs_ai.rs` | Each trigger alone, the complete draft in the instance language false, `nl-NL` and `nl_BE` against `nl` false, `en-GB` against `nl` true, and an unknown page language false |
| `tests/pipeline/merge.rs` | Imperial units to metric (cup, oz, dl), a measure without an amount, domain units and amounts kept, bogus and valid ingredient ids, step positions from the AI, the title from the AI and the photo from the page, servings fallback, recomputed unmatched names, and the line text |
| `tests/pipeline/photos.rs` | Loopback AI: every photo is sent as a `data:` URL part with the language instruction and the answer becomes the recipe, an AI error is returned |
| `tests/pipeline/complete.rs` | Loopback AI server: not called for a complete draft, called for an unmatched line (its request carries the title and lines) and merged, called for another page language, an AI 500 keeps the reading with `ai_error`, and no AI config returns the reading |
| `tests/pipeline/server.rs` | The loopback server that records raw requests and answers with a canned status and body |

## crates/crabbage-store

SQLite via `sqlx` (bundled SQLite, tokio runtime, no TLS). `open(data_dir)` creates the directory and `crabbage.db` if missing, sets WAL journaling and foreign keys on every connection, and applies the embedded migrations. Every write transaction starts with `begin_write` (`BEGIN IMMEDIATE`), so concurrent writers wait for the busy timeout instead of failing to upgrade a read transaction. `StoreError` wraps the io, sqlx and migration errors. Depends on `crabbage-domain` for `Servings`, `Quantity` and `Unit` (recipe scaling in the shopping list).

| Module | Responsibility |
| --- | --- |
| `src/lib.rs` | Crate doc, module declarations, re-exports |
| `src/pool.rs` | `open`: connection options, the embedded `MIGRATOR`; `begin_write` |
| `src/error.rs` | `StoreError`: io, database and migration failures, `Conflict(text)` when a name or alias is taken or would equal a name or alias, `InUse` when recipe lines use an ingredient, `UnknownLine` when a step links to a line the recipe does not have, `InvalidStars` when a rating is not from 1 to 5, `Empty` when a shopping text is blank, and planner date or setting validation errors |
| `src/users.rs` | `upsert_user`: one row per OIDC subject, returns the user id. `find`: the name and admin flag of a user (`User`) |
| `src/sessions.rs` | `create`, `find_user`, `delete`. Takes the raw token and stores only its SHA-256 hash |
| `src/login_attempts.rs` | `create` and `take` for pending logins. `take` deletes the row; attempts expire after 10 minutes |
| `src/households.rs` | `sync_membership` (one transaction per login: admin flag, households upserted by group name, memberships replaced, active household kept, cleared or defaulted to the first by group name), `households_of`, `active_household`, `set_active_household` (only for members) |
| `src/catalog/mod.rs` | Declares `categories`, `ingredients` and `sections` (all public) and the shared rules in `names.rs` |
| `src/catalog/names.rs` | Maps unique violations to `Conflict`. A name is never an alias and an alias never a name, both case-insensitively; the checks run inside the store functions |
| `src/catalog/ingredients.rs` | `Ingredient` with its aliases. `list`, `list_pending`, `create`, `create_pending_with_aliases` (one transaction, skipping aliases already used as names or aliases), `rename`, `delete` (`InUse` while recipe lines use it), `add_alias`, `remove_alias`, `set_section`, `approve`, `merge` (one transaction: recipe lines and shopping items move to the target, aliases move, the source name becomes an alias of it, the source is deleted). Missing rows return `false` |
| `src/catalog/sections.rs` | `Section`. `list` in position order, `create` at the end, `rename`, `delete` (ingredients keep their row and lose the section), `move_section` renumbers 0 to n-1 with the moved section at the new position |
| `src/catalog/categories.rs` | `Category`. `list` by name, `create`, `rename`, `delete` (the category leaves every recipe it was filed under) |
| `src/shopping/mod.rs` | Declares `model`, `read` and `write`, and re-exports `ShoppingItem`, `list` and the write functions |
| `src/shopping/model.rs` | `ShoppingItem`: ingredient id and name, text, amount, unit, `checked`, recipe id, plan-entry id, and the ingredient's store section id and position |
| `src/shopping/read.rs` | `list(pool, household)`: the household's items, unchecked first, then by id. Joins ingredients and store sections |
| `src/shopping/write.rs` | `add_recipe` scales recipe lines; `add_plan` atomically adds each not-yet-listed non-rest entry in an inclusive date range and marks it listed. Rows carry recipe and optional plan-entry source ids. Updating a listed plan entry replaces unchecked rows at its new recipe and servings; removing it cascades unchecked rows while checked rows are detached and kept. Shared insertion uses the recipe servings and `Unit::from_symbol`/`scale`. `add_text` and `edit_text` trim and refuse blank text (`Empty`); edit only changes manual items. `remove`, `set_checked` (one `begin_write`, returns the rows changed) and `clear_checked`. Every query filters by household |
| `src/planner/mod.rs` | Declares the planner model, recipe lookup, read and write modules and re-exports `PlannerSettings`, `PlanEntry`, and planner operations |
| `src/planner/model.rs` | `PlannerSettings` (lookback weeks, default servings, seven Monday-first optional category ids; `None` is rest) and `PlanEntry` (ISO date, optional recipe id, servings). The stored plan row also tracks whether its shopping items have been generated |
| `src/planner/read.rs` | `settings` returns household settings or defaults (one lookback week, four servings, rest every day); `entries` returns the inclusive date range ordered by date; `history` returns non-rest recipe ids and dates in an inclusive range. Every query filters by household |
| `src/planner/recipes.rs` | `recipes_in_categories` returns the `(category, recipe)` pairs for a set of categories in one query; `recipe_exists` checks a recipe id; `categories_exist` checks all selected weekday categories in one query |
| `src/planner/write.rs` | `save_settings` atomically replaces settings and all weekday choices; `set_entry` upserts by household and date and refreshes listed shopping rows atomically; `remove_entry` removes the entry and its unchecked shopping rows while retaining checked rows. Dates are validated calendar dates in `YYYY-MM-DD` form, servings must be positive, and lookback cannot be negative |
| `src/recipes/mod.rs` | Declares `model`, `write`, `read`, `editors`, `image` and `standing`, and re-exports the public functions and records |
| `src/recipes/model.rs` | `RecipeInput` (title, servings, `LineInput`s, `StepInput`s with `line_positions`, `category_ids`) and what the store returns: `Recipe` (lines and steps in order, owner, editors, `categories`, `image`), `RecipeLine`, `RecipeStep` (`line_ids`), `RecipeSummary` (with `image`), `Person`. Both recipe records carry the viewer's view of ratings: `average` (one decimal, `None` without ratings), `rating_count`, `my_stars` and `favourite` |
| `src/recipes/write.rs` | `create(pool, owner, input, now)`, `update(pool, id, input)` and `delete`. Create and update run in one transaction and replace categories, lines and steps wholesale. Unknown category ids are ignored. An ingredient name matches a name or alias case-insensitively, and an unknown name creates an ingredient with `pending_review`. Blank names become no ingredient |
| `src/recipes/mealie.rs` | `create_mealie` writes the source Mealie id on recipe creation, and `imported_mealie_ids` returns the ids already stored |
| `src/recipes/read.rs` | `get(pool, id, viewer)` (`Option<Recipe>`), `list(pool, category, favourites_only, search, viewer)` (ordered by title; `category` keeps the recipes filed under it when `Some`, `favourites_only` keeps the viewer's favourites, `search` keeps the recipes whose title or an ingredient's name or alternative name contains it, ignoring case for ASCII letters, with `%`, `_` and `\` literal through `ESCAPE`; a blank search is none; all combine) and `list_users`. Averages are rounded to one decimal in SQL |
| `src/recipes/standing.rs` | `rate(pool, recipe, user, stars)` (upsert; `InvalidStars` for anything outside 1 to 5 before the database is touched; `false` for a missing recipe), and `set_favourite(pool, recipe, user, bool)` (`false` for a missing recipe). Both writes run in `begin_write` |
| `src/recipes/image.rs` | `set_image(pool, id, Option<&str>)`: sets or clears the file name of the recipe's photo (the `image` column of `recipes`, from migration 0004) and returns the name it replaced, `None` when there was none or the recipe is missing |
| `src/recipes/editors.rs` | `add_editor` (false for a missing recipe or user; adding twice changes nothing) and `remove_editor` |
| `migrations/` | SQL migrations, applied in file-name order. `0008_planner.sql` stores household planner settings and weekday categories, and dated recipe or rest entries; deleting a household cascades, deleting a category clears weekday choices, and deleting a recipe turns its plan entry into rest |
| `tests/open.rs` | Fresh and existing databases, pragmas, nested data dir |
| `tests/concurrent_writes.rs` | Many concurrent creates and renames all succeed |
| `tests/recipes_images.rs` | `set_image` returns the replaced name, clears it, reads back through `get` and `list`, and does nothing for a missing recipe |
| `tests/users.rs`, `tests/sessions.rs`, `tests/login_attempts.rs`, `tests/households.rs` | Store behaviour; `tests/common` opens a migrated database in a temp dir |
| `tests/catalog_ingredients.rs`, `tests/catalog_aliases.rs`, `tests/catalog_merge.rs`, `tests/catalog_sections.rs` | Name and alias rules in every case, `InUse`, the review queue, merging (recipe lines and shopping items inserted with SQL, both re-pointed), section ordering and renumbering |
| `tests/catalog_categories.rs` | Category order by name, unique names regardless of case, rename, and delete taking the category off its recipes |
| `tests/planner.rs` | Settings defaults and per-household round trips; recipe and rest entry upserts, inclusive ordered ranges, recipe history and household isolation; invalid dates and servings |
| `tests/recipes_categories.rs` | Categories stored by name with unknown ids ignored, replaced on update, the list filtered by category |
| `tests/recipes_round_trip.rs` | Lines and steps round-trip in order with step-line links, unknown ingredients created pending, alias and case-insensitive resolution, list order, `UnknownLine` |
| `tests/recipes_update.rs` | Update replaces lines and steps, a missing recipe is unchanged, a bad link changes nothing |
| `tests/recipes_editors.rs` | Editors added, listed and removed; missing recipe or user refused; user list by name |
| `tests/recipes_delete.rs` | Delete removes lines, steps, step links, editors, ratings and favourites (ratings and favourites inserted with SQL) |
| `tests/recipes_ratings.rs` | A user's rating changes in place, out-of-range stars are refused before storing, rating a missing recipe changes nothing, the average and count cover every user |
| `tests/recipes_favourites.rs` | Favourites belong to one user and toggle, a missing recipe changes nothing, the list filters by the viewer's favourites alone and with a category |
| `tests/shopping.rs` | Recipe lines scaled and skipped, unknown recipe, manual items trimmed and edited, blank refused, edit refuses ingredient items, checked items last, clearing, other households refused in every change, household delete removes items |
| `tests/recipes_search.rs` | Search by title, ingredient and alternative name, case-insensitively, `%`, `_` and `\` literal, blank search is none, and search combined with the category and favourites filters |

Migrations:

| File | Tables |
| --- | --- |
| `0001_households_and_users.sql` | `households`, `users`, `user_households` |
| `0002_sessions.sql` | `login_attempts` (pending logins by `state`), `sessions` (by token hash, cascade on user delete) |
| `0003_active_household.sql` | `users.active_household_id` (set null when the household is deleted) |
| `0004_catalog_and_recipes.sql` | `store_sections`, `ingredients` (`pending_review`), `ingredient_aliases` (cascade with the ingredient), `recipes`, `recipe_lines` (`ingredient_id` restricts deletes of used ingredients) |
| `0005_recipe_details.sql` | `recipe_editors`, `steps` (position, text, optional duration), `step_lines` (step to line), `ratings` (one per user and recipe, 1 to 5 stars) and `favourites` (one per user and recipe); both cascade with the recipe and the user |
| `0006_categories.sql` | `categories` (unique name regardless of case), `recipe_categories` (cascade with the recipe and the category) |
| `0007_shopping.sql` | `shopping_items` (cascade with the household; ingredient items cascade with the ingredient, recipe items set null with the recipe; exactly one of ingredient or text) |

Migrations are embedded at compile time by `sqlx::migrate!()`. To add one, create `migrations/NNNN_description.sql` with the next number. Never edit an applied migration; add a new one. Tables are `STRICT`.

## crates/crabbage

A Leptos 0.8 app. The `ssr` feature builds the server binary and the server-rendered router, and depends on `crabbage-store` and on `crabbage-import` (the `recipes/import.rs` mapping of its pipeline). The `hydrate` feature builds the WASM library that hydrates the page in the browser. The two features are independent and are never enabled together by the build.

| Module | Responsibility |
| --- | --- |
| `src/lib.rs` | Crate doc, module declarations, re-exports, and the `hydrate` entry point for the browser |
| `src/app.rs` | `App` component (title, heading and sign-out button, all from `t`) and `shell`, the HTML document around it. `shell` sets `<html lang>` from the render's language and links `/pkg/crabbage.css` |
| `style/main.css` | The one base stylesheet, hand-written, no framework. cargo-leptos builds it from `style-file` in `[package.metadata.leptos]` into `/pkg/crabbage.css`. Mobile first: system fonts, 18px base, a 40rem body, colours as custom properties on `:root` with a dark override under `prefers-color-scheme: dark`, 44px controls that go full width on phones and sit side by side from 40rem |
| `src/i18n/` | Translated text, see below. Compiled for both `ssr` and `hydrate` |
| `src/config/` (`ssr`) | Settings from environment variables: `Config::from_env` and `from_lookup`, `AiConfig`, `ConfigError` listing every problem. `CRABBAGE_LANGUAGE` is parsed into `i18n::Language`. `import_allow_private` is `CRABBAGE_IMPORT_ALLOW_PRIVATE` (`true` or `false`, default `false`, anything else is a problem): whether recipe imports may fetch private network addresses. `mod.rs` re-exports; `settings.rs`, `vars.rs` (lookup and parsing), `error.rs`, `tests.rs` |
| `src/state.rs` (`ssr`) | `AppState`: leptos options, pool, HTTP client, the configuration, the in-memory `PhotoDrafts`, `ShoppingHub` and Mealie migration status, the lazily discovered OIDC client (`oidc()`), the admin group and household prefix used on login, and the language. `FromRef` gives leptos_axum its `LeptosOptions` |
| `src/household.rs` (`ssr`) | `POST /household` (form field `household_id`): sets the signed-in user's active household. 303 to `/` on success, 403 when the user is not a member |
| `src/server.rs` (`ssr`) | `router(options, &Config, pool)`: builds `/healthz`, the `auth` routes, the `household` route, the `api` route, the `images` route, the recipe photo, photo import, rating, shopping and Mealie migration routes, the Leptos routes and the site files as fallback. The Leptos routes provide the pool, the configured language, the data directory and the configuration as context, the error fallback the language. The session middleware wraps all of them |
| `src/shopping/hub.rs` (`ssr`) | `ShoppingHub`: lazily creates a per-household Tokio broadcast sender and publishes data-free change notifications |
| `src/shopping/live.rs` (`ssr`) | `GET /shopping/events`: sends the household's serialized shopping list immediately and after notifications, rereading after lag, with SSE keep-alive. Forms add text, set checked state for one or several household-owned ids, remove items, and clear checked items; successful changes publish and redirect to `/shopping` |
| `src/shopping_page.rs` | `/shopping` page: loads the active household list and ordered section names through a server function, merges ingredient rows separately within checked state using `crabbage_domain::shopping::merge`, keeps text rows separate, groups unchecked rows by store section with `Other` last and checked rows in a final group, and checks every source id in a merged row. Refreshes from shopping SSE `list` events after hydration |
| `src/plan_page.rs` | `/plan` page: shows Monday through Sunday for the requested week (or current week), recipe and cook links with planned servings, per-day recipe/rest and servings forms, generation, and category and lookback settings; data is loaded for the active household |
| `src/planner.rs` (`ssr`) | Planner forms: generate only missing household dates through the domain planner using candidates and existence checks from `crabbage-store`, add a selected week to shopping, reroll a weekday category while avoiding the current recipe when possible, assign recipes or rest days, update servings, save weekday categories and validate dates, ranges and recipe ids. Changes to plan entries and shopping generation publish through `ShoppingHub` |
| `src/api.rs` (`ssr`) | `POST /api/{*fn_name}`: serves every `#[server]` function, providing the pool, the language and the configuration as context. Behind the session check |
| `src/access.rs` (`ssr`) | `signed_in` (the pool and the session user id), `role` (the `Role` of a user), `require_admin` (the pool, or a 403), `refuse` (sets 403 and returns the error with its message, shared by every refusal), `store_failure`, `clean` (trimmed, non-empty), `ensure_found`, `data_dir` (the data directory from context), `internal_error` (a status of 500 for a plain HTTP handler) |
| `src/images/` (`ssr`) | Recipe photos, see below |
| `src/notice.rs` | How pages show a failed action or load: `failure`, `error_notice` (`.notice.error`), `message` |
| `src/route.rs` | `route_id`: the `:id` of the current route as a number, or `None`. Used by the recipe, cook and ingredient pages |
| `src/app.rs` | `App`: the `Router` with `/` (home: the recipes and shopping links for everyone, admin links for admins), `/admin/ingredients`, `/admin/ingredients/:id`, `/admin/sections`, `/admin/categories`, `/admin/review`, `/admin/mealie`, `/recipes`, `/recipes/new`, `/recipes/:id`, `/recipes/:id/cook`, `/recipes/:id/edit` and `/shopping`, and the sign-out button (`.secondary`). `shell` is the HTML document |
| `src/user.rs` | `current_user` server function: `SignedInUser` (name and `is_admin`) |
| `src/catalog/` | Catalog server functions and their records, see below. `categories.rs` holds the admin endpoints for recipe categories. `types.rs` holds `IngredientRow` and `SectionRow` (both builds), `convert.rs` the store conversions (`ssr`) |
| `src/recipes/` | Recipes: server functions, records and pages, see below |
| `src/admin/` | The admin pages: `ingredients/` (`list.rs`: the add form, the search on `q` and one row link per ingredient; `detail.rs`: one ingredient's rename, alternative names, store section, merge and delete, with merge and delete asking first through `?confirm=`), `sections.rs`, `categories.rs` (add, and a card per category with rename and delete), `review.rs`, and `mealie/` (`page.rs`: the URL and password token form and live progress/summary; `runner.rs`: imports Mealie catalog data once, creates pending ingredients and categories, imports recipes in a spawned task, records per-recipe outcomes and skips stored Mealie ids), each showing a not-allowed message to non-admins. `mod.rs` declares them and re-exports the pages |
| `src/auth/` (`ssr`) | Sign-in and sessions, see below. `mod.rs` declares and re-exports; `SessionUser` (the signed-in user id, set in request extensions) is re-exported from the crate root |
| `src/main.rs` (`ssr`) | Server binary: reads `Config` from the environment, opens the database in `data_dir` with `crabbage_store::open` (migrations run here), binds `site_addr`, builds `router` and serves it. It starts without the provider. Without `ssr` it is an empty stub |

### Translations (`src/i18n/`, `locales/`)

`locales/en.json` and `locales/nl.json` are flat JSON objects of dotted keys to text (`"app.title": "crabbage"`), with `{name}` placeholders. Both are embedded with `include_str!`. They must have the same key set.

| Module | Responsibility |
| --- | --- |
| `src/i18n/mod.rs` | Module declarations and re-exports |
| `src/i18n/language.rs` | `Language` (`En`, `Nl`), re-exported from the crate root. `code` and `from_code` map to and from `en` and `nl` |
| `src/i18n/messages.rs` | `Messages`: one language's keys, parsed once per language from its embedded file. `get` returns the key when missing, `format` fills `{name}` placeholders and keeps unknown ones |
| `src/i18n/context.rs` | `messages()` (the current language's `Messages`), `t(key)`, `t_with(key, args)` (fills `{name}` placeholders) and `current_language()` read the language from leptos context, English when none is provided. `document_language()` (`hydrate`) reads the `lang` attribute of the page |
| `src/i18n/tests.rs` | Every file in `locales/` parses, has the English keys and a language code as its name; every key is translated; formatting; language codes |

The language flows from `Config` to `AppState`, then to the Leptos routes and the error fallback, which provide it as context for each render. `shell` sets `<html lang>` from it and `App` reads the text with `t`. The browser takes the language from the `lang` attribute of the page before it hydrates.

To add a language: add `locales/<code>.json` with the same keys, add a `Language` variant with its code in `from_code` and `code`, and add its arm in `Messages::for_language`, which embeds the file. Visible text in `view!` blocks must come from `t`; `tests/strings.rs` fails on string literals used as text children.

### Styling

Pages get their look from `style/main.css` and never from inline styles. Use the plain elements (`button`, `input`, `select`, `label`, `form`) and these shared classes:

| Class | Use |
| --- | --- |
| `.card` | A bordered section: a store section, category or review row, or a question with its buttons |
| `.ingredient-list` | The ingredient list: one bordered box with a divider between rows, each row a `.ingredient-row` link of at least 44px |
| `.badge` | A small rounded tag, such as Waiting for review or a recipe's category |
| `.filters` | The category filter links above the recipe list, each at least 44px tall |
| `.stepper` | The servings stepper of a recipe page: its minus, plus and Reset links, each at least 44px |
| `.cook-card`, `.cook-step`, `.cook-nav`, `.cook-dots`, `.cook-dot` | The cook page: the card holding one step (a touch swipe moves between steps), the step at 1.5rem, the Previous and Next links side by side at least 56px tall, and the step dots at 44px, the done ones filled and the current one outlined |
| `.timer-starts`, `.timer-tray`, `.timer-list`, `.timer-row`, `.timer-label`, `.timer-time` | The cook page's timers: the start buttons under a step's text, and the tray that sticks to the bottom of the viewport (a surface with a top border, hidden while no timer runs), one row per timer, the finished ones in the accent colour |
| `.row` | A horizontal group that wraps, such as the up and down buttons of a section |
| `.notice`, `.notice.error` | A status message; `.error` for failed actions, failed loads and the not-allowed message |
| `.nav` | The home page admin links as large tappable blocks |
| `.danger` | A button that deletes |
| `.secondary` | A button that is not the main action, such as sign-out and remove |

Forms stack their fields on phones and sit side by side from 40rem. Message texts come from `t` as everywhere else.

### Sign-in (`src/auth/`)

| Module | Responsibility |
| --- | --- |
| `oidc.rs` | `Oidc`: discovery of `OIDC_ISSUER_URL`, the authorization request (code, PKCE S256, nonce, state, scopes `openid profile groups`), the code exchange and the ID token check. Name comes from `name`, then `preferred_username`, then the subject. The `groups` claim is returned in `Identity`; a missing claim counts as no groups. The HTTP client does not follow redirects |
| `login.rs` | `GET /auth/login`: stores a login attempt, sets the `crabbage_login` cookie (the state, `Path=/auth/`, 10 minutes) and redirects (303) to the provider |
| `callback.rs` | `GET /auth/callback`: requires the `crabbage_login` cookie to equal `state` (otherwise 400, attempt kept), takes the attempt by `state` (unknown or expired: 400), exchanges the code, verifies the ID token and nonce (failure: 400), upserts the user, derives the membership from `groups` and syncs it (`households::sync_membership`), creates a session, sets the session cookie, clears the login cookie and redirects to `/` |
| `logout.rs` | `POST /auth/logout`: deletes the session row, clears the cookie, redirects to `/` |
| `middleware.rs` | `require_session`: every path except `/healthz`, `/auth/*` and `/pkg/*` needs an unexpired session, otherwise 303 to `/auth/login`. A valid session adds `SessionUser` to the request |
| `cookie.rs` | `Cookie` (name, path, max age) with two instances: `SESSION` (`crabbage_session`, `Path=/`, 30 days) and `LOGIN` (`crabbage_login`, `Path=/auth/`, 10 minutes). Each reads its value from a request and builds the `Set-Cookie` for set or clear. All are `HttpOnly; Secure; SameSite=Lax` |
| `session.rs` | Session lifetime, unix time, and the 32-byte base64url session token |
| `error.rs` | `AuthError`: 400 with a message for rejected logins, 500 for store or randomness failures, 503 when the provider cannot be discovered |

The provider is discovered by `AppState::oidc()` on the first request that needs it, and the client is kept only after discovery succeeds. While the provider is unreachable, `/auth/login` and `/auth/callback` answer 503 and the next request tries again. `/healthz` and the pages that need no session do not depend on the provider.

Flow: the login attempt holds `state`, `nonce` and the PKCE verifier in `login_attempts`. The browser also holds the state in the `crabbage_login` cookie, so only the browser that started the login can complete it (login CSRF). The callback checks the cookie before it takes the attempt, then the attempt is deleted, and the ID token must match the stored nonce. The cookie holds the raw token; `sessions` stores only its SHA-256 hash.

Roles and households follow the `groups` claim on every login: the callback replaces the user's admin flag and household memberships, so changes at the provider take effect on the next login. Each `crabbage-household-*` group is a household, created on first sight and shared by its members. The active household defaults to the first by group name, and a user keeps their choice across logins while they stay a member. Users without a household can use recipes; the shopping list and planner need an active household (`has_household_features`).

### Catalog and server functions

Admin pages call server functions. Each is a `#[server(prefix = "/api", endpoint = "...")]` function, so it is served at `/api/<endpoint>`. Input is form-encoded (`PostUrl`, the default) and output is JSON. Every function has an explicit `endpoint`, so the URL does not depend on the function name. The function name in PascalCase is the marker type that `ServerAction::<Marker>` uses in a page.

Each server function in `src/catalog/` begins with `crate::access::require_admin()`, which returns the pool. Errors go through `store_failure`, so each message is translated, and a missing row through `ensure_found`. A non-admin gets a 403 (`ResponseOptions` is set from the context) with the not-allowed text. The body of a server function is compiled for `ssr` only, so anything it needs from the store is imported inside the body. The client build keeps the signature and the record types, which must serialise on both sides.

To add an endpoint: add the store function with its tests in `crabbage-store`, add the `#[server]` function to the matching file in `src/catalog/` with `require_admin`, add a `ServerAction` and its form to the page in `src/admin/`, add the messages to both locale files and cover the endpoint in `tests/server/catalog.rs`.

Permissions: the session user's `is_admin` (set from the admin group on every login) becomes a `Role`, and `can_manage_catalog` decides. The server functions enforce the rule; the page only decides what it shows.

### Recipes (`src/recipes/`)

Recipe server functions are `#[server(prefix = "/api")]` like the catalog ones, and input is form-encoded. Every change begins with `access::signed_in`, then `rights::permit` loads the recipe, reads the user's role and checks the domain rule (`can_edit_recipe`, `can_delete_recipe` or `can_manage_editors`); a refusal is 403 with the translated `recipe.not_allowed`. `get_recipe` returns the record with `can_edit`, `can_delete` and `can_manage_editors`, which the pages use to show their buttons.

Form encoding: the recipe form sends one hidden field, `draft`, holding the whole form as JSON. The page's signals (`pages/form/draft.rs`) hold each field as text, and `Draft::json` serialises them into `RecipeDraft` (both builds). The server parses it with `serde_json`, and `recipes/draft.rs` checks every field and turns it into the store's `RecipeInput`. Rows that are entirely blank are skipped. `create_recipe(draft)` and `update_recipe(id, draft)` take the JSON string, because `ActionForm` cannot send nested rows as urlencoded fields. Fields keep their text, so the server reports bad values in words (`recipe.error.*`).

| Module | Responsibility |
| --- | --- |
| `types.rs` | `RecipeSummaryRow`, `RecipeDetail` (with its categories and the three rights), `LineRow`, `StepRow` (with `line_positions`, the positions of its lines in the recipe), `PersonRow` and the draft records `RecipeDraft` (with the category ids), `LineDraft`, `StepDraft` (text as typed, with `line_positions` into the form's lines) and `ImportPreview` (the draft, the photo address, the source address, the unresolved ingredient names and the AI line) |
| `units.rs` | `unit_text(messages, symbol)`: the word of a unit from its `unit.*` locale key (`g`, `tl`, `a pinch`, ...). Unknown symbols show as typed |
| `line_text.rs` | `line_text(line, messages)`: a line's readable text. A count shows no unit word (`2 Egg`), a unitless unit follows the ingredient (`Salt, to taste`), the note follows after a comma. Numbers keep at most two decimals without trailing zeros, with the `number.decimal` locale separator |
| `list.rs`, `read.rs`, `write.rs`, `editors.rs`, `ingredients.rs`, `categories.rs` | Server functions: `list_recipes(category, favourites_only, search)`, `get_recipe`, `create_recipe`, `update_recipe`, `delete_recipe` (also removes the recipe's photo file), `add_recipe_editor`, `remove_recipe_editor`, `list_users` (any signed-in user, for the editor choice), `ingredient_names` (any signed-in user, for the ingredient suggestions) and `list_recipe_categories` (any signed-in user, for the filter and the form) |
| `photo.rs` (`ssr`) | `POST /recipes/:id/image` (multipart field `image`, 303 to the edit page) and `POST /recipes/:id/image/delete`. Plain HTML forms, so no JavaScript is needed. Missing recipe 404, a user who may not edit 403, a body over 10 MiB 413, a file that is not a JPEG, PNG or WebP 415, no `image` field 400. Replacing and deleting remove the old file after the store points at the new one |
| `rating.rs` (`ssr`) | `POST /recipes/:id/rating` (field `stars`, 1 to 5, else 400) and `POST /recipes/:id/favourite` (field `on`, `true` or `false`, else 400). Plain HTML forms that answer 303 to the recipe page; a missing recipe 404. Any signed-in user may rate or favourite any recipe they can see |
| `rating_text.rs` | `rating_text(average, count)`: `★ 4.5 (2 ratings)` with the singular and plural keys, or `No ratings yet`. The decimal separator comes from the locale |
| `rights.rs` (`ssr`) | `Rights::of` and `permit` (the rule per change), `load` (the recipe or the missing-item error) |
| `convert.rs` (`ssr`) | Store records to the page records. A step's line ids become positions in the recipe's lines |
| `import.rs` (`ssr`) | `recipe_draft(imported)`: an `Imported` recipe (from a web page) to the wire `RecipeDraft` the form uses: amounts as the form expects them (`200`, `0.5`), unit symbols, ingredient names, notes, steps with their `line_positions` (positions outside the lines dropped), servings from the import or `2`. `preview(imported, source, ai_enabled)`: an `ImportPreview` whose `unresolved` names are those of the lines with no catalogue ingredient, once each, and whose `ai_error` is the translated AI failure, or, when the AI is off and something is unresolved, the AI-off line. `read_page(pool, config, url)`: `fetch` and `extract` under `Limits` with `allow_private` from the configuration, the candidates from the store's ingredients with their aliases, an `AiClient` from `config.ai`, then `complete` with the instance language; an `ImportError` becomes a translated message (`import.error.*`). Its tests are in the module, since `recipes` is private to the app, and the pending-review one opens a temporary store |
| `import_fns.rs` | Server functions: `import_recipe(url)` (any signed-in user; the `ImportPreview` of `read_page`) and `save_import(draft, image)` (the draft is parsed and created like `create_recipe`, the signed-in user is the owner; then the photo at `image`, if any, is attached by `import_photo`, and a photo failure does not fail the save), `ai_enabled()` (whether the AI is configured) and `photo_preview(token)` (the stored `PhotoDrafts` preview for the signed-in user, else the translated expiry message) |
| `import_photo.rs` (`ssr`) | `attach(pool, config, id, url)`: downloads the photo with `fetch_bytes` (the upload size limit, private addresses per the configuration), accepts only a JPEG, PNG or WebP by its first bytes, writes it with `images::write` and sets it with `set_image`; returns whether the recipe has the photo now |
| `import_photos.rs` (`ssr`) | `POST /recipes/import/photos` (multipart field `photo`, up to 4; body limit 4 × the upload size plus framing): signed-in users only, 404 when the AI is not configured. Files over the upload size or not a JPEG, PNG or WebP are skipped; none left redirects (303) to `/recipes/import?photo_error=invalid`. `from_photos` reads the rest against the catalogue; an AI error redirects to `?photo_error=ai`, success stores the `preview` (source `N photo(s)`, no photo) in `PhotoDrafts` and redirects to `?photos=<token>`. Plain HTML form, no JavaScript needed |
| `photo_drafts.rs` (`ssr`) | `PhotoDrafts` (in `AppState` and provided as context): a `Mutex<HashMap>` of token (32 hex from `getrandom`) to user id, `ImportPreview` and creation time. `insert` prunes entries older than 1 hour and drops the oldest beyond 100; `get` returns a draft only for its user within the hour |
| `draft.rs` (`ssr`) | `parse`: JSON to `RecipeInput`, with the validation. A step's `line_positions` follow their lines past blank rows, which are not stored, and links to missing lines are dropped: a non-empty title, servings of at least 1, amounts finite and not negative, a unit that is a `Unit` symbol, step text and whole minutes of at least 1 |
| `servings.rs` | `shown_servings(requested, original)`: the `servings` value of the address when it is 1 to 99, else the recipe's own count. `scale_lines(lines, from, to)`: each line's amount scaled with `crabbage_domain::scaling::scale` (an amount without a unit counts in pieces; a line without an amount or with a unit that is not a measure, such as `to taste`, is kept) |
| `duration.rs` | `duration_text(seconds)`: a step's duration as whole minutes rounded up, with the `unit.minutes` word. Shared by the recipe page and the cook page |
| `cook/` | The cook page at `/recipes/:id/cook`, one step at a time. `mod.rs` declares the parts (`alert`, `tray_client` and `wake_client` are `hydrate` only, `wake` is `hydrate` and `ssr` tests). Its timers are described in the rows below. Under the step's text, `You need` lists the lines the step links to, scaled to the shown servings (`page.rs`, `needed`). `mod.rs` declares the parts and re-exports `CookPage` and `start_href`. `page.rs`: `CookPage` (one `Suspense` returning `Suspend`, the recipe from `get_recipe`, the title, `Step n of total` with a `<progress>`, the step in `.cook-step` with its duration, a `.cook-done` mark, a Mark done or Not done link with `aria-pressed`, Previous and Next or Done cooking, and one dot link per step with `aria-current="step"` and class `done`; a recipe without steps shows its message; the Start timer buttons follow the duration, and `TimerTray` is outside the `Suspense` so it stays across steps). `query.rs`: `current_step` (1-based, invalid or zero is 1, too large is the last step), `done_steps` (the `done` list: invalid, out-of-range, duplicate entries dropped, sorted), `toggled`, `done_text`. `links.rs`: `Place` builds every step address with `servings` (when given) and `done`, and `start_href` for the recipe page's Start cooking link. `swipe.rs` (`ssr` tests and `hydrate`): `swipe_target(start_x, end_x, start_y, end_y)` gives `Previous` for a rightward and `Next` for a leftward move of at least 50px that is longer than the vertical move, and under `hydrate` `swipe_card` wires the touch events to `use_navigate`, with the card's `data-prev` and `data-next` addresses |
| `cook/timers.rs` | The timers, pure and in both builds. `Timer` (`id`, `label`, `ends_at_ms` in milliseconds since the epoch, so sleeping does not stop them): `start` (the same label twice makes two timers), `cancel`, `remaining_ms` (clamped to 0), `finished`, `soonest_first`, `clock` (`mm:ss` or `h:mm:ss`, rounded up), `step_durations(text, stored)` (the stored duration, then those named in the text, each once). `encode` and `decode` (JSON for sessionStorage, `hydrate` and tests) |
| `cook/tray.rs` | `TimerTray` (both builds): the `section.timer-tray` live region, rendered empty on the server, one row per timer soonest first (label, remaining time or Done!, Cancel or Dismiss). `start_button`: the `Start timer mm:ss` button of a step, with `data-seconds`, `data-label` (`Step N`) and an aria-label naming the step. The list is an `RwSignal` created by `CookPage` and kept while the step changes; each change is saved to sessionStorage through `save` |
| `cook/tray_client.rs` (`hydrate`) | `attach`: after hydration, loads the list from sessionStorage (key `crabbage.timers`) and runs a one-second interval that updates `now` and alerts once for each timer that has ended. `now_ms` reads the wall clock through `Date.now`. `save` writes the list |
| `cook/alert.rs` (`hydrate`) | `unlock` creates the Web Audio context during the tap that starts a timer, `alert` plays three 0.5s beeps and calls `navigator.vibrate([300, 150, 300])` where it exists, ignoring errors |
| `cook/wake.rs` (`hydrate` and tests) | The screen wake lock, pure. `WakeApi` (`supported`, `request`, `release`) and `WakeLock<A>`: `enter` and `leave` set whether the lock is wanted, `visibility_changed(visible)` drops the held mark when hidden (the browser drops the lock) and requests again when shown, `lock_lost` clears the held mark. It never requests while held, and makes no calls when the browser does not support it |
| `cook/wake_client.rs` (`hydrate`) | `attach`: called by `CookPage` on mount. Enters the wake lock, follows `visibilitychange` on the document, and leaves on cleanup. `Browser` implements `WakeApi` through the `wakeSupported`, `wakeRequest` and `wakeRelease` inline JS functions, which keep the sentinel, catch every rejection and clear it on the `release` event. Screen Wake Lock is not in web-sys, so it is inline JS |
| `pages/` | `RecipesPage` (an Import link beside New recipe; a search form on `q` that keeps the category and favourites as hidden fields; the filter links All, Favourites (`?favourites=1`) and one per category, each keeping `q` (`Filters::href`, values percent-encoded), the active ones marked `aria-current`; the empty text names the search when there is one; a card per recipe with its rating and a favourite badge; New recipe), `RecipePage` (the view with its categories as badges, the servings stepper and the scaled lines, the rating section, the Start cooking link to the cook page, Edit when allowed, Delete when allowed and only after `?confirm=delete` asks the question), `RecipeNewPage` and `RecipeEditPage` (the form; the edit page also shows `EditorsSection` to owners and admins). The id comes from `src/route.rs` |
| `pages/rating_section.rs` | The rating on the recipe page: the summary, five star buttons (each a button named `stars` in a form posting to the rating route, the viewer's own one with `aria-pressed="true"`) and the favourite button, which posts `on` with the opposite of the current state |
| `pages/form/` | `draft.rs` (the signals and the JSON, `from_wire` also for the import page; a step's links are held as the keys of its line rows, so removing a line drops its links, and `positions_of` turns them into positions when the draft is read), `fields.rs` (`DraftFields`: title, servings, ingredient rows with a unit select and a datalist of the known names (an optional `unresolved` list of names puts the class `unresolved` on the rows that hold one of them, styled in `main.css` with an accent left border), step rows with a text area and minutes and, when the recipe has lines, an `Ingredients for this step` fieldset with a checkbox per line; "Add" buttons append rows and "Remove" buttons drop them; `CategoryBoxes`: a checkbox per category), `page.rs` (the two pages) |
| `pages/import.rs` | `RecipeImportPage` at `/recipes/import`: a GET form with the `url` input; when the address has `url`, one `Suspense` returning a `Suspend` calls `import_recipe` and shows the recipe: where it was read from, a notice naming the unresolved ingredients, the AI line, `DraftFields` prefilled with `Draft::from_wire` (the lines of the unresolved ingredients get the class `unresolved`; units, ingredient names and notes are editable per line), the photo address as an image preview with a hidden `image` field, and Save, which runs `save_import` and opens the new recipe. When the AI is configured it also shows a plain multipart form (`photo`, `accept="image/*"`, `multiple`, no `capture`) posting to `/recipes/import/photos`; `?photos=<token>` shows the stored photo import in the same review and save form, and `?photo_error=invalid|ai` a translated notice. Failures show as a notice with the translated message. It works without JavaScript up to saving, like the other recipe forms |
| `pages/editor_list.rs` | The editors of a recipe. Add and remove return the new list, which the section shows in place, so the form on the page keeps what was typed |
| `pages/photo_card.rs` | The photo card of the edit page: the current photo, an upload form (`accept` for the three types) and a remove form when there is a photo. `RecipePage` shows the photo full width above the ingredients |
| `pages/stepper.rs` | `servings_stepper(id, shown, original)`: the minus and plus links (`?servings=N-1` from 2 to 99, `N+1` below 99) with the locale's aria-labels, the count, and Reset when the count is not the recipe's own |

Each page loads its data in one `<Suspense>` returning `Suspend`. The step-to-line links (`step_lines`) travel in the JSON draft as positions, so saving the form keeps them. The form has no field names for them: like the rest of the form, they are sent only through the hidden `draft`.

Pages load data in one `<Suspense>` whose closure returns a `Suspend` future that awaits the server functions. Pages do not create `Resource`s: `reactive_graph` keeps the current transition in a process-wide static that `leptos_router` sets around each route, so resources of concurrent server renders can register with another request's transition and never resolve.

### Photos (`src/images/`, `src/recipes/photo.rs`)

A recipe has at most one photo, stored as a file in `<data_dir>/images/`. The file is named `<32 random hex>.<jpg|png|webp>`, so the name tells the type and nothing from the client is used. The type comes from the first bytes: `FF D8 FF` (JPEG), the PNG signature, or `RIFF....WEBP`.

| Module | Responsibility |
| --- | --- |
| `kind.rs` | `Kind` (`Jpeg`, `Png`, `Webp`): `detect` from the signature, the extension, the media type, and `from_extension` |
| `naming.rs` | `generate(kind)`: a new name from 16 random bytes (`getrandom`). `kind_of(name)`: the type of a name that has exactly the generated form, otherwise `None`, so a path can never reach the disk |
| `files.rs` | `write` (creates the directory, writes `.<name>.part` and renames it into place, so a half-written file is never served), `read`, `remove` |
| `serve.rs` | `GET /images/:name`: 404 for a name that is not generated or a missing file, otherwise the bytes with the media type, `Cache-Control: private, max-age=31536000, immutable` and `X-Content-Type-Options: nosniff`. The session check runs first, so signed-out requests are redirected to sign-in |

The upload route's body limit is `DefaultBodyLimit` of 10 MiB (`MAX_UPLOAD_BYTES`), which includes the multipart framing. The `multipart` feature of `axum` is enabled in the workspace.

### Tests

| Path | Responsibility |
| --- | --- |
| `tests/server/main.rs` | `healthz`, the rendered shell for a signed-in user in English and in Dutch (`lang`, title, sign-out text), the stylesheet link, and the shared harness module declarations |
| `tests/stylesheet.rs` | String checks on `style/main.css`: the dark colour scheme, a 44px minimum height for inputs and buttons, a `max-width` on `body`, `overflow-wrap: anywhere` on `body`, `max-width: 100%` on `img`, and no fixed `width` in px above 360 |
| `tests/strings.rs` | No string literal is used as text in a `view!` block under `src/`. Tests the heuristic on inline samples |
| `tests/server/login.rs` | Unauthenticated redirects, the authorization request, the callback (cookie attributes, unknown state, single use, wrong nonce), the stored name (`name`, `preferred_username`, subject, changes on the next login) |
| `tests/server/session.rs` | Logout deletes the row and clears the cookie; expired and unknown sessions require a new login |
| `tests/server/household.rs` | Group claims on login (admin, households, changes on the next login), switching the active household, 403 for non-members, a user without households |
| `tests/server/catalog.rs` | Admin create, rename and delete; members refused on every endpoint with 403; alias uniqueness through the API; sections created and reordered; categories created, renamed and deleted, a taken name refused; the review queue; merge through the API |
| `tests/server/recipes.rs` | Recipe endpoints: a member creates and gets a recipe back, another member views but gets 403 on change, a listed editor changes but not deletes or manages editors, a removed editor is refused, an admin changes and deletes anyone's, invalid input is refused in words (English and Dutch), the list and the user and ingredient names, editors filing a recipe under zero or more categories |
| `tests/server/recipe_images.rs` | Photos over HTTP: each type is stored under a generated name and served with its type and cache header, a GIF, a text file or a missing `image` field is refused and writes nothing, an oversized upload is 413, only editors may upload or remove (403, 404 for a missing recipe, sign-in redirect when signed out), an image is not served without a session, bad names are 404 (traversal, wrong length, uppercase, wrong extension, unknown), replacing and removing delete the old file, deleting a recipe deletes its photo, the recipe page shows the photo and the edit page its card |
| `tests/server/recipe_import.rs` | Importing from a loopback fixture server (`CRABBAGE_IMPORT_ALLOW_PRIVATE=true`): the page without `url` shows the form and the list links to it, signed-out users are sent to sign-in, the preview shows the title, units, ingredient names and notes in fields with the unresolved line marked and listed, the AI-off line, and the photo, a failing AI is reported, a page without a recipe and a private address without the flag show their messages, `save_import` creates the recipe for the importer with the edited unit, ingredient and note, a new ingredient pending review and the photo attached, and a missing or non-image photo still saves the recipe without one |
| `tests/server/recipe_photo_import.rs` | Photo import against a loopback AI that records requests: the form (`multiple`, `accept`) only with the AI configured and the route 404 without it, signed-out uploads sent to sign-in, two PNGs make one request with two `image_url` data parts and a draft shown in the form, non-images give `photo_error=invalid`, an AI 500 gives `photo_error=ai`, other users and unknown tokens see the expiry message |
| `tests/server/mealie.rs` | Loopback Mealie API: admin run creates the recipe for its admin owner with pending ingredients and aliases, categories and rating, counts a missing recipe as failed, shows the summary and skips the imported recipe on a second run, and reports an unauthorized token without exposing it |
| `tests/server/recipe_ratings.rs` | Two users rate and the recipe and the list show the average and count, a second rating replaces the first and the viewer's own star is pressed, stars 0, 6, `x` or missing are 400, unknown recipes are 404, signed-out requests are sent to sign-in, favourites are per user and toggle, a bad `on` is 400, and `?favourites=1` lists only the viewer's favourites with its pill marked |
| `tests/server/recipe_step_lines.rs` | Step links through `update_recipe` and `get_recipe`: an update that does not touch the links keeps them, removing a line drops its links and shifts the later ones, a link follows its line past a blank row and a missing position is dropped, the record gives each step its line positions |
| `tests/server/recipe_browse.rs` | Search by title or ingredient (case-insensitive, blank is none), a no-match text that names the search, the filters keeping `q` and `q` keeping the filters, encoded search text in the links, the servings value rescaling the amounts, invalid values (0, abc, 1000, -3, blank) falling back to the recipe's own servings, the stepper links with no minus at 1 and no plus at 99, Reset, and the viewport meta on the list and the recipe page |
| `tests/server/recipe_pages.rs` | The list and its category filter (only the chosen category, the active link marked, the empty text for an unknown category), a recipe page with lines, numbered steps (`2 Egg`, `Salt, to taste`) and category badges, the new and edit forms with a checkbox per category (checked for the recipe's own), a Dutch recipe page with decimal commas and Dutch unit words, Edit and Delete links for the users who may use them, the delete question with `?confirm=delete` and no delete form without it, the editors section for owners and admins, the missing recipe | The edit form's step fieldsets, one checkbox per line with the linked ones checked, and none on the new form without lines.
| `tests/server/ingredient_pages.rs` | The ingredient list rows (one link each, no actions), the search by name and alias (case-insensitive, no-match text), the detail actions, the not-found text, the not-allowed message for members, and the delete and merge questions shown only with `?confirm=delete` and `?confirm=merge` |
| `tests/server/cook_mode.rs` | The cook page: the start timer buttons (text, `data-seconds`, one per duration, a Dutch duration, a stored duration, none without a duration), the tray on every step, progress text and step text, no Previous on step 1, the Next and Previous addresses, Done cooking on the last step, out-of-range and invalid steps clamped, the Mark done and Not done links with `aria-pressed`, done steps kept on other steps, the `done` list deduplicated and sorted with invalid entries dropped, servings kept in every link, `data-prev` and `data-next` on the swipe card, a recipe without steps, unknown or invalid recipes, the signed-out redirect, and the Start cooking link on the recipe page | The lines of a step under `You need`, with the amounts scaled to `servings`, and none for a step without links.
| `tests/server/screenshots.rs` | Ignored. Also writes the import form and the import preview of a fixture recipe with one unresolved line (`member-recipe-import`, `member-recipe-import-preview`, and `member-recipe-import-photos` with the photo form, AI configured). Also writes a finished Mealie migration summary (`admin-mealie`) and the cook page on step 2 with step 1 done (`member-cook`). Writes the rendered pages, including a sample recipe filed under Breakfast with a photo (also the search for `flour` and the recipe at 8 servings), rated by two users (5 and 4) and a favourite of the member, its delete question, the categories, the filtered lists (including the favourites), and the ingredient pages, to `CRABBAGE_SCREENSHOT_DIR` for review screenshots, with `style/main.css` and the photo copied beside them and the links made relative so a `file://` view is styled |
| `tests/server/pages.rs` | Admin pages render for admins (the categories and Mealie pages list their controls), show the not-allowed message to members, the ingredient list's sections, alternative-name counts and pending badge, and the home page links to them for admins only |
| `tests/server/common/` | `Harness`: the router on a temp database (`start_with` sets more environment variables), and `issuer.rs`, a fake provider on 127.0.0.1 with a runtime-generated RSA key (discovery, JWKS, token endpoint). Its ID token carries the nonce and the `groups` claim the test sets |
| `tests/server/discovery.rs` | Lazy discovery: with an unreachable provider sign-in is 503, and once the provider listens on the same address it is 303 |
| `tests/binary.rs` | Runs the server binary: missing config vars and invalid `LEPTOS_SITE_ADDR` or an address already in use exit with an error. With an unreachable issuer the server runs: `/healthz` is 200 and `/auth/login` is 503. Each run uses its own temp `CRABBAGE_DATA_DIR`, and the server process is killed when the test ends |

## xtask

| Module | Responsibility |
| --- | --- |
| `cli.rs` | Command definitions (clap) |
| `task/plan.rs` | Maps a command to a list of actions. Composite commands (`ci`, `pre-commit`) live here |
| `task/run.rs` | Executes actions and reports violations |
| `task/smoke.rs` | Container smoke test for `cargo xtask docker`: the hardened `docker run` step, `/healthz` polling over a raw HTTP/1.1 `GET` (30 s), and the status-line parser. `run.rs` always runs `docker rm -f` afterwards |
| `task/step.rs` | An external process invocation |
| `policy/repository.rs` | Applies every file rule to the tracked files |
| `policy/comments.rs` | Finds plain comments in Rust source |
| `policy/module_root.rs` | Ensures `mod.rs` only declares and re-exports |
| `policy/length.rs` | Maximum lines per Rust file |
| `policy/layout.rs` | Allowed documents, forbidden files (`.env`, keys), size limit |
| `policy/leaks.rs` | Personal emails, home paths, tokens, image metadata, `authors` |
| `policy/identity.rs` | Anonymous git author and committer emails |
| `policy/commit.rs` | Conventional commits, no tool attribution |
| `repo.rs` | Git queries |

To add a check, add a submodule under `policy/`, call it from `repository.rs` (file rules) or `commit.rs` (message rules), and test it in the same file.

To add a task, add a variant to `cli.rs` and map it in `task/plan.rs`. Prefer composing existing steps.

## Container image

`Dockerfile` has five stages. Each platform is built natively on a runner of that platform (`ubuntu-24.04` for amd64, `ubuntu-24.04-arm` for arm64), so no emulated compiler runs:

| Stage | Contents |
| --- | --- |
| `tools` | Rust on Debian trixie, pinned by digest. `musl-tools` from apt, and cargo-leptos from its release archive, checked by SHA-256 |
| `site` | `cargo leptos build --release --frontend-only`: the WASM bundle, JS and CSS (`pkg/crabbage.css`) in `target/site`, copied to `/out/site` |
| `server` | `cargo build` of `crabbage` with `ssr` for the native musl target `$(uname -m)-unknown-linux-musl`. Bundled SQLite is C code compiled with `musl-gcc`; rustls needs no system libraries, so the binary is static |
| `data` | `/data` owned by 65532:65532 |
| final | `FROM scratch`. Copies `/data`, `/site` and `/crabbage`. Sets `CRABBAGE_DATA_DIR`, `LEPTOS_SITE_ROOT`, `LEPTOS_OUTPUT_NAME` and `LEPTOS_SITE_ADDR`. Runs as `65532:65532`, exposes 3000, declares `/data` as a volume |

The cargo-leptos version and its checksum are pinned in `ARG` and in the `tools` stage. CI's `image` job is a matrix with one leg per platform. Each leg builds its platform with its own GitHub Actions cache scope and pushes the image by digest on pushes to `main`. The `manifest` job then combines the digests into `ghcr.io/borrowcheckmate/crabbage:main` and `:sha-<short>`.

## Rules enforced by `cargo xtask policy`

- Rust files have at most 500 lines.
- `mod.rs` contains only `mod` declarations and `pub use` re-exports. `lib.rs` is exempt.
- No plain comments. Use `///` docs. `// NOTE:` is the escape hatch for the rare unavoidable comment.
- Documents live in `README.md`, `AGENTS.md`, `ARCHITECTURE.md`, `CONTRIBUTING.md`, `SECURITY.md`, `CHANGELOG.md` or `.github/`.
- No `.env` files, keys, credentials, OS metadata files or files over 256 KiB.
- No personal emails, home directory paths, tokens or image metadata in any tracked file.
- No `authors` in `Cargo.toml`.

## Lints

All lints are configured once in the workspace `Cargo.toml` and inherited with `[lints] workspace = true`. Clippy `pedantic`, `nursery` and `cargo` are denied, plus selected `restriction` lints (no `unwrap`, `expect`, `panic`, printing macros, indexing). Tests may `unwrap` (see `clippy.toml`). Use `#[expect(lint, reason = "...")]` for justified exceptions; `#[allow]` is denied. Duplicate crate versions that Leptos and sqlx pull in are allowed by name in `clippy.toml` (`allowed-duplicate-crates`) and by version in `deny.toml` (`bans.skip`), each with a reason.
