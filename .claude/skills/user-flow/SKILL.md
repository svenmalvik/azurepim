---
name: user-flow
description: Analyze all user flows in the application to identify potential pitfalls, broken flows, race conditions, error handling gaps, and UX issues. Use when reviewing the application holistically or before releases.
---

# User Flow Analysis

Perform a comprehensive analysis of all user flows in the application, identifying potential pitfalls, broken paths, edge cases, and areas that might not work as expected.

# Goal

Systematically trace through every user journey in the application to discover:
- Broken or incomplete flows
- Race conditions and threading issues
- Missing error handling
- Edge cases that could cause failures
- Poor UX in error scenarios
- Security vulnerabilities in flows
- State management issues

# Instructions for Claude

## Phase 1: Flow Discovery

First, explore the codebase to map all user-facing flows:

### 1.1 Entry Points
Identify all ways a user can interact with the application:
- Menu bar actions (clicks, hover states)
- System events (app launch, wake from sleep, network changes)
- URL scheme handlers (callbacks, deep links)
- Keyboard shortcuts
- Background operations (token refresh, polling)

### 1.2 State Transitions
Map all application states and transitions:
- Authentication states (logged out → authenticating → logged in → token expired)
- UI states (loading, success, error, disabled)
- Background process states

### 1.3 Data Flows
Trace how data moves through the system:
- User input → validation → processing → storage
- External API calls → response handling → UI updates
- Credential storage and retrieval

## Phase 2: Flow Analysis

For each identified flow, analyze these aspects:

### 2.1 Happy Path
- Does the flow complete successfully under normal conditions?
- Is the feedback to the user clear and timely?
- Are all necessary state updates performed?

### 2.2 Error Handling
- What happens when external services fail (network, API, etc.)?
- Are errors communicated clearly to the user?
- Can the user recover from errors?
- Are errors logged appropriately for debugging?

### 2.3 Edge Cases
- What happens with invalid or unexpected input?
- How does the flow handle timeout scenarios?
- What if the user interrupts the flow (closes app, clicks elsewhere)?
- What happens during concurrent operations?

### 2.4 Threading & Concurrency
For Rust/macOS applications specifically:
- Are AppKit operations on the main thread?
- Are async operations properly dispatched?
- Are there potential race conditions?
- Is state accessed safely across threads?

### 2.5 State Consistency
- Is the UI always in sync with the underlying state?
- What happens if state changes mid-flow?
- Are there orphaned states the user can't escape?

### 2.6 Security
- Is sensitive data protected throughout the flow?
- Are there CSRF/replay attack vectors?
- Is input validated before use?

## Phase 3: Common Pitfall Patterns

Check specifically for these known issues:

### Authentication Flows
- [ ] Token refresh during active operation
- [ ] Multiple simultaneous auth attempts
- [ ] Browser callback never received
- [ ] State parameter mismatch (CSRF)
- [ ] Expired token not detected
- [ ] Keychain access failures
- [ ] Race between token refresh and API calls

### UI/Menu Flows
- [ ] Menu items enabled when they shouldn't be
- [ ] Stale menu state after operation completes
- [ ] Missing loading indicators
- [ ] Click handlers that block main thread
- [ ] Menu updates from background threads (crash risk)

### Network Flows
- [ ] No timeout on HTTP requests
- [ ] Retry logic missing or broken
- [ ] No handling for network unavailable
- [ ] SSL certificate validation bypassed
- [ ] API rate limiting not handled

### State Management
- [ ] Global state accessed without synchronization
- [ ] UI not updated after state change
- [ ] State persisted with sensitive data
- [ ] Stale state after app relaunch

### macOS-Specific (objc2)
- [ ] Retained objects released too early
- [ ] Missing MainThreadMarker for AppKit calls
- [ ] Objective-C exceptions not caught
- [ ] URL scheme handler timing issues

## Phase 4: Report Generation

After analysis, produce a structured report:

```markdown
# User Flow Analysis Report

## Executive Summary
- Total flows analyzed: N
- Critical issues found: N
- Warnings: N
- Recommendations: N

## Flows Analyzed

### Flow 1: [Name]
**Path**: [Step 1] → [Step 2] → [Step 3]
**Status**: OK / Warning / Critical

#### Issues Found
1. **[Severity]** - [Description]
   - Location: `file.rs:line`
   - Impact: [What could go wrong]
   - Recommendation: [How to fix]

#### Edge Cases Not Handled
- [Case 1]
- [Case 2]

### Flow 2: [Name]
...

## Critical Issues (Must Fix)
1. [Issue summary with locations]

## Warnings (Should Fix)
1. [Issue summary with locations]

## Recommendations (Nice to Have)
1. [Suggestion for improvement]

## Test Scenarios to Add
Based on the analysis, these scenarios should be tested:
1. [Scenario 1]
2. [Scenario 2]
```

## Analysis Guidelines

1. **Read before assuming**: Always read the actual code to understand what happens, don't assume based on function names.

2. **Follow the thread**: When analyzing async code, trace the execution path through all callbacks and dispatches.

3. **Check error paths**: For every `Result` or `Option`, verify what happens on error/None.

4. **Consider timing**: Think about what happens if operations complete in unexpected orders.

5. **Test mentally**: Walk through the flow as a user would experience it.

6. **Be specific**: Reference exact file paths and line numbers for issues found.

7. **Prioritize by impact**: Focus on issues that affect security, data loss, or crashes first.

## Output Format

Use the Read, Grep, and Glob tools to explore the codebase. Present findings in a clear, actionable format that developers can use to fix issues.

After the analysis, ask the user if they want to:
1. Deep dive into any specific flow
2. Generate test cases for problem areas
3. Create a prioritized fix list
4. Analyze a specific component in more detail
