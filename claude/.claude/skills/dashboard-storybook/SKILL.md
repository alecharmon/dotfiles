---
name: dashboard-storybook
description: Use when creating Storybook stories for dashboard components, when a new or modified component in packages/fern-dashboard/ needs story coverage, or when adding visual documentation for UI components.
---

# Dashboard Storybook Stories

## Overview

Create Storybook stories for components in `packages/fern-dashboard/`. Stories are living documentation — every UI component should have a `.stories.tsx` covering all variants and states.

**Setup:** `@storybook/react-vite` v9, dark/light theme toggle, `layout: "centered"` default, Next.js mocks included.

## Story File Template

```tsx
import type { Meta, StoryObj } from "@storybook/react";

import { MyComponent } from "./my-component";

const meta: Meta<typeof MyComponent> = {
    title: "UI/MyComponent",           // Category/Name
    component: MyComponent,
    parameters: { layout: "centered" },
    tags: ["autodocs"],
    argTypes: {
        variant: {
            control: { type: "select" },
            options: ["default", "outline", "ghost"]
        },
        disabled: { control: { type: "boolean" } }
    }
};

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {
    args: { children: "Default" }
};
```

**File location:** Same directory as the component, named `ComponentName.stories.tsx`.

**Title convention:** `"UI/ComponentName"` for base UI, `"Layout/ComponentName"` for layout, `"Upsells/ComponentName"` for upsell components.

## Story Patterns

### Simple Component (args only)
```tsx
export const Default: Story = {
    args: { children: "Click me", variant: "default" }
};

export const Destructive: Story = {
    args: { children: "Delete", variant: "destructive" }
};
```

### Compound Component (render function, no args)
For Dialog, Select, Tabs, DropdownMenu — use `render`:
```tsx
export const Default: Story = {
    render: () => (
        <Dialog>
            <DialogTrigger asChild>
                <Button variant="outline">Open</Button>
            </DialogTrigger>
            <DialogContent>
                <DialogHeader>
                    <DialogTitle>Title</DialogTitle>
                    <DialogDescription>Description</DialogDescription>
                </DialogHeader>
                <DialogBody>{/* content */}</DialogBody>
                <DialogFooter>
                    <DialogClose asChild>
                        <Button variant="outline">Cancel</Button>
                    </DialogClose>
                    <Button>Save</Button>
                </DialogFooter>
            </DialogContent>
        </Dialog>
    )
};
```

### Render with Args (composition)
```tsx
export const WithIcon: Story = {
    args: { children: "Add" },
    render: (args) => (
        <Button {...args}>
            <Plus />
            {args.children}
        </Button>
    )
};
```

### Interactive Story (useState in render)
```tsx
export const Interactive: Story = {
    render: () => {
        const [value, setValue] = useState("default");
        return (
            <div className="flex gap-2">
                <Button onClick={() => setValue("a")}>Set A</Button>
                <Button onClick={() => setValue("b")}>Set B</Button>
                <p>Current: {value}</p>
            </div>
        );
    }
};
```

### Provider Decorator (Tooltip, etc.)
```tsx
import * as TooltipPrimitive from "@radix-ui/react-tooltip";

const meta: Meta<typeof Tooltip> = {
    title: "UI/Tooltip",
    component: Tooltip,
    decorators: [
        (Story) => (
            <TooltipPrimitive.Provider>
                <Story />
            </TooltipPrimitive.Provider>
        )
    ]
};
```

### Callback Spies
```tsx
import { fn } from "storybook/test";  // NOT @storybook/test

const meta: Meta<typeof MyComponent> = {
    args: {
        onAction: fn(),
        onClose: fn().mockName("onClose")
    }
};
```

### ReactNode Slot Props
For components with named slot props (`afterSlot`, `icon`, `prefix`, etc.), use `render`:
```tsx
export const WithAfterSlot: Story = {
    render: () => (
        <StatusBadge status="live" afterSlot={<ChevronDown className="size-3" />} />
    )
};
```

### Boolean Modifier Props
For props like `hideDot`, `persistent`, `disabled` that toggle visual states:
```tsx
export const HiddenDot: Story = {
    name: "With hidden dot",
    args: { status: "live", hideDot: true }
};
```

### Named Stories
```tsx
export const AtLimit: Story = {
    name: "At member limit",  // Human-readable display name
    args: { count: 5, max: 5 }
};
```

## ArgType Controls

| Type | Control | Example |
|------|---------|---------|
| String union | `{ type: "select" }` | variant options |
| Boolean | `{ type: "boolean" }` | disabled, loading |
| Number | `{ type: "number" }` | count, max |
| Short string union | `{ type: "inline-radio" }` | 2-3 options |
| Free text | `{ type: "text" }` | labels, descriptions |

## Next.js Mocking

Already handled in `.storybook/mocks/`:
- `next/navigation` — `useRouter`, `usePathname`, `useSearchParams`, `useParams`, `redirect`, `notFound` all stubbed
- `next/dynamic` — uses `lazy` + `Suspense`
- `server-only` — empty stub

For components that import server-only providers: create a local "story wrapper" component that takes the same props but avoids pulling in server code.

## Coverage Checklist

Every new/modified UI component should have:

- [ ] **Default story** — component with default props
- [ ] **One story per variant** — each variant value gets its own story
- [ ] **Size variants** — if component has size prop
- [ ] **Loading state** — if component supports loading
- [ ] **Disabled state** — if component can be disabled
- [ ] **Boolean modifiers** — one story per boolean prop that changes visual output (`hideDot`, `persistent`, etc.)
- [ ] **Slot props** — if component has ReactNode props (`afterSlot`, `icon`, `prefix`), show with content
- [ ] **Edge cases** — empty content, overflow text, very long labels
- [ ] **Dark mode** — verified via Storybook theme toggle (no extra story needed)
- [ ] **With icons** — if component commonly takes icon children (use lucide-react)
- [ ] **Compound usage** — if component has sub-parts, show full composition

## Running Storybook

```bash
cd packages/fern-dashboard
pnpm storybook          # Dev server on port 6006
pnpm build-storybook    # Static build
```

## Common Mistakes

| Mistake | Fix |
|---------|-----|
| Import `fn` from `@storybook/test` | Use `from "storybook/test"` (v9) |
| Missing `tags: ["autodocs"]` | Always include for auto-generated docs |
| Stories outside `src/` | Must be in `src/**/*.stories.tsx` per config |
| Compound component with args only | Use `render: () => (...)` for compound components |
| Missing provider decorator | Tooltip needs `TooltipPrimitive.Provider` wrapper |
| Hardcoded dark story | Use the theme toggle in toolbar instead |
| Story for default export component | `import Card from "./card"` — Card is the only default export |
