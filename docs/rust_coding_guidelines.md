## Rust Coding Guidelines

### Be upfront about placeholders and todos
- Always add placeholder!(...) along with placeholder comments 
- Always add todo!(...) along with todo comments 

### Be clear about debug code
- For code that should only be available in debug builds, use `#[cfg(debug_assertions)]`
