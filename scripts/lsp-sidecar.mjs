#!/usr/bin/env node
import http from 'node:http'
import path from 'node:path'
import process from 'node:process'
import { spawn } from 'node:child_process'

const wosixRoot = process.env.WOSIX_ROOT || process.cwd()
const repoRoot = path.resolve(wosixRoot, '..')
const lspBin = process.env.PHPX_LSP_BIN || path.resolve(repoRoot, 'target/release/phpx_lsp')
const host = process.env.PHPX_LSP_HOST || '127.0.0.1'
const port = Number(process.env.PHPX_LSP_PORT || 8531)

class LspBridge {
  constructor() {
    this.proc = null
    this.nextId = 1
    this.buffer = ''
    this.pending = new Map()
    this.initialized = false
    this.opened = new Set()
  }

  ensureStarted() {
    if (this.proc) return
    this.proc = spawn(lspBin, [], {
      stdio: ['pipe', 'pipe', 'pipe'],
      env: {
        ...process.env,
        PHPX_MODULE_ROOT: process.env.PHPX_MODULE_ROOT || repoRoot,
      },
    })

    this.proc.stdout.on('data', (chunk) => this.consumeStdout(chunk))
    this.proc.stderr.on('data', (_chunk) => {})

    const onDown = (err) => {
      this.proc = null
      this.initialized = false
      this.opened.clear()
      for (const { reject } of this.pending.values()) reject(err || new Error('phpx_lsp exited'))
      this.pending.clear()
    }

    this.proc.on('error', onDown)
    this.proc.on('exit', () => onDown())
  }

  consumeStdout(chunk) {
    this.buffer += chunk.toString('utf8')
    for (;;) {
      const crlf = this.buffer.indexOf('\r\n\r\n')
      const lf = this.buffer.indexOf('\n\n')
      let headEnd = crlf
      let sepLen = 4
      if (headEnd < 0 || (lf >= 0 && lf < headEnd)) {
        headEnd = lf
        sepLen = 2
      }
      if (headEnd < 0) return

      const head = this.buffer.slice(0, headEnd)
      const match = head.match(/Content-Length:\s*(\d+)/i)
      if (!match) {
        this.buffer = this.buffer.slice(headEnd + sepLen)
        continue
      }

      const len = Number(match[1])
      const bodyStart = headEnd + sepLen
      const bodyEnd = bodyStart + len
      if (this.buffer.length < bodyEnd) return

      const body = this.buffer.slice(bodyStart, bodyEnd)
      this.buffer = this.buffer.slice(bodyEnd)

      let msg = null
      try { msg = JSON.parse(body) } catch { continue }
      if (msg && typeof msg.id === 'number' && this.pending.has(msg.id)) {
        const pending = this.pending.get(msg.id)
        this.pending.delete(msg.id)
        if (msg.error) pending.reject(new Error(String(msg.error.message || 'LSP error')))
        else pending.resolve(msg.result)
      }
    }
  }

  request(method, params) {
    this.ensureStarted()
    if (!this.proc) throw new Error('phpx_lsp unavailable')
    const id = this.nextId++
    const payload = JSON.stringify({ jsonrpc: '2.0', id, method, params })
    const frame = `Content-Length: ${Buffer.byteLength(payload, 'utf8')}\r\n\r\n${payload}`
    return new Promise((resolve, reject) => {
      const timer = setTimeout(() => {
        if (this.pending.has(id)) {
          this.pending.delete(id)
          reject(new Error(`phpx_lsp timeout for ${method}`))
        }
      }, 8000)
      this.pending.set(id, {
        resolve: (value) => {
          clearTimeout(timer)
          resolve(value)
        },
        reject: (err) => {
          clearTimeout(timer)
          reject(err)
        },
      })
      this.proc.stdin.write(frame)
    })
  }

  notify(method, params) {
    this.ensureStarted()
    if (!this.proc) throw new Error('phpx_lsp unavailable')
    const payload = JSON.stringify({ jsonrpc: '2.0', method, params })
    const frame = `Content-Length: ${Buffer.byteLength(payload, 'utf8')}\r\n\r\n${payload}`
    this.proc.stdin.write(frame)
  }

  async init() {
    if (this.initialized) return
    const rootUri = 'file:///workspace'
    await this.request('initialize', {
      processId: process.pid,
      rootUri,
      capabilities: {
        textDocument: {
          diagnostic: { dynamicRegistration: false },
          completion: { dynamicRegistration: false },
          hover: { dynamicRegistration: false },
        },
      },
      workspaceFolders: [{ uri: rootUri, name: 'workspace' }],
      initializationOptions: {
        phpx: { target: 'wosix' },
      },
    })
    this.notify('initialized', {})
    this.initialized = true
  }

  async upsertText(uri, text) {
    await this.init()
    if (!this.opened.has(uri)) {
      this.notify('textDocument/didOpen', {
        textDocument: {
          uri,
          languageId: 'phpx',
          version: 1,
          text,
        },
      })
      this.opened.add(uri)
      return
    }
    this.notify('textDocument/didChange', {
      textDocument: { uri, version: Date.now() },
      contentChanges: [{ text }],
    })
  }

  async diagnostics(uri, text) {
    await this.upsertText(uri, text)
    const out = await this.request('textDocument/diagnostic', { textDocument: { uri } })
    if (out && typeof out === 'object' && Array.isArray(out.items)) return out.items
    return []
  }

  async completion(uri, text, line, character) {
    await this.upsertText(uri, text)
    const out = await this.request('textDocument/completion', {
      textDocument: { uri },
      position: { line, character },
    })
    if (Array.isArray(out)) return out
    if (out && typeof out === 'object' && Array.isArray(out.items)) return out.items
    return []
  }

  async hover(uri, text, line, character) {
    await this.upsertText(uri, text)
    return this.request('textDocument/hover', {
      textDocument: { uri },
      position: { line, character },
    })
  }
}

const lsp = new LspBridge()

const send = (res, status, obj) => {
  const body = JSON.stringify(obj)
  res.writeHead(status, {
    'content-type': 'application/json; charset=utf-8',
    'access-control-allow-origin': '*',
    'access-control-allow-methods': 'GET, OPTIONS',
    'access-control-allow-headers': 'content-type',
    'content-length': Buffer.byteLength(body),
  })
  res.end(body)
}

const server = http.createServer(async (req, res) => {
  try {
    if (req.method === 'OPTIONS') {
      res.writeHead(204, {
        'access-control-allow-origin': '*',
        'access-control-allow-methods': 'GET, OPTIONS',
        'access-control-allow-headers': 'content-type',
      })
      res.end()
      return
    }

    const url = new URL(req.url || '/', `http://${host}:${port}`)
    if (url.pathname === '/ping') {
      send(res, 200, { ok: true })
      return
    }

    const uri = url.searchParams.get('uri') || 'file:///workspace/main.phpx'
    const text = String(url.searchParams.get('text') ?? '')
    const line = Number(url.searchParams.get('line') ?? 0)
    const character = Number(url.searchParams.get('character') ?? 0)

    if (url.pathname === '/diagnostics') {
      const items = await lsp.diagnostics(uri, text)
      send(res, 200, { ok: true, items })
      return
    }
    if (url.pathname === '/completion') {
      const items = await lsp.completion(uri, text, line, character)
      send(res, 200, { ok: true, items })
      return
    }
    if (url.pathname === '/hover') {
      const hover = await lsp.hover(uri, text, line, character)
      send(res, 200, { ok: true, hover })
      return
    }

    send(res, 404, { ok: false, error: 'not found' })
  } catch (err) {
    send(res, 500, { ok: false, error: err instanceof Error ? err.message : String(err) })
  }
})

server.listen(port, host, () => {
  process.stdout.write(`[lsp-sidecar] listening http://${host}:${port}\n`)
})
