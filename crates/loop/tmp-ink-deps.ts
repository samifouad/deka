const deps = [
  'ansi-escapes',
  'is-in-ci',
  'auto-bind',
  'signal-exit',
  'patch-console',
  'wrap-ansi',
  'log-update',
];
for (const dep of deps) {
  console.log('import', dep);
  await import(dep);
  console.log('loaded', dep);
}
console.log('all deps loaded');
