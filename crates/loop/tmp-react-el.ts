import React from 'react';
const el = React.createElement('div', null, 'hi');
console.log('keys', Object.keys(el));
console.log('symbols', Object.getOwnPropertySymbols(el));
console.log('typeof', typeof el.$$typeof, el.$$typeof?.toString());
