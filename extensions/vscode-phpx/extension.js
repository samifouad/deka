const vscode = require('vscode')
const { LanguageClient, TransportKind } = require('vscode-languageclient/node')
const fs = require('fs')
const path = require('path')

/** @type {LanguageClient | null} */
let client = null

function findInPath(bin) {
  const envPath = process.env.PATH || ''
  const sep = process.platform === 'win32' ? ';' : ':'
  const parts = envPath.split(sep).filter(Boolean)
  for (const part of parts) {
    const candidate = path.join(part, bin)
    if (fs.existsSync(candidate)) {
      return candidate
    }
  }
  return null
}

function resolveServerCommand(context) {
  const cfg = vscode.workspace.getConfiguration('phpx')
  const explicit = cfg.get('lsp.path', '').trim()
  const explicitArgs = cfg.get('lsp.args', [])
  const args = Array.isArray(explicitArgs) ? explicitArgs : []

  if (explicit) {
    return {
      command: explicit,
      args
    }
  }

  const exe = process.platform === 'win32' ? 'phpx_lsp.exe' : 'phpx_lsp'
  const bundledCandidates = [
    path.join(context.extensionPath, 'bin', exe),
    path.join(context.extensionPath, 'bin', `${process.platform}-${process.arch}`, exe)
  ]
  for (const candidate of bundledCandidates) {
    if (fs.existsSync(candidate)) {
      return {
        command: candidate,
        args
      }
    }
  }

  const dekaBin = findInPath(process.platform === 'win32' ? 'deka.exe' : 'deka')
  if (dekaBin) {
    return {
      command: dekaBin,
      args: ['lsp']
    }
  }

  const lspBin = findInPath(process.platform === 'win32' ? 'phpx_lsp.exe' : 'phpx_lsp')
  if (lspBin) {
    return {
      command: lspBin,
      args
    }
  }

  return {
    command: 'deka',
    args: ['lsp']
  }
}

async function startClient(context) {
  const server = resolveServerCommand(context)

  const serverOptions = {
    run: {
      command: server.command,
      args: server.args,
      transport: TransportKind.stdio
    },
    debug: {
      command: server.command,
      args: server.args,
      transport: TransportKind.stdio
    }
  }

  const clientOptions = {
    documentSelector: [{ scheme: 'file', language: 'phpx' }],
    synchronize: {
      fileEvents: vscode.workspace.createFileSystemWatcher('**/*.phpx')
    }
  }

  client = new LanguageClient(
    'phpx-lsp',
    'PHPX Language Server',
    serverOptions,
    clientOptions
  )

  context.subscriptions.push(client.start())
}

async function restartClient(context) {
  if (client) {
    await client.stop()
    client = null
  }
  await startClient(context)
  vscode.window.showInformationMessage('PHPX language server restarted')
}

async function activate(context) {
  const restartCmd = vscode.commands.registerCommand('phpx.restartLanguageServer', async () => {
    await restartClient(context)
  })

  context.subscriptions.push(restartCmd)
  await startClient(context)
}

async function deactivate() {
  if (client) {
    await client.stop()
    client = null
  }
}

module.exports = {
  activate,
  deactivate
}
