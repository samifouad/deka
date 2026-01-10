/**
 * deka/router - Hono-style HTTP router
 *
 * Lightweight router with middleware support, inspired by Hono.
 */

/**
 * Router context - provides helper methods for building responses
 */
class Context {
  constructor(request, params = {}) {
    this.req = request
    this.params = params
    this._status = 200
    this._headers = {}
  }

  /**
   * Get request body as text
   */
  async text() {
    return this.req.body || ''
  }

  /**
   * Get request body as JSON
   */
  async json() {
    try {
      return JSON.parse(this.req.body || '{}')
    } catch {
      throw new Error('Invalid JSON body')
    }
  }

  /**
   * Get URL query parameter
   */
  query(key) {
    const url = new URL(this.req.url)
    return url.searchParams.get(key)
  }

  /**
   * Get route parameter (from /users/:id)
   */
  param(key) {
    return this.params[key]
  }

  /**
   * Get request header
   */
  header(key) {
    return this.req.headers[key] || this.req.headers[key.toLowerCase()]
  }

  /**
   * Set response status
   */
  status(code) {
    this._status = code
    return this
  }

  /**
   * Set response header
   */
  setHeader(key, value) {
    this._headers[key] = value
    return this
  }

  /**
   * Return JSON response
   */
  json(data, status) {
    return {
      status: status || this._status,
      headers: {
        'Content-Type': 'application/json',
        ...this._headers
      },
      body: JSON.stringify(data)
    }
  }

  /**
   * Return text response
   */
  text(data, status) {
    return {
      status: status || this._status,
      headers: {
        'Content-Type': 'text/plain',
        ...this._headers
      },
      body: String(data)
    }
  }

  /**
   * Return HTML response
   */
  html(data, status) {
    return {
      status: status || this._status,
      headers: {
        'Content-Type': 'text/html',
        ...this._headers
      },
      body: String(data)
    }
  }

  /**
   * Redirect to another URL
   */
  redirect(url, status = 302) {
    return {
      status,
      headers: {
        'Location': url,
        ...this._headers
      },
      body: ''
    }
  }

  /**
   * Return 404 Not Found
   */
  notFound() {
    return this.status(404).json({ error: 'Not Found' })
  }
}

/**
 * Route matcher - handles static and dynamic routes
 */
class RouteMatcher {
  constructor(pattern) {
    this.pattern = pattern
    this.keys = []

    // Convert /users/:id to regex
    const regexPattern = pattern
      .split('/')
      .map(segment => {
        if (segment.startsWith(':')) {
          this.keys.push(segment.slice(1))
          return '([^/]+)'
        }
        return segment
      })
      .join('\\/')

    this.regex = new RegExp(`^${regexPattern}$`)
  }

  match(path) {
    const matches = path.match(this.regex)
    if (!matches) return null

    const params = {}
    this.keys.forEach((key, i) => {
      params[key] = matches[i + 1]
    })

    return params
  }
}

/**
 * Router - main class
 *
 * The router is callable as a function (Hono-compatible):
 *   const app = new Router()
 *   export default app  // Works! No need for app.fetch
 */
class Router {
  constructor() {
    this.routes = []
    this.middlewares = []
    this.notFoundHandler = null
    this.errorHandler = null

    // Make the router instance callable as a function
    // This allows: export default app (instead of export default app.fetch)
    return new Proxy(this, {
      apply: (target, thisArg, args) => {
        return target.fetch(args[0])
      }
    })
  }

  /**
   * Add a route handler
   */
  on(method, path, ...handlers) {
    const matcher = new RouteMatcher(path)
    this.routes.push({ method, matcher, handlers })
    return this
  }

  /**
   * Convenience methods for common HTTP methods
   */
  get(path, ...handlers) {
    return this.on('GET', path, ...handlers)
  }

  post(path, ...handlers) {
    return this.on('POST', path, ...handlers)
  }

  put(path, ...handlers) {
    return this.on('PUT', path, ...handlers)
  }

  delete(path, ...handlers) {
    return this.on('DELETE', path, ...handlers)
  }

  patch(path, ...handlers) {
    return this.on('PATCH', path, ...handlers)
  }

  head(path, ...handlers) {
    return this.on('HEAD', path, ...handlers)
  }

  options(path, ...handlers) {
    return this.on('OPTIONS', path, ...handlers)
  }

  /**
   * Add middleware that runs on all routes
   */
  use(middleware) {
    this.middlewares.push(middleware)
    return this
  }

  /**
   * Set custom 404 handler (Hono-compatible)
   */
  notFound(handler) {
    this.notFoundHandler = handler
    return this
  }

  /**
   * Set custom error handler (Hono-compatible)
   */
  onError(handler) {
    this.errorHandler = handler
    return this
  }

  /**
   * Main request handler (compatible with deka-runtime)
   */
  async fetch(request) {
    const url = new URL(request.url)
    const path = url.pathname
    const method = request.method

    // Find matching route
    for (const route of this.routes) {
      if (route.method !== method) continue

      const params = route.matcher.match(path)
      if (!params) continue

      // Create context
      const ctx = new Context(request, params)

      try {
        // Run middlewares
        for (const middleware of this.middlewares) {
          const result = await middleware(ctx)
          if (result) return result // Middleware returned early
        }

        // Run route handlers
        for (const handler of route.handlers) {
          const result = await handler(ctx)
          if (result) return result
        }

        // Handler didn't return anything
        return ctx.status(500).json({ error: 'Handler did not return a response' })
      } catch (error) {
        // Custom error handler or default
        if (this.errorHandler) {
          return await this.errorHandler(error, ctx)
        }

        console.error('Route handler error:', error)
        return ctx.status(500).json({
          error: 'Internal Server Error',
          message: error.message
        })
      }
    }

    // No route matched - use custom 404 handler or default
    const ctx = new Context(request)
    if (this.notFoundHandler) {
      return await this.notFoundHandler(ctx)
    }

    return {
      status: 404,
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ error: 'Not Found' })
    }
  }
}

/**
 * CORS middleware factory
 */
function cors(options = {}) {
  const {
    origin = '*',
    methods = ['GET', 'POST', 'PUT', 'DELETE', 'PATCH', 'HEAD', 'OPTIONS'],
    headers = ['Content-Type', 'Authorization'],
    credentials = false,
    maxAge = 86400
  } = options

  return async (ctx) => {
    // Set CORS headers
    ctx.setHeader('Access-Control-Allow-Origin', origin)
    ctx.setHeader('Access-Control-Allow-Methods', methods.join(', '))
    ctx.setHeader('Access-Control-Allow-Headers', headers.join(', '))

    if (credentials) {
      ctx.setHeader('Access-Control-Allow-Credentials', 'true')
    }

    // Handle preflight
    if (ctx.req.method === 'OPTIONS') {
      ctx.setHeader('Access-Control-Max-Age', String(maxAge))
      return ctx.status(204).text('')
    }

    // Continue to next handler
    return null
  }
}

/**
 * Logger middleware
 */
function logger() {
  return async (ctx) => {
    const start = Date.now()
    const { method, url } = ctx.req

    console.log(`-> ${method} ${url}`)

    // Continue to handler (would need response to log properly, simplified here)
    return null
  }
}

/**
 * Basic auth middleware
 */
function basicAuth(options = {}) {
  const { username, password, realm = 'Secure Area' } = options

  return async (ctx) => {
    const auth = ctx.header('Authorization')

    if (!auth || !auth.startsWith('Basic ')) {
      ctx.setHeader('WWW-Authenticate', `Basic realm="${realm}"`)
      return ctx.status(401).json({ error: 'Unauthorized' })
    }

    try {
      const credentials = atob(auth.slice(6))
      const [user, pass] = credentials.split(':')

      if (user === username && pass === password) {
        return null // Authorized, continue
      }
    } catch {
      // Invalid base64
    }

    return ctx.status(401).json({ error: 'Invalid credentials' })
  }
}

/**
 * Bearer token middleware
 */
function bearerAuth(options = {}) {
  const { token: validToken, verify } = options

  return async (ctx) => {
    const auth = ctx.header('Authorization')

    if (!auth || !auth.startsWith('Bearer ')) {
      return ctx.status(401).json({ error: 'Missing bearer token' })
    }

    const token = auth.slice(7)

    // Custom verification function
    if (verify) {
      const valid = await verify(token)
      if (!valid) {
        return ctx.status(401).json({ error: 'Invalid token' })
      }
      return null
    }

    // Simple token comparison
    if (validToken && token === validToken) {
      return null
    }

    return ctx.status(401).json({ error: 'Invalid token' })
  }
}

/**
 * Rate limiting middleware (simple in-memory)
 */
function rateLimit(options = {}) {
  const {
    max = 100,
    window = 60000, // 1 minute
    keyGenerator = (ctx) => ctx.header('X-Forwarded-For') || 'anonymous'
  } = options

  const requests = new Map()

  return async (ctx) => {
    const key = keyGenerator(ctx)
    const now = Date.now()

    // Clean old entries
    for (const [k, data] of requests.entries()) {
      if (now - data.resetTime > window) {
        requests.delete(k)
      }
    }

    // Get or create counter
    let data = requests.get(key)
    if (!data || now - data.resetTime > window) {
      data = { count: 0, resetTime: now }
      requests.set(key, data)
    }

    data.count++

    // Check limit
    if (data.count > max) {
      ctx.setHeader('X-RateLimit-Limit', String(max))
      ctx.setHeader('X-RateLimit-Remaining', '0')
      ctx.setHeader('X-RateLimit-Reset', String(data.resetTime + window))
      return ctx.status(429).json({ error: 'Too Many Requests' })
    }

    // Add rate limit headers
    ctx.setHeader('X-RateLimit-Limit', String(max))
    ctx.setHeader('X-RateLimit-Remaining', String(max - data.count))
    ctx.setHeader('X-RateLimit-Reset', String(data.resetTime + window))

    return null
  }
}

/**
 * Pretty JSON middleware (Hono-compatible)
 * Formats JSON responses with indentation
 */
function prettyJSON(options = {}) {
  const { space = 2 } = options

  return async (ctx) => {
    // Store original json method
    const originalJson = ctx.json.bind(ctx)

    // Override json method to pretty-print
    ctx.json = function(data, status) {
      return {
        status: status || ctx._status,
        headers: {
          'Content-Type': 'application/json',
          ...ctx._headers
        },
        body: JSON.stringify(data, null, space)
      }
    }

    return null
  }
}

// Export everything
export { Router, Context, cors, logger, basicAuth, bearerAuth, rateLimit, prettyJSON }

// Also expose as global for handler code (can't use import from regular JS)
globalThis.__dekaRouter = { Router, Context, cors, logger, basicAuth, bearerAuth, rateLimit, prettyJSON }
