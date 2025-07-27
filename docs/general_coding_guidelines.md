## General Coding Guidelines

### Keep it simple
- Don't implement support for speculative future use-cases
- Follow the YAGNI principle
- Follow the KISS principles

### Breaking changes are allowed
- Don't maintain API compatibility when refactoring
- This is not a library that will be used by external code
- Remove deprecated code paths rather than preserving them

### Be upfront about placeholders and todos
- Distinguish clearly between working functionality and placeholders
- Never claim tasks are "complete" when they contain placeholders or todos

### Avoid evolutionary language
- Variable names are not meant to keep a changelog
- Code comments are not meant to keep a changelog
- Avoid words like: `evolved`, `improved`, `enhanced`, `upgraded`, `modernized`, `optimized`, `refactored`, `migrated`, `removed`, `simplified`, etc

### Avoid selling language
- Write for developers, not customers
- Focus on factual descriptions rather than qualitative assessments
- Acknowledge limitations and trade-offs
- Be humble and breif, but maintain technical precision
