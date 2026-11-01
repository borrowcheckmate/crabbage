# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.0](https://github.com/borrowcheckmate/crabbage/compare/crabbage-store-v0.1.0...crabbage-store-v0.2.0) - 2026-11-01

### Added

- show friendly household names ([#187](https://github.com/borrowcheckmate/crabbage/pull/187))
- browser checks for layout, accessibility and interactions ([#156](https://github.com/borrowcheckmate/crabbage/pull/156))
- cargo xtask demo serves the app with demo data ([#148](https://github.com/borrowcheckmate/crabbage/pull/148))
- admin settings page for the AI provider, endpoint, model and key ([#145](https://github.com/borrowcheckmate/crabbage/pull/145))
- recipe list and recipe page match the redesign ([#142](https://github.com/borrowcheckmate/crabbage/pull/142))
- seed a default catalog of ingredients, sections and categories ([#144](https://github.com/borrowcheckmate/crabbage/pull/144))
- read the role and household from Pocket ID group claims ([#124](https://github.com/borrowcheckmate/crabbage/pull/124))
- add a healthcheck command to the image ([#123](https://github.com/borrowcheckmate/crabbage/pull/123))
- run the Mealie migration from the admin page ([#99](https://github.com/borrowcheckmate/crabbage/pull/99))
- turn the week's plan into a shopping list ([#94](https://github.com/borrowcheckmate/crabbage/pull/94))
- store weekly plans and household planner settings ([#92](https://github.com/borrowcheckmate/crabbage/pull/92))
- sync the shopping list live over SSE ([#89](https://github.com/borrowcheckmate/crabbage/pull/89))
- store the household shopping list ([#88](https://github.com/borrowcheckmate/crabbage/pull/88))
- review imported recipes before saving ([#85](https://github.com/borrowcheckmate/crabbage/pull/85))
- search recipes and scale servings on the recipe page ([#73](https://github.com/borrowcheckmate/crabbage/pull/73))
- rate and favourite recipes ([#72](https://github.com/borrowcheckmate/crabbage/pull/72))
- upload a photo for a recipe ([#69](https://github.com/borrowcheckmate/crabbage/pull/69))
- categorise recipes and filter the list by category ([#68](https://github.com/borrowcheckmate/crabbage/pull/68))
- create, view, edit and delete recipes ([#60](https://github.com/borrowcheckmate/crabbage/pull/60))
- manage the ingredient database ([#56](https://github.com/borrowcheckmate/crabbage/pull/56))
- publish a multi-arch Docker image ([#52](https://github.com/borrowcheckmate/crabbage/pull/52))
- derive role and households from group claims ([#50](https://github.com/borrowcheckmate/crabbage/pull/50))
- log in with OIDC ([#49](https://github.com/borrowcheckmate/crabbage/pull/49))
- open the SQLite database and run migrations ([#48](https://github.com/borrowcheckmate/crabbage/pull/48))
- load configuration from the environment ([#47](https://github.com/borrowcheckmate/crabbage/pull/47))

### Fixed

- take the write lock at the start of store transactions ([#71](https://github.com/borrowcheckmate/crabbage/pull/71))

### Other

- move the planner's SQL into crabbage-store ([#121](https://github.com/borrowcheckmate/crabbage/pull/121))
- remove an empty file left in the store crate ([#76](https://github.com/borrowcheckmate/crabbage/pull/76))
