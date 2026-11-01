# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.0](https://github.com/borrowcheckmate/crabbage/compare/crabbage-import-v0.1.0...crabbage-import-v0.2.0) - 2026-11-01

### Added

- let the AI clean up every import and pick its categories ([#173](https://github.com/borrowcheckmate/crabbage/pull/173))
- browser checks for layout, accessibility and interactions ([#156](https://github.com/borrowcheckmate/crabbage/pull/156))
- cargo xtask demo serves the app with demo data ([#148](https://github.com/borrowcheckmate/crabbage/pull/148))
- admin settings page for the AI provider, endpoint, model and key ([#145](https://github.com/borrowcheckmate/crabbage/pull/145))
- seed a default catalog of ingredients, sections and categories ([#144](https://github.com/borrowcheckmate/crabbage/pull/144))
- take the recipe photo from the imported pages ([#126](https://github.com/borrowcheckmate/crabbage/pull/126))
- read the role and household from Pocket ID group claims ([#124](https://github.com/borrowcheckmate/crabbage/pull/124))
- add a healthcheck command to the image ([#123](https://github.com/borrowcheckmate/crabbage/pull/123))
- map Mealie data to crabbage ([#97](https://github.com/borrowcheckmate/crabbage/pull/97))
- read data from the Mealie API ([#96](https://github.com/borrowcheckmate/crabbage/pull/96))
- import recipes from photos ([#86](https://github.com/borrowcheckmate/crabbage/pull/86))
- review imported recipes before saving ([#85](https://github.com/borrowcheckmate/crabbage/pull/85))
- complete and translate imported recipes with AI ([#84](https://github.com/borrowcheckmate/crabbage/pull/84))
- call an OpenAI-compatible endpoint for recipe drafts ([#83](https://github.com/borrowcheckmate/crabbage/pull/83))
- extract recipes from web pages ([#82](https://github.com/borrowcheckmate/crabbage/pull/82))
- match ingredient names to the catalogue by name and alias ([#81](https://github.com/borrowcheckmate/crabbage/pull/81))
- parse ingredient lines into amount, unit, name and note ([#80](https://github.com/borrowcheckmate/crabbage/pull/80))
- publish a multi-arch Docker image ([#52](https://github.com/borrowcheckmate/crabbage/pull/52))
- log in with OIDC ([#49](https://github.com/borrowcheckmate/crabbage/pull/49))
- open the SQLite database and run migrations ([#48](https://github.com/borrowcheckmate/crabbage/pull/48))
- load configuration from the environment ([#47](https://github.com/borrowcheckmate/crabbage/pull/47))

### Fixed

- keep AI lines with an unknown unit ([#175](https://github.com/borrowcheckmate/crabbage/pull/175))
