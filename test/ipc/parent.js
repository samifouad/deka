#!/usr/bin/env node
// Parent process for IPC testing

const { fork } = require('child_process');
const path = require('path');

console.log('[Parent] Starting IPC test...');
console.log('[Parent] Parent PID:', process.pid);

const workerPath = path.join(__dirname, 'worker.js');
console.log('[Parent] Forking worker:', workerPath);

const child = fork(workerPath);

console.log('[Parent] Child forked with PID:', child.pid);
console.log('[Parent] Child connected:', child.connected);

let messageCount = 0;

// Listen for messages from child
child.on('message', (msg) => {
    console.log('[Parent] Received from child:', JSON.stringify(msg));
    messageCount++;

    if (msg.status === 'ready') {
        console.log('[Parent] Child is ready, sending ping...');
        child.send({ command: 'ping' });
    } else if (msg.response === 'pong') {
        console.log('[Parent] Got pong! Sending echo test...');
        child.send({ command: 'echo', data: 'Hello from parent!' });
    } else if (msg.echo) {
        console.log('[Parent] Got echo response:', msg.echo);
        console.log('[Parent] Sending exit command...');
        child.send({ command: 'exit' });
    }
});

child.on('disconnect', () => {
    console.log('[Parent] Child disconnected');
});

child.on('exit', (code, signal) => {
    console.log(`[Parent] Child exited with code ${code}, signal ${signal}`);
    console.log(`[Parent] Total messages exchanged: ${messageCount}`);

    if (messageCount >= 3) {
        console.log('\n✓ SUCCESS: IPC test passed!');
        console.log('  - Child sent ready message');
        console.log('  - Ping/pong exchange worked');
        console.log('  - Echo test worked');
        process.exit(0);
    } else {
        console.error('\n✗ FAILED: Expected at least 3 messages, got', messageCount);
        process.exit(1);
    }
});

child.on('error', (err) => {
    console.error('[Parent] Child process error:', err);
    process.exit(1);
});

// Timeout after 5 seconds
setTimeout(() => {
    console.error('\n✗ TIMEOUT: Test did not complete in 5 seconds');
    child.kill();
    process.exit(1);
}, 5000);
