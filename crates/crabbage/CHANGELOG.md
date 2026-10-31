# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.0](https://github.com/borrowcheckmate/crabbage/compare/crabbage-v0.1.0...crabbage-v0.2.0) - 2026-10-31

### Added

- redesign the today and plan pages ([#133](https://github.com/borrowcheckmate/crabbage/pull/133))
- redesign the shopping list ([#132](https://github.com/borrowcheckmate/crabbage/pull/132))
- finish administration, account context and system states ([#131](https://github.com/borrowcheckmate/crabbage/pull/131))
- build the shared visual foundation and responsive navigation ([#128](https://github.com/borrowcheckmate/crabbage/pull/128))
- take the recipe photo from the imported pages ([#126](https://github.com/borrowcheckmate/crabbage/pull/126))
- read the role and household from Pocket ID group claims ([#124](https://github.com/borrowcheckmate/crabbage/pull/124))
- add a healthcheck command to the image ([#123](https://github.com/borrowcheckmate/crabbage/pull/123))
- run the Mealie migration from the admin page ([#99](https://github.com/borrowcheckmate/crabbage/pull/99))
- turn the week's plan into a shopping list ([#94](https://github.com/borrowcheckmate/crabbage/pull/94))
- plan the week ([#93](https://github.com/borrowcheckmate/crabbage/pull/93))
- store weekly plans and household planner settings ([#92](https://github.com/borrowcheckmate/crabbage/pull/92))
- use the shopping list in the store ([#90](https://github.com/borrowcheckmate/crabbage/pull/90))
- sync the shopping list live over SSE ([#89](https://github.com/borrowcheckmate/crabbage/pull/89))
- store the household shopping list ([#88](https://github.com/borrowcheckmate/crabbage/pull/88))
- import recipes from photos ([#86](https://github.com/borrowcheckmate/crabbage/pull/86))
- review imported recipes before saving ([#85](https://github.com/borrowcheckmate/crabbage/pull/85))
- complete and translate imported recipes with AI ([#84](https://github.com/borrowcheckmate/crabbage/pull/84))
- keep the screen awake in cook mode ([#79](https://github.com/borrowcheckmate/crabbage/pull/79))
- start timers from cook mode steps ([#78](https://github.com/borrowcheckmate/crabbage/pull/78))
- show each step's ingredients, scaled, in cook mode ([#75](https://github.com/borrowcheckmate/crabbage/pull/75))
- cook mode shows a recipe one step at a time ([#74](https://github.com/borrowcheckmate/crabbage/pull/74))
- search recipes and scale servings on the recipe page ([#73](https://github.com/borrowcheckmate/crabbage/pull/73))
- rate and favourite recipes ([#72](https://github.com/borrowcheckmate/crabbage/pull/72))
- upload a photo for a recipe ([#69](https://github.com/borrowcheckmate/crabbage/pull/69))
- categorise recipes and filter the list by category ([#68](https://github.com/borrowcheckmate/crabbage/pull/68))
- ask before deleting a recipe ([#67](https://github.com/borrowcheckmate/crabbage/pull/67))
- show recipe lines in natural words for each language ([#65](https://github.com/borrowcheckmate/crabbage/pull/65))
- calm ingredient admin page with one row each and a detail page ([#64](https://github.com/borrowcheckmate/crabbage/pull/64))
- create, view, edit and delete recipes ([#60](https://github.com/borrowcheckmate/crabbage/pull/60))
- add a base stylesheet for a mobile-first layout ([#58](https://github.com/borrowcheckmate/crabbage/pull/58))
- manage the ingredient database ([#56](https://github.com/borrowcheckmate/crabbage/pull/56))
- publish a multi-arch Docker image ([#52](https://github.com/borrowcheckmate/crabbage/pull/52))
- translate the interface from one JSON file per language ([#51](https://github.com/borrowcheckmate/crabbage/pull/51))
- derive role and households from group claims ([#50](https://github.com/borrowcheckmate/crabbage/pull/50))
- log in with OIDC ([#49](https://github.com/borrowcheckmate/crabbage/pull/49))
- open the SQLite database and run migrations ([#48](https://github.com/borrowcheckmate/crabbage/pull/48))
- load configuration from the environment ([#47](https://github.com/borrowcheckmate/crabbage/pull/47))

### Fixed

- time out test response bodies that never finish ([#130](https://github.com/borrowcheckmate/crabbage/pull/130))
- time out test requests that get no answer ([#127](https://github.com/borrowcheckmate/crabbage/pull/127))
- store the user's display name from the ID token ([#63](https://github.com/borrowcheckmate/crabbage/pull/63))

### Other

- move the planner's SQL into crabbage-store ([#121](https://github.com/borrowcheckmate/crabbage/pull/121))
