---
name: dashboard-ui-components
description: Use when building new UI in packages/fern-dashboard/, adding interactive elements, or choosing between raw HTML and existing components. Covers shadcn/ui component catalog, import patterns, variants, and design tokens.
---

# Dashboard UI Components

## Overview

The Fern dashboard (`packages/fern-dashboard/`) uses **shadcn/ui (new-york style)** with Radix primitives, CVA variants, and Tailwind CSS v4. Always check `src/components/ui/` before building custom UI.

## Component Catalog

| Component | File | Export | Variants |
|-----------|------|--------|----------|
| **Button** | `button.tsx` | `Button`, `buttonVariants` | variant: `default` `destructive` `destructiveOutline` `outline` `secondary` `ghost` `link` `linkUnderlined` `dark` / size: `default` `xs` `sm` `lg` `icon` `iconSm` |
| **Badge** | `Badge.tsx` | `Badge` | variant: `info` `success` `warning` (required prop) |
| **StatusBadge** | `StatusBadge.tsx` | `StatusBadge` | status: `live` `unpublished` `loading` `uncommitted` `open` `closed` `merged` `draft` `preview` |
| **Card** | `card.tsx` | `default export` | No variants — simple wrapper |
| **Dialog** | `dialog.tsx` | `Dialog` `DialogTrigger` `DialogContent` `DialogHeader` `DialogTitle` `DialogDescription` `DialogBody` `DialogFooter` `DialogClose` `DialogOverlay` `DialogPortal` | `persistent` prop on DialogContent hides close button |
| **Input** | `input.tsx` | `Input` | No variants — uses `forwardRef` |
| **Select** | `select.tsx` | `Select` `SelectTrigger` `SelectValue` `SelectContent` `SelectItem` `SelectGroup` `SelectLabel` `SelectSeparator` `SelectScrollUpButton` `SelectScrollDownButton` | SelectTrigger size: `default` `sm` / SelectItem `description` prop / `checkOnLeft` on SelectContent |
| **Tooltip** | `tooltip.tsx` | `Tooltip` `TooltipProvider` | All-in-one wrapper (not separate primitives). Pass `content`, `side`, `sideOffset` |
| **DropdownMenu** | `dropdown-menu.tsx` | `DropdownMenu` + sub-parts | Standard Radix compound pattern |
| **Tabs** | `tabs.tsx` | `Tabs` `TabsList` `TabsTrigger` `TabsContent` | Standard compound pattern |
| **Table** | `table.tsx` | `Table` `TableHeader` `TableBody` `TableRow` `TableHead` `TableCell` | Standard compound pattern |
| **Label** | `label.tsx` | `Label` | — |
| **Switch** | `switch.tsx` | `Switch` | — |
| **RadioGroup** | `radio-group.tsx` | `RadioGroup` `RadioGroupItem` | — |
| **Textarea** | `textarea.tsx` | `Textarea` | — |
| **Skeleton** | `skeleton.tsx` | `Skeleton` | — |
| **Popover** | `popover.tsx` | `Popover` `PopoverTrigger` `PopoverContent` | — |
| **Pagination** | `pagination.tsx` | Pagination parts | — |
| **Sonner** | `sonner.tsx` | `Toaster` | Toast notifications |
| **Note** | `Note.tsx` | `Note` | Callout/admonition component |
| **Kbd** | `kbd.tsx` | `Kbd` | Keyboard shortcut display |
| **SearchableDropdown** | `SearchableDropdown.tsx` | `SearchableDropdown` | Generic `<T>` — searchable select |
| **CopyableText** | `CopyableText.tsx` | `CopyableText` | Click-to-copy |
| **Steps** | `steps.tsx` | `Steps` | Step indicator |

## Import Patterns

```typescript
// Always import cn from this path
import { cn } from "@/utils/utils";

// Named exports from individual files (no barrel index.ts)
import { Button } from "@/components/ui/button";
import {
    Dialog, DialogBody, DialogClose, DialogContent, DialogDescription,
    DialogFooter, DialogHeader, DialogTitle, DialogTrigger
} from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";

// Card is the ONLY default export
import Card from "@/components/ui/card";

// Icons from lucide-react
import { Plus, XIcon, ChevronDown } from "lucide-react";

// Type-only React import
import type * as React from "react";
```

## Component Patterns

### CVA Variants (Button pattern)
```typescript
import { cva, type VariantProps } from "class-variance-authority";

const myVariants = cva("base-classes", {
    variants: {
        variant: { default: "...", outline: "..." },
        size: { default: "h-9 px-4", sm: "h-8 px-3" }
    },
    defaultVariants: { variant: "default", size: "default" }
});

// Usage: cn(myVariants({ variant, size, className }))
```

### Compound Components (Dialog, Select, Tabs)
```tsx
<Dialog>
    <DialogTrigger asChild>
        <Button variant="outline">Open</Button>
    </DialogTrigger>
    <DialogContent>
        <DialogHeader>
            <DialogTitle>Title</DialogTitle>
            <DialogDescription>Description</DialogDescription>
        </DialogHeader>
        <DialogBody>
            {/* content */}
        </DialogBody>
        <DialogFooter>
            <DialogClose asChild><Button variant="outline">Cancel</Button></DialogClose>
            <Button variant="destructive">Delete</Button>
        </DialogFooter>
    </DialogContent>
</Dialog>
```

### Required Directives
- `"use client"` on all interactive components (button, dialog, select, tooltip, dropdown-menu, tabs, switch, popover)
- `data-slot="component-name"` on Radix-based wrappers

### asChild Pattern
Use `asChild` to render a different element while keeping component behavior:
```tsx
<DialogTrigger asChild>
    <Button variant="outline">Click me</Button>
</DialogTrigger>
```

### Loading State (Button)
```tsx
<Button loading>Saving...</Button>
// Renders children invisible + centered Loader2 spinner
// Automatically sets disabled={true}
```

## Design Tokens

### Semantic Colors (use these first)
| Token | Tailwind Class | Purpose |
|-------|---------------|---------|
| `--primary` | `bg-primary` `text-primary` | Fern green (`#008700` light, `#70e155` dark) |
| `--destructive` | `bg-destructive` `text-destructive` | Error/delete actions |
| `--muted-foreground` | `text-muted-foreground` | Secondary text |
| `--background` | `bg-background` | Page background |
| `--foreground` | `text-foreground` | Primary text |
| `--border` | `border-border` | Default borders |
| `--card` | `bg-card` | Card backgrounds |
| `--accent` | `bg-accent` | Hover/active states |
| `--sidebar` | `bg-sidebar` | Sidebar background |

### Color Scale (100-1200)
Available for: `gray`, `green`, `blue`, `purple`, `yellow`

```
bg-gray-100 … bg-gray-1200
text-green-1100  (brand green in light mode)
border-blue-900
```

- **100-400**: Backgrounds, surfaces
- **500-700**: Borders, separators
- **800-1000**: Icons, secondary elements
- **1100-1200**: Text, primary elements

### Dark Mode
Class-based: `.dark` on parent element. All tokens automatically remap. Never hardcode light/dark-specific hex values — use the semantic tokens or color scale.

## Decision Flow

```
Need UI element?
├── Check src/components/ui/ first
│   ├── Component exists → Use it with correct variant
│   └── Component doesn't exist
│       ├── Can shadcn generate it? → npx shadcn add <component>
│       └── Truly custom → Create in src/components/ui/ following patterns above
└── Styling only? → Use semantic tokens, then color scale, never arbitrary hex
```

## Common Mistakes

| Mistake | Fix |
|---------|-----|
| `import Card from "@/components/ui/Card"` | Card file is lowercase: `card.tsx` |
| `import { cn } from "@/lib/utils"` | Correct path: `@/utils/utils` |
| Using `forwardRef` everywhere | Only `Input` uses it; others are plain function components |
| Separate Tooltip primitives | Dashboard Tooltip is an all-in-one wrapper: `<Tooltip content="text">` |
| Missing `"use client"` | Required on all interactive components |
| Hardcoded hex colors | Use `--primary`, `--destructive`, or gray/green/blue/purple/yellow scale |
| `className={...}` without `cn()` | Always merge with `cn()` for Tailwind class deduplication |
