/**
 * Mock Identity Server for Testing QR Authentication
 *
 * This simulates the backend identity service for testing the QR login flow.
 * Run with: node mock-identity-server.js
 */

const http = require('http')
const url = require('url')

// Store active sessions
const sessions = new Map()

// CORS headers
const corsHeaders = {
  'Access-Control-Allow-Origin': '*',
  'Access-Control-Allow-Methods': 'GET, POST, OPTIONS',
  'Access-Control-Allow-Headers': 'Content-Type',
}

// Create session endpoint
function createSession(req, res) {
  let body = ''

  req.on('data', chunk => {
    body += chunk.toString()
  })

  req.on('end', () => {
    const data = JSON.parse(body)

    const sessionId = `session_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`
    const challenge = `challenge_${Math.random().toString(36).substr(2, 32)}`

    const sessionData = {
      sessionId,
      challenge,
      qrData: {
        protocol: 'tana',
        type: 'auth',
        sessionId,
        challenge,
        server: 'http://localhost:8090'
      },
      expiresIn: 300,
      expiresAt: new Date(Date.now() + 300000).toISOString(),
      status: 'waiting'
    }

    sessions.set(sessionId, sessionData)

    console.log(`✓ Created session: ${sessionId}`)
    console.log(`  App: ${data.appName}`)
    console.log(`  Return URL: ${data.returnUrl}`)

    res.writeHead(200, {
      'Content-Type': 'application/json',
      ...corsHeaders
    })
    res.end(JSON.stringify(sessionData))
  })
}

// SSE endpoint
function handleSSE(req, res, sessionId) {
  console.log(`✓ SSE connected for session: ${sessionId}`)

  // Set SSE headers
  res.writeHead(200, {
    'Content-Type': 'text/event-stream',
    'Cache-Control': 'no-cache',
    'Connection': 'keep-alive',
    ...corsHeaders
  })

  // Send initial status
  const sendEvent = (data) => {
    res.write(`data: ${JSON.stringify(data)}\n\n`)
  }

  sendEvent({
    type: 'status_update',
    sessionId,
    status: 'waiting',
    timestamp: new Date().toISOString()
  })

  // Simulate scan after 3 seconds
  const scanTimeout = setTimeout(() => {
    console.log(`📱 Simulating scan for: ${sessionId}`)
    sendEvent({
      type: 'status_update',
      sessionId,
      status: 'scanned',
      timestamp: new Date().toISOString()
    })
  }, 3000)

  // Simulate approval after 6 seconds
  const approveTimeout = setTimeout(() => {
    console.log(`✅ Simulating approval for: ${sessionId}`)
    sendEvent({
      type: 'approved',
      sessionId,
      sessionToken: `token_${Math.random().toString(36).substr(2, 32)}`,
      userId: 'user_123',
      username: 'alice',
      timestamp: new Date().toISOString()
    })

    // Close connection after approval
    setTimeout(() => {
      res.end()
    }, 100)
  }, 6000)

  // Handle client disconnect
  req.on('close', () => {
    console.log(`✗ SSE disconnected for: ${sessionId}`)
    clearTimeout(scanTimeout)
    clearTimeout(approveTimeout)
  })
}

// Create HTTP server
const server = http.createServer((req, res) => {
  const parsedUrl = url.parse(req.url, true)
  const pathname = parsedUrl.pathname

  // Handle CORS preflight
  if (req.method === 'OPTIONS') {
    res.writeHead(204, corsHeaders)
    res.end()
    return
  }

  // Create session
  if (pathname === '/auth/session/create' && req.method === 'POST') {
    createSession(req, res)
    return
  }

  // SSE events
  const sseMatch = pathname.match(/^\/auth\/session\/(.+)\/events$/)
  if (sseMatch && req.method === 'GET') {
    const sessionId = sseMatch[1]
    handleSSE(req, res, sessionId)
    return
  }

  // 404
  res.writeHead(404, { 'Content-Type': 'text/plain', ...corsHeaders })
  res.end('Not Found')
})

// Start server
const PORT = 8090
server.listen(PORT, () => {
  console.log('\n=================================')
  console.log('🚀 Mock Identity Server Running')
  console.log('=================================')
  console.log(`URL: http://localhost:${PORT}`)
  console.log('\nEndpoints:')
  console.log('  POST /auth/session/create')
  console.log('  GET  /auth/session/:id/events')
  console.log('\nTest Flow:')
  console.log('  1. QR code appears immediately')
  console.log('  2. After 3s: "Scanned" status')
  console.log('  3. After 6s: "Approved" → redirect')
  console.log('\nPress Ctrl+C to stop')
  console.log('=================================\n')
})
