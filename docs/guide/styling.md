# 样式系统

Industry Vis 采用现代化的设计系统，基于统一的设计令牌（Design Tokens）构建，提供一致的视觉体验和优秀的性能表现。

## 设计令牌系统

我们使用 CSS 自定义属性（CSS Variables）构建了完整的设计令牌系统，位于 `src/styles/tokens.css`。

### 令牌分类

#### 间距系统
```css
--space-xs: 4px    /* 极小间距 */
--space-sm: 8px    /* 小间距 */
--space-md: 16px   /* 中等间距 */
--space-lg: 24px   /* 大间距 */
--space-xl: 32px   /* 超大间距 */
--space-2xl: 48px  /* 极大间距 */
--space-3xl: 64px  /* 最大间距 */
```

#### 字体系统
```css
/* 字体大小 */
--text-xs: 11px    /* 极小文字 */
--text-sm: 13px    /* 小文字 */
--text-base: 15px  /* 基础文字 */
--text-lg: 17px    /* 大文字 */
--text-xl: 19px    /* 超大文字 */
--text-2xl: 23px   /* 标题文字 */
--text-3xl: 27px   /* 主标题文字 */

/* 字体粗细 */
--font-weight-normal: 400
--font-weight-medium: 500
--font-weight-semibold: 600
--font-weight-bold: 700
```

#### 圆角系统
```css
--radius-sm: 4px   /* 小圆角 */
--radius-md: 8px   /* 中圆角 */
--radius-lg: 12px  /* 大圆角 */
--radius-xl: 16px  /* 超大圆角 */
--radius-2xl: 24px /* 极大圆角 */
--radius-full: 9999px /* 完全圆角 */
```

#### 阴影系统
```css
--shadow-sm: 0 1px 2px rgba(0, 0, 0, 0.05)
--shadow-md: 0 4px 6px rgba(0, 0, 0, 0.07)
--shadow-lg: 0 10px 15px rgba(0, 0, 0, 0.1)
--shadow-xl: 0 20px 25px rgba(0, 0, 0, 0.1)
--shadow-2xl: 0 25px 50px rgba(0, 0, 0, 0.25)
```

#### 动画系统
```css
/* 缓动函数 */
--ease-in-out: cubic-bezier(0.4, 0, 0.2, 1)
--ease-out: cubic-bezier(0, 0, 0.2, 1)
--ease-in: cubic-bezier(0.4, 0, 1, 1)

/* 动画时长 */
--transition-fast: 150ms var(--ease-in-out)
--transition-normal: 200ms var(--ease-in-out)
--transition-slow: 300ms var(--ease-in-out)
```

## 主题系统

支持明暗两种主题，自动跟随系统设置或手动切换。

### 主题切换

```javascript
// 切换到亮色主题
document.documentElement.classList.remove('dark')
document.documentElement.classList.add('light')

// 切换到暗色主题
document.documentElement.classList.remove('light')
document.documentElement.classList.add('dark')

// 跟随系统主题
document.documentElement.classList.remove('light', 'dark')
```

### 主题变量

主题相关的颜色变量定义在 `src/styles/themes.css` 中：

#### 亮色主题
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

#### 暗色主题
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

## 玻璃拟物设计

系统采用玻璃拟物设计风格，提供现代感的视觉体验。

### 玻璃卡片

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

### 玻璃效果变量
```css
.glass-bg {
  background: rgba(255, 255, 255, 0.1);
  backdrop-filter: blur(10px);
}

.glass-border {
  border: 1px solid rgba(255, 255, 255, 0.2);
}
```

## 性能优化

样式系统包含多种性能优化措施：

### CSS Containment
```css
.glass-card {
  contain: layout style paint;
  will-change: transform;
}
```

### 硬件加速
```css
.transform-gpu {
  transform: translateZ(0);
  backface-visibility: hidden;
}
```

### 高效动画
```css
.smooth-transition {
  transition: var(--transition-fast);
  transition-property: transform, opacity;
}
```

## 使用指南

### 1. 使用设计令牌

```vue
<template>
  <div class="card">
    <h2 class="title">标题</h2>
    <p class="description">描述文本</p>
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

### 2. 响应式设计

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

### 3. 主题感知组件

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

## 最佳实践

### 1. 令牌优先
始终使用设计令牌而不是硬编码值：
```css
/* ❌ 错误 */
.button {
  padding: 8px 16px;
  border-radius: 8px;
}

/* ✅ 正确 */
.button {
  padding: var(--space-sm) var(--space-md);
  border-radius: var(--radius-md);
}
```

### 2. 语义化命名
使用语义化的类名：
```css
/* ❌ 错误 */
.mb-16 { margin-bottom: 16px; }

/* ✅ 正确 */
.card-spacing { margin-bottom: var(--space-md); }
```

### 3. 性能意识
避免过度使用昂贵的CSS属性：
```css
/* ❌ 错误 - 过度使用模糊 */
.heavy-blur {
  backdrop-filter: blur(20px);
  filter: blur(10px);
}

/* ✅ 正确 - 适度使用 */
.glass-effect {
  backdrop-filter: var(--glass-blur);
}
```

## 文件结构

```
src/styles/
├── tokens.css     # 设计令牌系统
├── themes.css     # 主题定义
├── glass.css      # 玻璃拟物效果
├── main.css       # 主样式文件
└── components/    # 组件特定样式
```

这套设计系统确保了整个应用的视觉一致性，同时提供了优秀的性能和可维护性。