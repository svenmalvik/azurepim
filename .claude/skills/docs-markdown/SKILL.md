---
name: docs-markdown
description: Write or review markdown documentation following best practices. Use when creating docs, README files, or reviewing existing documentation for quality and consistency.
---

# Documentation Standards for Markdown Files

Apply these standards when writing or reviewing markdown documentation.

## General Principles
- Documentation is a first-class deliverable, not an afterthought
- Write for clarity, not cleverness
- Keep docs synchronized with code changes
- Use examples to illustrate concepts
- Assume readers have varying levels of technical expertise

## Structure and Organization

### File Naming
- Use lowercase with hyphens: `phase-3-setup.md`
- Be descriptive: prefer `stripe-integration-guide.md` over `guide.md`
- Group related docs in subdirectories when appropriate

### Document Structure
1. **Title**: Single H1 (`#`) at the top
2. **Overview**: Brief description of what this document covers
3. **Table of Contents**: For documents longer than 3 sections
4. **Main Content**: Logical sections with H2-H4 headings
5. **References/Links**: Related documentation at the end

## Formatting Standards

### Headings
- Use ATX-style headings (`#`, `##`, `###`)
- One H1 per document (the title)
- Don't skip heading levels (H1 -> H2 -> H3, not H1 -> H3)
- Add blank lines before and after headings

### Code Blocks
- Always specify language for syntax highlighting
- Use inline code (backticks) for commands, file names, variables
- Add comments to complex code examples
- Include expected output where helpful

### Lists
- Use `-` for unordered lists (not `*` or `+`)
- Use `1.` for ordered lists (auto-numbering)
- Indent nested lists with 2 spaces
- Add blank lines between list items if they contain multiple paragraphs

### Links
- Use descriptive link text: `[deployment guide](./deployment.md)`, not `[click here](./deployment.md)`
- Use relative paths for internal docs: `./setup.md` or `../api/endpoints.md`
- Verify links work (no broken references)

### Emphasis
- Use `**bold**` for important terms or UI elements
- Use `*italic*` for emphasis or introducing new concepts
- Use `code` for technical terms, file paths, commands

## Content Guidelines

### Code Examples
- Provide complete, runnable examples when possible
- Show both success and error cases where relevant
- Include setup/prerequisites if needed
- Explain what the code does, not just what it is

### Procedural Documentation
- Number sequential steps clearly
- Include expected outcomes: "You should see..."
- Mention common pitfalls or gotchas
- Provide troubleshooting tips

### Technical Accuracy
- Test all commands and code examples before documenting
- Keep version numbers up-to-date
- Note platform-specific differences (macOS vs Windows)
- Update docs when implementation changes

### Accessibility
- Use descriptive alt text for images
- Don't rely solely on color to convey information
- Ensure tables are simple and have headers

## Special Sections

### Prerequisites
List required knowledge, tools, or setup steps:
```markdown
## Prerequisites
- Node.js 18+ installed
- Supabase CLI configured
- Basic understanding of TypeScript
```

### Related Documentation
Always link to related docs at the end:
```markdown
## See Also
- [Phase 2 Setup](./phase-2-setup.md)
- [Troubleshooting Guide](./troubleshooting.md)
```

## Review Checklist
Before committing documentation:
- [ ] All code examples tested and working
- [ ] Links are valid (no 404s)
- [ ] Spelling and grammar checked
- [ ] Consistent formatting throughout
- [ ] Table of contents updated (if present)

## Anti-Patterns to Avoid

- Don't use vague language like "You might want to configure some settings"
- Don't assume too much knowledge without providing links
- Don't leave code examples incomplete
- Be specific: "Set the `timeout` value to 30 seconds" instead of "configure settings"
