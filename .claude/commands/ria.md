Analyze the codebase to create a comprehensive Security and Privacy Risk Identification Assessment (RIA) document. Focus on $ARGUMENTS if provided, otherwise analyze the entire repository.

# Goal
Generate a thorough RIA document following Vipps MobilePay standards by analyzing the codebase architecture, data flows, dependencies, and potential security risks to produce actionable security controls and risk scenarios.

# Instructions for Claude

## 1) Repository Analysis and Service Description
- Identify the service purpose, architecture, and key components
- Map data flows, APIs, databases, and external integrations
- Catalog dependencies, frameworks, and third-party services
- Determine deployment model (cloud, on-premises, hybrid)
- Identify the service owner and classify the overall data sensitivity

## 2) Data Discovery and Classification
Create comprehensive data dictionary by:
- Scanning configuration files, schemas, and documentation for data types
- Identifying PII, financial data, authentication tokens, business data
- Analyzing API endpoints and database models for data structures
- Reviewing logs, monitoring, and analytics for data exposure
- Classifying each data type according to sensitivity levels:
  - `PUBLIC`: No access restrictions
  - `INTERNAL`: Vipps MobilePay employees only
  - `CONFIDENTIAL`: Restricted access, business sensitive
  - `RESTRICTED`: Highly sensitive, minimal access

## 3) Security Architecture Assessment
- Map authentication and authorization mechanisms
- Identify encryption at rest and in transit
- Analyze network security controls and segmentation
- Review secrets management and configuration security
- Assess logging, monitoring, and incident response capabilities

## 4) Risk Scenario Development
Generate realistic risk scenarios by analyzing:
- **Authentication/Authorization risks**: Weak access controls, privilege escalation
- **Data exposure risks**: Unencrypted data, logging sensitive information
- **Infrastructure risks**: Misconfigured services, vulnerable dependencies
- **Application risks**: Input validation, business logic flaws
- **Operational risks**: Deployment issues, configuration drift
- **Compliance risks**: GDPR, PCI-DSS, or other regulatory requirements

## 5) Risk Impact Analysis
For each risk scenario, define:
- **Risk Driver**: Root cause and vulnerability details
- **Impact**: Specific business and technical consequences
- **Security Controls**: Preventive and detective measures

## 6) Service Metadata Collection
Extract or determine:
- Service owner (team responsible)
- Risk owner (individual accountable)
- Data classification (highest sensitivity level)
- Last reviewed date (current date)

# How to use $ARGUMENTS
If $ARGUMENTS is provided, focus the analysis on specific areas:
- Component names (e.g., "authentication service", "payment API")
- Data types (e.g., "user data", "financial transactions")
- Technologies (e.g., "Redis cache", "PostgreSQL database")

Examples:
- "payment processing module"
- "user authentication system"  
- "data analytics pipeline"
- "API gateway"

# Output Format
Create a complete `ria.md` file with:

```markdown
# Security and Privacy Risk Identification

|                                 |                  |
|---|---|
| **Service Owner(s)**            | [Team Name] |
| **Risk Owner**                  | [Individual Name] |
| **Service Data Classification** | [Classification Level] |
| **Last Reviewed**               | [Current Date] |

## Service description
[Comprehensive service description including purpose, architecture, data flows, and access controls]

## Data dictionary
| Data                        | Classification | Comments                                                                                                    |
|---|---|---|
| [Data Type 1]               | [Classification] | [Detailed description of data, usage, and sensitivity rationale] |
| [Data Type 2]               | [Classification] | [Detailed description of data, usage, and sensitivity rationale] |

## Risk scenarios / Impact Analysis
| Risk scenario                          | Risk Driver                                                                                                                                                                                                 | Impact - What may happen if we don't implement the controls?                                                                                          | Security Controls                                                                 |
|---|---|---|---|
| [Risk Scenario 1] | [Root cause and vulnerability details] | [Specific business and technical consequences] | [Preventive and detective security measures] |
| [Risk Scenario 2] | [Root cause and vulnerability details] | [Specific business and technical consequences] | [Preventive and detective security measures] |

[//]: # (Do not delete; used for stats: [Generated UUID])
```

# Quality Standards
Ensure the RIA meets these criteria:
- **Completeness**: Cover all major data types and risk scenarios
- **Specificity**: Avoid generic risks, focus on actual codebase vulnerabilities
- **Actionability**: Security controls must be implementable
- **Business Context**: Connect technical risks to business impact
- **Compliance**: Consider relevant regulatory requirements (GDPR, PCI-DSS)

# Notes
- Save the output as `ria.md` in the project root directory
- Use the Write tool to create the file
- If uncertain about service owner or risk owner, use placeholders with TODO comments
- Focus on realistic, implementable security controls rather than theoretical best practices
- Include a generated UUID at the bottom for tracking purposes
- Analyze the existing codebase thoroughly before generating content
- Consider both current architecture and potential future risks based on the technology stack