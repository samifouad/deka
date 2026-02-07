console.log('dom start');
await import('./node_modules/ink/build/dom.js');
console.log('dom loaded');
await import('./node_modules/ink/build/styles.js');
console.log('styles loaded');
