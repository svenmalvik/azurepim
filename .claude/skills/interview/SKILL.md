---
name: interview
description: This skill should be used when the user asks to be "interviewed", runs "/interview", says "interview me about...", or requests a structured discovery process for requirements gathering. Conducts an in-depth technical interview and generates a comprehensive specification document.
---

# Interview Command

Conduct an in-depth technical interview with the user about a feature, project, or system they want to build. Use the AskUserQuestion tool to systematically gather requirements, understand tradeoffs, and explore technical decisions. Continue until you have complete understanding, then write a comprehensive specification.

# Goal

Through iterative questioning, extract a complete picture of:
- Technical implementation details and architecture
- UI/UX requirements and user flows
- Edge cases, constraints, and concerns
- Performance, scalability, and security considerations
- Tradeoffs between different approaches
- Dependencies, integrations, and infrastructure needs

# Instructions for Claude

## Interview Process

You MUST use the AskUserQuestion tool throughout this process. Never make assumptions - always ask.

### Phase 1: Problem Space (2-4 rounds)

**Round 1 - Core Problem**
- What specific problem or need does this solve?
- Who are the users and what are their pain points?
- What currently exists (if anything) and why is it insufficient?
- What does success look like?

**Round 2 - Scope and Constraints**
- What are the hard constraints (budget, timeline, tech stack, compliance)?
- What's in scope vs out of scope for the initial version?
- What are the dependencies on other systems or teams?
- What are the non-negotiable requirements?

**Round 3 - User Experience**
- How should users interact with this (UI, API, CLI, etc.)?
- What's the primary user journey from start to finish?
- What should happen in error/edge cases from a user perspective?
- What are the accessibility and internationalization requirements?

**Round 4 - Scale and Performance**
- What are the expected usage patterns (users, requests, data volume)?
- What are the performance requirements (latency, throughput)?
- How should the system handle load spikes or failures?
- What are the data retention and backup requirements?

### Phase 2: Solution Space (2-4 rounds)

**Round 5 - Technical Approach**
- What's the high-level architecture (monolith, microservices, serverless)?
- What are the key components and how do they communicate?
- What tech stack is preferred or required (languages, frameworks)?
- What hosting/infrastructure is planned (cloud provider, on-prem)?

**Round 6 - Data and State**
- What data needs to be stored and for how long?
- What's the data model and relationships?
- What database technology fits best (SQL, NoSQL, graph, etc.)?
- How should data be validated, transformed, and migrated?

**Round 7 - Integration Points**
- What external systems need to be integrated?
- What APIs or SDKs need to be consumed or provided?
- What authentication/authorization mechanism is needed?
- What third-party services are required (payment, email, etc.)?

**Round 8 - Operations and Security**
- How will the system be monitored and debugged?
- What logging, metrics, and alerting are needed?
- What are the security requirements (encryption, compliance, audit)?
- How will deployments and rollbacks work?

### Phase 3: Tradeoffs and Decisions (2-3 rounds)

**Round 9 - Architecture Tradeoffs**
Ask about specific tradeoffs relevant to their answers:
- Consistency vs availability vs partition tolerance
- Complexity vs time-to-market
- Cost vs performance
- Build vs buy for key components
- Flexibility vs simplicity

**Round 10 - Risk Assessment**
- What are the biggest technical risks?
- What could go wrong at scale?
- What happens if a dependency fails?
- What's the disaster recovery plan?

**Round 11 - Future Considerations**
- What features are planned for future phases?
- How might requirements change in 6-12 months?
- What should be designed for extensibility?
- What technical debt is acceptable in v1?

## Interview Guidelines

1. **Ask non-obvious questions**: Don't ask basic questions like "what color should the button be?" Ask about implications, edge cases, and hidden complexity.

2. **Build on previous answers**: Reference their earlier responses to go deeper. For example: "You mentioned using microservices - how will you handle distributed transactions?"

3. **Explore tradeoffs**: Present 2-4 specific options with pros/cons and ask them to choose and explain why.

4. **Challenge assumptions**: If they suggest something that might have issues, ask probing questions to ensure they've thought it through.

5. **Use the AskUserQuestion tool properly**:
   - Ask 1-4 related questions per round
   - Provide clear, concise options when appropriate
   - Use multiSelect when multiple answers make sense
   - Make headers short (e.g., "Auth method", "Database", "Approach")

6. **Iterate until complete**: Don't rush. If answers are vague, ask follow-up questions. If you discover new areas to explore, add rounds.

7. **Adapt to the project**: Adjust phases and questions based on what they're building. A CLI tool needs different questions than a web app.

## After Interview: Write Specification

Once you have thorough answers (typically 10-15 rounds), synthesize everything into a comprehensive specification document.

### Specification Structure

```markdown
# [Project Name] - Technical Specification

## 1. Executive Summary
- Problem statement
- Proposed solution (1-2 paragraphs)
- Key success metrics

## 2. Requirements

### 2.1 Functional Requirements
- Core features and capabilities
- User stories or use cases
- Acceptance criteria

### 2.2 Non-Functional Requirements
- Performance targets (latency, throughput, etc.)
- Scalability requirements
- Security and compliance requirements
- Availability and reliability targets

### 2.3 Constraints
- Technical constraints
- Business constraints
- Timeline and resource constraints

## 3. User Experience

### 3.1 User Personas
- Primary and secondary users
- User goals and pain points

### 3.2 User Flows
- Primary user journeys (step-by-step)
- Error handling flows
- Edge case scenarios

### 3.3 Interface Design
- UI/UX principles and guidelines
- Key screens or interactions (if applicable)
- API design (if applicable)

## 4. Technical Architecture

### 4.1 System Overview
- High-level architecture diagram (textual description)
- Component breakdown
- Communication patterns

### 4.2 Technology Stack
- Languages and frameworks
- Databases and storage
- Infrastructure and hosting
- Third-party services

### 4.3 Data Model
- Core entities and relationships
- Database schema design
- Data flow and transformations

### 4.4 API Design
- Endpoint specifications
- Request/response formats
- Authentication and authorization

## 5. Integration Points

### 5.1 External Systems
- Third-party APIs and services
- Internal system dependencies
- Webhooks and callbacks

### 5.2 Authentication & Authorization
- Auth mechanism (OAuth, JWT, etc.)
- Permission model
- Session management

## 6. Operations

### 6.1 Deployment Strategy
- Deployment pipeline
- Environment configuration
- Rollback procedures

### 6.2 Monitoring & Observability
- Logging strategy
- Metrics and dashboards
- Alerting rules

### 6.3 Security
- Threat model
- Security controls
- Compliance requirements
- Data privacy measures

## 7. Implementation Plan

### 7.1 Phases
- MVP/Phase 1 features
- Future phases

### 7.2 Dependencies
- External dependencies
- Prerequisite work

### 7.3 Risks & Mitigations
- Technical risks and mitigation strategies
- Open questions and assumptions

## 8. Tradeoffs & Decisions

### 8.1 Architecture Decisions
- Key decisions made and rationale
- Alternatives considered

### 8.2 Technical Debt
- Acceptable shortcuts in v1
- Plans to address debt later

## 9. Success Criteria

### 9.1 Launch Criteria
- Required functionality
- Performance benchmarks
- Security checklist

### 9.2 Metrics
- KPIs to track
- Success thresholds

## 10. Open Questions

- Remaining unknowns
- Items requiring further discussion
```

### Specification Guidelines

1. **Be specific**: Include concrete numbers, technologies, and decisions
2. **Reference the interview**: Quote or reference their answers where relevant
3. **Flag uncertainties**: If something is still unclear, call it out explicitly
4. **Include diagrams**: Describe architecture, data flow, and user flows in text
5. **Make it actionable**: The spec should be detailed enough to start implementation
6. **Highlight risks**: Be honest about challenges and unknowns

## Example Interview Round

```
<AskUserQuestion>
{
  "questions": [
    {
      "question": "How should the system handle authentication and user sessions?",
      "header": "Auth method",
      "multiSelect": false,
      "options": [
        {
          "label": "JWT tokens with refresh mechanism",
          "description": "Stateless auth, good for distributed systems, requires token refresh logic"
        },
        {
          "label": "Session-based with Redis",
          "description": "Server-side state, easier revocation, requires session store"
        },
        {
          "label": "OAuth 2.0 / OIDC with external provider",
          "description": "Delegate to Google/Auth0, requires third-party dependency"
        },
        {
          "label": "API keys for service accounts",
          "description": "Simple for machine-to-machine, not suitable for end users"
        }
      ]
    },
    {
      "question": "What should happen when a user's session expires while they're working?",
      "header": "Session expiry",
      "multiSelect": false,
      "options": [
        {
          "label": "Silent refresh in background",
          "description": "Best UX but more complex, requires refresh token mechanism"
        },
        {
          "label": "Show modal and require re-login",
          "description": "Simple but disruptive, may lose user work"
        },
        {
          "label": "Save work and redirect to login",
          "description": "Preserve state, requires draft/autosave functionality"
        }
      ]
    }
  ]
}
</AskUserQuestion>
```

## When to Stop

Continue interviewing until:
1. You understand the complete user journey
2. You know the technical architecture and all major components
3. You've explored key tradeoffs and decisions
4. You understand edge cases and error scenarios
5. You have clarity on scope, constraints, and timeline
6. The user has no more important considerations to add

Typically this takes 10-15 rounds of questions (30-60 total questions).

## Notes

- **Always use AskUserQuestion tool**: This is not optional. The entire interview MUST use this tool.
- **Don't rush**: Depth is more important than speed
- **Stay curious**: If something seems simple, probe deeper
- **Be thorough**: Better to over-ask than under-specify
- **Think like a staff engineer**: Consider scalability, security, maintainability, operations
- **Customize per project**: A mobile app needs different questions than a batch processing system

After completing the interview and writing the spec, ask the user if they'd like to refine any section or proceed with implementation planning.
