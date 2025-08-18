# GroupWeave - Claude Code Instructions

## Repository Context
- This is a user-owned AI platform with web, mobile, and blockchain components.
- Follows strict TypeScript practices with a zero-warnings policy.
- Uses a shared component library (`@repo/ui`), which may use Radix UI internally for consistency.

## Development Preferences
- Always run `pnpm lint` and `pnpm check-types` before committing.
- Follow existing component patterns in `@repo/ui`.
- Test mobile changes across platforms when possible.
- Coordinate changes across workspace dependencies.

## Key Files
- `/turbo.json` - Build pipeline configuration
- `/apps/web/components/` - Main UI components
- `/packages/ui/` - Shared component library
