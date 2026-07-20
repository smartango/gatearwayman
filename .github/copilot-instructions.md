# Project Guidelines

## Overview

API manager backend service built in Rust. It serves gateway management APIs and static GUI assets embedded at build time.

## Architecture

- src/main.rs: application entry point and router bootstrap.
- src/lib.rs: shared crate-level logic.
- src/static_routes.rs: static asset route wiring.
- build.rs: compile-time asset route generation helpers.
- Dockerfile: multi-stage build producing a scratch runtime image.
- scripts/extract-img-tag.sh: computes CI image tags from git tags and commit hash.

## Stack

- Rust 2021 edition
- Axum 0.8
- Tokio 1.x
- Reqwest 0.12
- PHF for static lookups

## Build And Run

- Local build: cargo build
- Local run: cargo run
- Tests: cargo test
- Release with assets: ASSETS=/assets cargo build --release

## Docker Build Rules

- Keep the Docker image as multi-stage with rust build stage and scratch final stage.
- GUI assets are downloaded from a private repository during Docker build.
- Always use BuildKit secret mounts for tokens; never use ARG or ENV for secrets.
- Keep the download in a single RUN step using mount type secret id github_token.
- Do not hardcode personal access tokens or credentials in source files, workflow files, or logs.

## Private GUI Repository Policy

- The GUI tarball source is a private repository under the same owner account.
- Any CI change touching GUI download must preserve authenticated access behavior.
- If download fails with 404 or unauthorized, treat it as a token scope/access issue first.
- Prefer failing early with clear curl errors before extraction steps.

## GitHub Actions Conventions

- Docker publish workflow file is .github/workflows/docker-image.yml.
- Keep explicit GHCR login before push steps.
- Keep job permissions minimal and explicit, including packages write for image push.
- For pull requests, preserve current push gating behavior.

## API And Response Conventions

- Keep endpoint documentation updated in README.md when routes or payloads change.
- Prefer consistent envelope responses with error and data fields.
- Avoid introducing authentication logic in this service when gateway already injects identity context.

## Change Strategy

- Prefer the smallest safe patch that solves the issue.
- Avoid unrelated refactors when fixing CI or build failures.
- When editing workflow or Docker logic, verify both file syntax and runtime assumptions.
