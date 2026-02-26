const files = [
  './node_modules/ink/build/reconciler.js',
  './node_modules/ink/build/renderer.js',
  './node_modules/ink/build/dom.js',
  './node_modules/ink/build/log-update.js',
  './node_modules/ink/build/instances.js',
  './node_modules/ink/build/components/App.js',
  './node_modules/ink/build/components/AccessibilityContext.js',
];
for (const file of files) {
  console.log('import', file);
  await import(file);
  console.log('loaded', file);
}
console.log('all ink internals loaded');
