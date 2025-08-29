# GroupWeave UI Component Guide

This document provides instructions for working with the shared UI component library in this monorepo. It is intended for developers who need to create, modify, or use UI components in any of the web applications.

## Overview

The GroupWeave frontend is built using a shared component library located in `packages/ui`. This library provides a set of reusable React components that are used across the various Next.js applications in the `apps/` directory (e.g., `apps/creation`, `apps/dashboard`).

Our approach is heavily inspired by [shadcn/ui](https://ui.shadcn.com/). While we do not use the `shadcn-ui` CLI directly, we use the same foundational libraries and principles. This means:
- Components are built using unstyled primitives from [Radix UI](https://www.radix-ui.com/).
- Styling is done with [Tailwind CSS](https://tailwindcss.com/).
- Component variations are managed with `class-variance-authority`.
- Class names are merged intelligently using `tailwind-merge`.

This approach keeps components self-contained and easy to customize while ensuring a consistent design language across all applications.

## Adding a New Component

To add a new component to the shared `packages/ui` library, follow these steps. We'll use a new `Badge` component as an example.

### 1. Create the Component File

Create a new file for your component inside the `packages/ui/src/ui/` directory.

**File:** `packages/ui/src/ui/badge.tsx`

```tsx
import * as React from "react"
import { cva, type VariantProps } from "class-variance-authority"

import { cn } from "@/lib/utils" // Ensure a 'cn' utility exists in packages/ui/src/lib/utils.ts (see note below)

const badgeVariants = cva(
  "inline-flex items-center rounded-full border px-2.5 py-0.5 text-xs font-semibold transition-colors focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2",
  {
    variants: {
      variant: {
        default:
          "border-transparent bg-primary text-primary-foreground hover:bg-primary/80",
        secondary:
          "border-transparent bg-secondary text-secondary-foreground hover:bg-secondary/80",
        destructive:
          "border-transparent bg-destructive text-destructive-foreground hover:bg-destructive/80",
        outline: "text-foreground",
      },
    },
    defaultVariants: {
      variant: "default",
    },
  }
)

export interface BadgeProps
  extends React.HTMLAttributes<HTMLDivElement>,
    VariantProps<typeof badgeVariants> {}

function Badge({ className, variant, ...props }: BadgeProps) {
  return (
    <div className={cn(badgeVariants({ variant }), className)} {...props} />
  )
}

export { Badge, badgeVariants }
```
*(Note: You would need to ensure a `cn` utility exists in `packages/ui/src/lib/utils.ts`, which is standard in this setup.)*

### 2. Export the Component

For components to be easily imported by applications, you need to export them from the `packages/ui` library.

**a) Main Entry Point:**

Add an export for your new component in `packages/ui/src/index.ts`.

```ts
// packages/ui/src/index.ts
export * from "./ui/button";
export * from "./ui/card";
// ... other component exports
export * from "./ui/badge"; // Add this line
```

**b) Direct Export for Tree-Shaking:**

To allow applications to import components directly, improving tree-shaking, add a new export path to `packages/ui/package.json`.

```json
// packages/ui/package.json
{
  "name": "@repo/ui",
  "version": "0.0.0",
  "exports": {
    ".": "./dist/index.js",
    "./button": "./dist/button.js",
    // ... other component exports
    "./badge": "./dist/ui/badge.js" // Add this line
  },
  //...
}
```

### 3. Rebuild the UI Package

After adding a new component and updating `package.json`, you need to rebuild the `ui` package for the changes to be available to other workspaces.

```bash
pnpm build --filter @repo/ui
```

## Using a Component in an App

Once the component is added to the `packages/ui` library and the package has been rebuilt, you can use it in any of the web applications.

For example, to use the new `Badge` component in `apps/creation`:

```tsx
// In any component within apps/creation, e.g., apps/creation/app/page.tsx

import { Badge } from "@repo/ui/badge"; // Direct import for better bundling
import { Button } from "@repo/ui/button";

export default function Page() {
  return (
    <div>
      <h1>My Awesome Creation</h1>
      <Badge variant="secondary">New!</Badge>
      <Button>Get Started</Button>
    </div>
  );
}
```

## Important Considerations

### React Native (`apps/mobile`)

The components in `packages/ui` are built for the web using React DOM and standard HTML elements. **They are not directly compatible with React Native.** The `apps/mobile` application requires its own set of components built specifically for the native environment (e.g., using primitives from `react-native`).

### Backend and Smart Contract Integration

The UI components are purely for presentation. They are agnostic to where the data comes from. Whether you are fetching data from the `apps/api` backend or interacting with smart contracts via `packages/near`, the UI components simply receive props and render accordingly. This separation of concerns is a core principle of the architecture.
