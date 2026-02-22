console.log('preload ink start');
await import('yoga-layout');
console.log('preloaded yoga');
const React = await import('react');
console.log('react loaded');
const ink = await import('ink');
console.log('ink loaded');
const { render, Text } = ink;
render(React.createElement(Text, null, 'ink ok'));
