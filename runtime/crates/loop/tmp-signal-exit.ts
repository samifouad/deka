const mod = await import('signal-exit');
console.log('signal-exit exports', Object.keys(mod));
console.log('default', mod.default);
