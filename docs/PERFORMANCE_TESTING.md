# Performance Testing Guide

## Quick Performance Check

### 1. React DevTools Profiler

1. Install React DevTools browser extension
2. Open DevTools → Profiler tab
3. Click "Record" button
4. Perform actions (filter models, switch pages, etc.)
5. Stop recording
6. Analyze flame graph for slow components

**What to look for**:
- Components taking > 16ms to render
- Unnecessary re-renders (same props, different render)
- Deep component trees

### 2. Chrome DevTools Performance

1. Open DevTools → Performance tab
2. Click "Record" button
3. Perform actions for 5-10 seconds
4. Stop recording
5. Analyze timeline

**What to look for**:
- Long tasks (> 50ms)
- Layout thrashing
- Excessive JavaScript execution
- Memory leaks

### 3. Lighthouse Audit

```bash
# Run Lighthouse in Chrome DevTools
# Or use CLI:
npm install -g lighthouse
lighthouse http://localhost:1420 --view
```

**Key Metrics**:
- First Contentful Paint (FCP): < 1.8s
- Largest Contentful Paint (LCP): < 2.5s
- Time to Interactive (TTI): < 3.8s
- Total Blocking Time (TBT): < 200ms
- Cumulative Layout Shift (CLS): < 0.1

## Performance Test Scenarios

### Scenario 1: Model List Performance

**Test**: Rapid filter changes
```typescript
// Expected behavior:
// - Filters update instantly (< 100ms)
// - Only affected cards re-render
// - No layout shift
```

**Steps**:
1. Navigate to Models page
2. Open React DevTools Profiler
3. Start recording
4. Click through all filters rapidly
5. Stop recording
6. Check render times

**Success Criteria**:
- Each filter change < 50ms
- Only ModelSettings and affected ModelCards re-render
- No full page re-renders

### Scenario 2: Large Model List (Virtual Scrolling)

**Test**: Scrolling performance with 50+ models
```typescript
// Expected behavior:
// - Smooth 60fps scrolling
// - Only visible cards rendered
// - Memory usage stable
```

**Steps**:
1. Temporarily increase MODELS array to 50+ items
2. Navigate to Models page
3. Open Chrome Performance tab
4. Start recording
5. Scroll up and down rapidly
6. Stop recording

**Success Criteria**:
- Scrolling maintains 60fps
- No dropped frames
- Memory usage < 100MB increase

### Scenario 3: Page Navigation

**Test**: Switching between pages
```typescript
// Expected behavior:
// - Page loads in < 200ms
// - Lazy loading works correctly
// - No unnecessary re-renders
```

**Steps**:
1. Open React DevTools Profiler
2. Start recording
3. Navigate through all pages
4. Stop recording

**Success Criteria**:
- First page load < 200ms
- Subsequent loads < 100ms (cached)
- Only new page component renders

### Scenario 4: AI Prompts List

**Test**: Managing 50+ prompts
```typescript
// Expected behavior:
// - List renders smoothly
// - Toggle/edit actions instant
// - No lag on interactions
```

**Steps**:
1. Create 50+ AI prompts (or mock data)
2. Open React DevTools Profiler
3. Start recording
4. Toggle several prompts
5. Edit a prompt
6. Delete a prompt
7. Stop recording

**Success Criteria**:
- Each action < 50ms
- Only affected PromptCard re-renders
- Modal opens instantly

### Scenario 5: Memory Leak Detection

**Test**: Long-running session
```typescript
// Expected behavior:
// - Memory usage stable over time
// - No memory leaks
// - Proper cleanup on unmount
```

**Steps**:
1. Open Chrome DevTools → Memory tab
2. Take heap snapshot (baseline)
3. Navigate through all pages 10 times
4. Take another heap snapshot
5. Compare snapshots

**Success Criteria**:
- Memory increase < 10MB
- No detached DOM nodes
- Event listeners properly cleaned up

## Automated Performance Tests

### Bundle Size Monitoring

```bash
# Analyze bundle size
npm run build
npx vite-bundle-visualizer
```

**Thresholds**:
- Main bundle: < 300KB (gzipped)
- Each lazy chunk: < 100KB (gzipped)
- Total bundle: < 1MB (gzipped)

### Performance Budget

```json
// Add to package.json
{
  "performance": {
    "maxBundleSize": "300kb",
    "maxChunkSize": "100kb",
    "maxAssetSize": "500kb"
  }
}
```

## Performance Regression Testing

### Before Each Release

1. **Run Lighthouse audit**
   - Score should be > 90 for Performance
   - No regressions from previous version

2. **Check bundle size**
   - Compare with previous build
   - Investigate increases > 10%

3. **Profile key interactions**
   - Model filtering
   - Page navigation
   - Prompt management

4. **Memory profiling**
   - Check for leaks
   - Verify cleanup

### Continuous Monitoring

```bash
# Add to CI/CD pipeline
npm run build
npm run analyze-bundle
npm run lighthouse-ci
```

## Common Performance Issues

### Issue 1: Unnecessary Re-renders

**Symptoms**:
- Components re-render with same props
- Slow interactions

**Solutions**:
- Add `React.memo`
- Use `useMemo` for computed values
- Use `useCallback` for event handlers

### Issue 2: Large Bundle Size

**Symptoms**:
- Slow initial load
- Large JavaScript files

**Solutions**:
- Code splitting with `React.lazy`
- Tree shaking
- Remove unused dependencies

### Issue 3: Slow List Rendering

**Symptoms**:
- Lag when scrolling
- Slow filter updates

**Solutions**:
- Virtual scrolling
- Memoize list items
- Debounce filter changes

### Issue 4: Memory Leaks

**Symptoms**:
- Memory usage increases over time
- App becomes sluggish

**Solutions**:
- Clean up event listeners
- Cancel pending requests
- Clear timers/intervals

## Performance Optimization Checklist

- [ ] All list items use stable keys
- [ ] Event handlers wrapped in `useCallback`
- [ ] Expensive computations use `useMemo`
- [ ] Pure components use `React.memo`
- [ ] Large lists use virtual scrolling
- [ ] Images use lazy loading
- [ ] Routes use code splitting
- [ ] Bundle size within budget
- [ ] No memory leaks detected
- [ ] Lighthouse score > 90

## Tools & Resources

### Browser Extensions
- React DevTools
- Redux DevTools (if using Redux)
- Lighthouse

### NPM Packages
```bash
npm install -D vite-bundle-visualizer
npm install -D lighthouse-ci
npm install -D webpack-bundle-analyzer
```

### Online Tools
- [WebPageTest](https://www.webpagetest.org/)
- [PageSpeed Insights](https://pagespeed.web.dev/)
- [Bundle Phobia](https://bundlephobia.com/)

## Reporting Performance Issues

When reporting performance issues, include:

1. **Environment**
   - OS and version
   - Browser and version
   - App version

2. **Reproduction Steps**
   - Exact steps to reproduce
   - Expected vs actual behavior

3. **Performance Data**
   - React DevTools Profiler screenshot
   - Chrome Performance timeline
   - Memory snapshots (if applicable)

4. **Impact**
   - User-facing impact
   - Frequency of occurrence
   - Severity (critical/high/medium/low)

## Conclusion

Regular performance testing ensures Recording King remains fast and responsive. Follow this guide before each release and when adding new features.
