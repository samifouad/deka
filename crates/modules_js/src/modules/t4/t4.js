/**
 * deka/t4 - T4 storage client
 *
 * HTTP client for T4 object storage with S3-compatible API.
 *
 * Example:
 *   import { t4 } from 'deka/t4'
 *
 *   // Upload
 *   await t4.file('test.json').write(JSON.stringify({ hello: 'world' }))
 *
 *   // Download
 *   const data = await t4.file('test.json').json()
 */

// T4File class - lazy reference to T4 object
class T4File {
  constructor(client, key) {
    this.client = client
    this.key = key
  }

  async text() {
    return await Deno.core.ops.op_t4_get_text(this.client.id, this.key)
  }

  async json() {
    const text = await this.text()
    return JSON.parse(text)
  }

  async arrayBuffer() {
    const buffer = await Deno.core.ops.op_t4_get_buffer(this.client.id, this.key)
    return buffer.buffer
  }

  async write(data, options = {}) {
    let bytes
    let contentType = options.type || 'application/octet-stream'

    if (typeof data === 'string') {
      bytes = new TextEncoder().encode(data)
    } else if (data instanceof ArrayBuffer) {
      bytes = new Uint8Array(data)
    } else if (data instanceof Uint8Array) {
      bytes = data
    } else if (data && typeof data.arrayBuffer === 'function') {
      // Response or Blob
      const ab = await data.arrayBuffer()
      bytes = new Uint8Array(ab)
    } else {
      throw new Error('Unsupported data type for write()')
    }

    return await Deno.core.ops.op_t4_put(
      this.client.id,
      this.key,
      bytes,
      contentType
    )
  }

  async delete() {
    return await Deno.core.ops.op_t4_delete(this.client.id, this.key)
  }

  async exists() {
    return await Deno.core.ops.op_t4_exists(this.client.id, this.key)
  }

  async stat() {
    const result = await Deno.core.ops.op_t4_stat(this.client.id, this.key)
    // Convert lastModified string to Date
    if (result.lastModified) {
      result.lastModified = new Date(result.lastModified)
    }
    return result
  }
}

// T4Client class
class T4Client {
  constructor(config = {}) {
    // Create client ID synchronously via promise (will be resolved immediately)
    this.id = null
    this._initPromise = Deno.core.ops.op_t4_create_client(config).then(id => {
      this.id = id
      return id
    })
  }

  // Ensure client is initialized before use
  async _ensureInit() {
    if (this.id === null) {
      await this._initPromise
    }
  }

  file(key) {
    // Return file reference immediately (lazy loading)
    return new T4File(this, key)
  }
}

// Global singleton (lazy initialization - only created when accessed)
let _t4Instance = null

const t4 = {
  get _client() {
    if (_t4Instance === null) {
      _t4Instance = new T4Client()
    }
    return _t4Instance
  },

  file(key) {
    return this._client.file(key)
  }
}

// Helper function
async function write(file, data, options = {}) {
  return await file.write(data, options)
}

export { t4, T4Client, T4File, write }

// Also expose as global for handler code
globalThis.__dekaT4 = { t4, T4Client, T4File, write }
