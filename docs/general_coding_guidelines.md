## General Coding Guidelines

### Keep it simple
- Follow the YAGNI principle
- Follow the KISS principles
- Don't implement support for hypothetic use-cases

### Make breaking changes
- Breaking changes are encouraged to improve architecture
- No one outside of this repository depends on our code
- Do not maintain compatibility with unused APIs
- Remove deprecated code paths rather than preserving them

### Be upfront about placeholders and todos
- Distinguish clearly between working functionality and placeholders
- Never claim tasks are "complete" when they contain placeholders or todos

### Avoid evolutionary language
- Variable names are not meant to keep a changelog
- Code comments are not meant to keep a changelog
- Avoid words like: `evolved`, `improved`, `enhanced`, `upgraded`, `modernized`, `optimized`, `refactored`, `migrated`, `removed`, etc

### Avoid selling language
- Write for developers, not customers
- Focus on factual descriptions rather than qualitative assessments
- Acknowledge limitations and trade-offs
- Be humble and breif, but maintain technical precision
