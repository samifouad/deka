// Child process for IPC testing

console.log('[Child] Worker started, PID:', process.pid);
console.log('[Child] IPC connected:', process.connected);

if (process.connected) {
    // Listen for messages from parent
    process.on('message', (msg) => {
        console.log('[Child] Received from parent:', JSON.stringify(msg));

        // Send response back to parent
        if (msg.command === 'ping') {
            process.send({ response: 'pong', timestamp: Date.now() });
        } else if (msg.command === 'echo') {
            process.send({ echo: msg.data });
        } else if (msg.command === 'exit') {
            console.log('[Child] Received exit command');
            process.exit(0);
        } else {
            process.send({ error: 'Unknown command', received: msg });
        }
    });

    // Send ready message to parent
    process.send({ status: 'ready', pid: process.pid });

    console.log('[Child] Sent ready message to parent');
} else {
    console.error('[Child] ERROR: IPC not connected!');
    process.exit(1);
}
