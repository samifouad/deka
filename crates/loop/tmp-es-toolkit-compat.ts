console.log('compat start');
const mod = await import('es-toolkit/compat');
console.log('compat loaded', typeof mod, Object.keys(mod).length);
