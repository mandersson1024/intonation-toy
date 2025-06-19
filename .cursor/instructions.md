# Cursor AI Instructions

## Date Handling

When creating documents that need dates:

1. **Always use the actual current date** - never guess or use placeholder dates
2. **To get today's date**: Use the terminal command `date` or ask the user for the current date
3. **Format**: Use format "Month YYYY" (e.g., "January 2024") for document headers
4. **Don't assume**: Never assume what month or year it is

## Document Standards

- All technical documents should have accurate timestamps
- Version dates should reflect when the document was actually created
- Update dates when making significant revisions

## Example Commands

```bash
# Get current date
date

# Get formatted date
date "+%B %Y"  # Returns "January 2024" format
```

If you need the current date for a document, either:
1. Run `date` command to get it
2. Ask the user: "What is today's date for the document header?" 