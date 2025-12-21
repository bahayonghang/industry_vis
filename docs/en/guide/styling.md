# Styling System

Industry Vis adopts a modern design system built on unified design tokens, providing consistent visual experience and excellent performance.

## Design Token System

We use CSS custom properties (CSS Variables) to build a comprehensive design token system, located in `src/styles/tokens.css`.

### Token Categories

#### Spacing System
```css
--space-xs: 4px    /* Extra small spacing */
--space-sm: 8px    /* Small spacing */
--space-md: 16px   /* Medium spacing */
--space-lg: 24px   /* Large spacing */
--space-xl: 32px   /* Extra large spacing */
--space-2xl: 48px  /* Double extra large spacing */
--space-3xl: 64px  /* Triple extra large spacing */
```

#### Typography System
```css
/* Font sizes */
--text-xs: 11px    /* Extra small text */
--text-sm: 13px    /* Small text */
--text-base: 15px  /* Base text */
--text-lg: 17px    /* Large text */
--text-xl: 19px    /* Extra large text */
--text-2xl: 23px   /* Title text */
--text-3xl: 27px   /* Main title text */

/* Font weights */
--font-weight-normal: 400
--font-weight-medium: 500
--font-weight-semibold: 600
--font-weight-bold: 700
```

#### Border Radius System
```css
--radius-sm: 4px   /* Small radius */
--radius-md: 8px   /* Medium radius */
--radius-lg: 12px  /* Large radius */
--radius-xl: 16px  /* Extra large radius */
--radius-2xl: 24px /* Double extra large radius */
--radius-full: 9999px /* Full radius */
```

#### Shadow System
```css
--shadow-sm: 0 1px 2px rgba(0, 0, 0, 0.05)
--shadow-md: 0 4px 6px rgba(0, 0, 0, 0.07)
--shadow-lg: 0 10px 15px rgba(0, 0, 0, 0.1)
--shadow-xl: 0 20px 25px rgba(0, 0, 0, 0.1)
--shadow-2xl: 0 25px 50px rgba(0, 0, 0, 0.25)
```

#### Animation System
```css
/* Easing functions */
--ease-in-out: cubic-bezier(0.4, 0, 0.2, 1)
--ease-out: cubic-bezier(0, 0, 0.2, 1)
--ease-in: cubic-bezier(0.4, 0, 1, 1)

/* Animation durations */
--transition-fast: 150ms var(--ease-in-out)
--transition-normal: 200ms var(--ease-in-out)
--transition-slow: 300ms var(--ease-in-out)
```

## Theme System

Supports both light and dark themes, automatically following system settings or manual switching.

### Theme Switching

```javascript
// Switch to light theme
document.documentElement.classList.remove('dark')
document.documentElement.classList.add('light')

// Switch to dark theme
document.documentElement.classList.remove('light')
document.documentElement.classList.add('dark')

// Follow system theme
document.documentElement.classList.remove('light', 'dark')
```

### Theme Variables

Theme-related color variables are defined in `src/styles/themes.css`:

#### Light Theme
```css
.light, :root:not(.dark) {
  --bg-primary: #ffffff;
  --bg-secondary: #f8f9fa;
  --bg-tertiary: #f1f3f4;
  --text-primary: #202124;
  --text-secondary: #5f6368;
  --text-tertiary: #80868b;
  --border-color: #dadce0;
  --primary-color: #1a73e8;
  --bg-hover: rgba(26, 115, 232, 0.04);
}
```

#### Dark Theme
```css
.dark {
  --bg-primary: #202124;
  --bg-secondary: #292a2d;
  --bg-tertiary: #3e3f42;
  --text-primary: #e8eaed;
  --text-secondary: #9aa0a6;
  --text-tertiary: #80868b;
  --border-color: #5f6368;
  --primary-color: #8ab4f8;
  --bg-hover: rgba(138, 180, 248, 0.04);
}
```

## Glassmorphism Design

The system adopts glassmorphism design style, providing a modern visual experience.

### Glass Cards

```css
.glass-card {
  background: var(--glass-bg);
  backdrop-filter: var(--glass-blur);
  border: 1px solid var(--glass-border);
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-lg);
  transition: var(--transition-normal);
}
```

### Glass Effect Variables
```css
.glass-bg {
  background: rgba(255, 255, 255, 0.1);
  backdrop-filter: blur(10px);
}

.glass-border {
  border: 1px solid rgba(255, 255, 255, 0.2);
}
```

## Performance Optimization

The styling system includes multiple performance optimization measures:

### CSS Containment
```css
.glass-card {
  contain: layout style paint;
  will-change: transform;
}
```

### Hardware Acceleration
```css
.transform-gpu {
  transform: translateZ(0);
  backface-visibility: hidden;
}
```

### Efficient Animations
```css
.smooth-transition {
  transition: var(--transition-fast);
  transition-property: transform, opacity;
}
```

## Usage Guidelines

### 1. Using Design Tokens

```vue
<template>
  <div class="card">
    <h2 class="title">Title</h2>
    <p class="description">Description text</p>
  </div>
</template>

<style scoped>
.card {
  padding: var(--space-lg);
  margin: var(--space-md);
  border-radius: var(--radius-lg);
  background: var(--bg-primary);
  box-shadow: var(--shadow-md);
}

.title {
  font-size: var(--text-xl);
  font-weight: var(--font-weight-semibold);
  color: var(--text-primary);
  margin-bottom: var(--space-sm);
}

.description {
  font-size: var(--text-sm);
  color: var(--text-secondary);
  line-height: 1.5;
}
</style>
```

### 2. Responsive Design

```css
.responsive-container {
  padding: var(--space-md);
}

@media (min-width: 768px) {
  .responsive-container {
    padding: var(--space-lg);
  }
}

@media (min-width: 1024px) {
  .responsive-container {
    padding: var(--space-xl);
  }
}
```

### 3. Theme-Aware Components

```vue
<template>
  <div class="theme-aware-component">
    <slot />
  </div>
</template>

<style scoped>
.theme-aware-component {
  background: var(--bg-primary);
  color: var(--text-primary);
  border: 1px solid var(--border-color);
  transition: var(--transition-normal);
}

.theme-aware-component:hover {
  background: var(--bg-hover);
}
</style>
```

## Best Practices

### 1. Token First
Always use design tokens instead of hardcoded values:
```css
/* ❌ Wrong */
.button {
  padding: 8px 16px;
  border-radius: 8px;
}

/* ✅ Correct */
.button {
  padding: var(--space-sm) var(--space-md);
  border-radius: var(--radius-md);
}
```

### 2. Semantic Naming
Use semantic class names:
```css
/* ❌ Wrong */
.mb-16 { margin-bottom: 16px; }

/* ✅ Correct */
.card-spacing { margin-bottom: var(--space-md); }
```

### 3. Performance Awareness
Avoid overusing expensive CSS properties:
```css
/* ❌ Wrong - Overusing blur */
.heavy-blur {
  backdrop-filter: blur(20px);
  filter: blur(10px);
}

/* ✅ Correct - Moderate usage */
.glass-effect {
  backdrop-filter: var(--glass-blur);
}
```

## File Structure

```
src/styles/
├── tokens.css     # Design token system
├── themes.css     # Theme definitions
├── glass.css      # Glassmorphism effects
├── main.css       # Main stylesheet
└── components/    # Component-specific styles
```

This design system ensures visual consistency across the entire application while providing excellent performance and maintainability.