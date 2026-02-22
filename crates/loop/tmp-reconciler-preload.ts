console.log('preload start');
await import('yoga-layout');
console.log('yoga preloaded');
await import('./node_modules/ink/build/reconciler.js');
console.log('reconciler loaded');
