Read the repository focusing on structure, build/run workflows, key components, and risks. Produce the deliverables below. If $ARGUMENTS is provided, prioritize that area.

# Goal
Think hardest and read the codebase, its architecture, risks, and recent changes to enable a productive discussion with the team.

# Instructions for Claude

## 1) Scope and inventory
- Identify primary languages, frameworks, build tools, and package managers.
- List top-level directories with brief purposes (src/, app/, packages/, services/, infra/, docs/, scripts/, etc.).
- Map entry points (CLI, server main, app bootstrap) and how the app starts locally and in CI.

## 2) Architecture overview
- Describe the high-level architecture: layers, modules, services, boundaries, and how data flows through the system.
- Identify key domain entities, data models/schemas, and core business workflows.
- Note integration points: databases, caches, queues, third-party APIs, feature flags, secrets, config.

## 3) Run and build
- Determine how to install dependencies, configure environment variables, and start the app locally.
- Summarize build steps, linting/formatting, type-checking, and test commands.
- Call out any Docker/Compose, dev containers, or Makefile targets.

## 4) Code quality and conventions
- Detect linters, formatters, type systems, and coding standards in use; summarize rules that matter.
- Surface test frameworks, structure, coverage setup, and any snapshot/mocking utilities.
- Note commit conventions, branching model, PR templates, and CODEOWNERS if present.

## 5) Observability and operations
- Outline logging, metrics, tracing, and feature toggles.
- Document configuration management, secrets handling, and environment-specific files.
- Summarize CI/CD pipeline stages, caches, parallelization, and longest path.

## 6) Risks, gaps, and questions
- List top risks (security, performance, reliability, maintainability) with rationale and suggested mitigations.
- Identify unclear or fragile areas (tight coupling, dead code, flaky tests, legacy patterns).
- Prepare concrete questions for the team to clarify assumptions and unblock work.

## 7) Recent changes and hotspots
- Scan recent commits/PRs to spot active areas of development and churn.
- Highlight files with high complexity or frequent modifications that merit attention.

## 8) Deliverables
A concise briefing with:
- System diagram (textual description)
- Runbook: install, configure, start, test
- Component/module map with responsibilities
- Risk register with mitigation ideas
- Open questions for discussion

Keep it actionable and prioritised. Prefer specifics over generalities.

# How to use $ARGUMENTS
If $ARGUMENTS is provided, use it to focus on a sub-area (e.g., "payments service", "frontend app", "auth module").

Examples:
- "payments service"
- "monorepo tooling"
- "API error handling"

# Output format
- Overview
- Architecture
- Run and Build
- Quality and Conventions
- Observability and Ops
- Risks and Questions
- Recent Changes
- Next Steps

# Notes
- Prefer reading README, package manifests, Dockerfiles, Makefiles, CI config, and entry points first for quick orientation.
- If information is missing, state assumptions explicitly and list where to confirm them.
- Think hardest
