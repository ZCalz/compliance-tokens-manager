# Project: [Your Project Name]

Brief description of what this project does and its purpose.

## Technology Stack
- **Language**: TypeScript / JavaScript / Python / etc.
- **Framework**: Express / React / FastAPI / etc.
- **Database**: PostgreSQL / SQLite / etc.
- **Testing**: Jest / pytest / etc.

## Build & Test Commands
- **Install**: `npm install`
- **Build**: `npm run build`
- **Dev**: `npm run dev`
- **Test**: `npm test`
- **Lint**: `npm run lint`

## Project Structure
```
src/
  ├── api/          # Route handlers
  ├── services/     # Business logic
  ├── utils/        # Shared utilities
  └── index.ts      # Entry point
tests/
  ├── unit/
  └── integration/
```

## Coding Standards
- 2-space indentation, single quotes, semicolons required
- camelCase for variables/functions, PascalCase for classes/types
- Tests required for all new features (80%+ coverage target)
- No secrets in code — use environment variables

## Key Workflows

### Feature Development
1. Create branch from `main`
2. Implement with tests
3. Run `npm test && npm run lint`
4. Open PR referencing the issue

### Bug Fixes
- Reference the issue number in commit message
- Add regression tests to prevent recurrence

## Common Gotchas
- [Add project-specific gotchas here]
- [E.g. "Database migrations must be idempotent"]
- [E.g. "API responses use standard error format — see src/utils/errors.ts"]

## Additional Context
@docs/architecture.md
@package.json
