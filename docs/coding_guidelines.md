## Coding Guidelines

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
- When coding in Rust, always add placeholder!(...) along with placeholder comments 
- When coding in Rust, always add todo!(...) along with todo comments 
- Never claim tasks are "complete" when they contain placeholders or todos

### Avoid evolutionary language
- Describe only the current state of the system, not the relation to what it used to be
- Write as if the current state has always existed
- Avoid words like: `evolved`, `improved`, `enhanced`, `upgraded`, `modernized`, `optimized`, `refactored`, `migrated`
- Apply this rule to all documentation, comments, and variable names
- Instead of "enhanced error handling", write "error handling handles X, Y, Z cases"
- Instead of "improved performance", write "processes N items per second"
- Instead of "newConfig", use "appConfig" or "systemConfig"

### Avoid selling language
- Write for developers, not customers
- Focus on factual descriptions rather than qualitative assessments
- Acknowledge limitations and trade-offs
- Be humble and breif, but maintain technical precision
