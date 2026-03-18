---
name: update-with-dashboard-ui
description: Use when refactoring dashboard code that has raw HTML elements, hardcoded colors, arbitrary Tailwind values, or inconsistent patterns that should use the existing UI component library and design tokens.
---

# Update Code with Dashboard UI Components

## Overview

Audit dashboard code for design system compliance, then replace raw HTML/Tailwind with existing UI components and design tokens. Audit first, fix second.

**REQUIRED BACKGROUND:** Load `dashboard-ui-components` skill for the component catalog and token reference.

## When to Use

- Refactoring existing dashboard pages/features
- Code review finds raw `<button>`, `<input>`, `<div>` where UI components exist
- Hardcoded hex colors or arbitrary Tailwind values in dashboard code
- Inconsistent component usage across files

## Four-Phase Audit

**Announce at start:** "Auditing this code for design system compliance using the update-with-dashboard-ui skill."

### Phase 1: Component Audit

Scan for raw HTML that maps to existing UI components:

| Raw HTML Pattern | Replace With |
|-----------------|-------------|
| `<button className="...">` | `<Button variant="...">` |
| `<input className="..." />` | `<Input />` |
| `<div className="rounded border p-4">` (card-like) | `<Card>` |
| `<span className="rounded-full px-2 text-xs">` (badge-like) | `<Badge variant="...">` |
| `<div>` with modal/overlay behavior | `<Dialog>` compound |
| `<select>` or custom dropdown | `<Select>` compound |
| `<div>` with tab switching | `<Tabs>` compound |
| `<table>` elements | `<Table>` compound |
| `<label>` | `<Label>` |
| `<textarea>` | `<Textarea>` |
| Tooltip via title attr or custom | `<Tooltip content="...">` |

**Skip:** Layout `<div>`s, one-off styled wrappers, elements with behavior not covered by the library.

**Important:** When replacing raw HTML with a UI component, **drop all styling classes** that the component handles internally (padding, border-radius, background, hover states, font-size). UI components own their own styling. Only pass `className` for layout concerns (margin, width) that the component doesn't control.

### Phase 2: Color Audit

Flag values NOT in the design system:

```
# BAD — hardcoded hex
className="text-[#008700]"
className="bg-[#ff0000]"
style={{ color: "#333" }}

# BAD — arbitrary Tailwind colors
className="text-red-500"  (standard Tailwind, not in Fern scale)

# GOOD — semantic tokens
className="text-primary"
className="bg-destructive"
className="text-muted-foreground"
className="border-border"

# GOOD — design scale (100-1200)
className="text-gray-1100"
className="bg-green-300"
className="border-blue-900"
```

**Allowed color tokens:**
- Semantic: `primary`, `destructive`, `muted`, `muted-foreground`, `foreground`, `background`, `card`, `accent`, `secondary`, `border`, `input`, `ring`, `sidebar-*`, `text-description`, `text-muted`
- Scale: `gray-{100-1200}`, `green-{100-1200}`, `blue-{100-1200}`, `purple-{100-1200}`, `yellow-{100-1200}`
- Brand: `fern` (`#51c233`)

**Hex → variant mapping for buttons:**
| Hex / Color | Maps to |
|------------|---------|
| `#008700`, `bg-green-*` (primary action) | `<Button>` (default variant = primary green) |
| Red/`bg-red-*`/`bg-destructive` | `<Button variant="destructive">` |
| `bg-white` with border | `<Button variant="outline">` |
| Transparent/no bg | `<Button variant="ghost">` |

**`bg-white` rule:** When replacing a card-like div or surface element, drop explicit `bg-white` — the UI component provides its own background via semantic tokens (`bg-card`, `bg-background`).

**Standard Tailwind vs Fern scale:** The Fern design system uses scales 100-1200 (e.g., `blue-300`, `gray-1100`). Standard Tailwind scales (50-950, e.g., `blue-100`, `red-500`) are NOT in the Fern system and should be flagged. If you see `blue-100` check whether it maps to a Fern scale step or standard Tailwind — Fern uses 100-1200 with different hex values.

### Phase 3: Spacing & Layout Audit

Flag arbitrary spacing values:

```
# BAD — arbitrary values
className="p-[17px]"
className="gap-[22px]"
className="mt-[3px]"
className="w-[347px]"

# GOOD — Tailwind scale
className="p-4"
className="gap-6"
className="mt-1"
className="w-full max-w-sm"
```

Exception: layout tokens like `--header-toolbar-height: 54px` are defined in the design system.

### Phase 4: Pattern Audit

Check for:
- Missing `"use client"` on interactive components
- Missing `data-slot` attributes on Radix wrappers
- Inline variant logic that should use CVA
- Class concatenation without `cn()`: `` className={`foo ${bar}`} `` → `className={cn("foo", bar)}`
- `import { cn } from "@/lib/utils"` → `@/utils/utils`
- Direct `clsx()` or `twMerge()` calls → use `cn()` instead

## Fix Workflow

For each finding:
1. Identify the replacement (component, token, or pattern)
2. Show before/after in the same file
3. Replace, preserving all behavior and accessibility
4. Verify: same visual output, same interactions, same a11y

## Before/After Examples

### Button
```tsx
// BEFORE
<button
    className="inline-flex items-center justify-center rounded-md bg-green-600 px-4 py-2 text-sm font-medium text-white hover:bg-green-700 disabled:opacity-50"
    onClick={handleSave}
    disabled={isLoading}
>
    Save changes
</button>

// AFTER
<Button onClick={handleSave} loading={isLoading}>
    Save changes
</Button>
```

### Card
```tsx
// BEFORE
<div className="rounded-xl border border-gray-200 bg-white p-6">
    <h3>Title</h3>
    <p>Content</p>
</div>

// AFTER
<Card>
    <h3>Title</h3>
    <p>Content</p>
</Card>
```

### Badge
```tsx
// BEFORE
<span className="rounded-full bg-blue-100 px-2 py-1 text-xs text-blue-800">
    Active
</span>

// AFTER
<Badge variant="info">Active</Badge>
```

### Colors
```tsx
// BEFORE
<p className="text-[#62636c]">Secondary text</p>
<div className="border border-[#e0e1e6]">

// AFTER
<p className="text-muted-foreground">Secondary text</p>
<div className="border-border">
```

### Class Merging
```tsx
// BEFORE
<div className={`flex items-center ${isActive ? "bg-blue-100" : ""} ${className}`}>

// AFTER
<div className={cn("flex items-center", isActive && "bg-blue-300", className)}>
```

## Audit Report Format

After scanning, present findings as:

```
## Audit Results

### Component Replacements (Phase 1)
- line 23: raw <button> → Button variant="outline"
- line 45: raw <input> → Input
- line 67: div card pattern → Card

### Color Issues (Phase 2)
- line 12: text-[#62636c] → text-muted-foreground
- line 34: bg-[#ff0000] → bg-destructive

### Spacing Issues (Phase 3)
- line 56: p-[17px] → p-4

### Pattern Issues (Phase 4)
- line 1: missing "use client"
- line 78: className template literal → cn()
```

Then fix each in order.

## What NOT to Migrate

- Layout `<div>` wrappers (flex containers, grid layouts)
- Components with complex behavior beyond what UI components offer
- One-off styled elements used exactly once with unique styling
- Editor/TipTap components (they have their own `--tt-*` token system)
- Third-party component wrappers
