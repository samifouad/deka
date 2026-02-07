const vscode = require('vscode')
const { LanguageClient, TransportKind, State } = require('vscode-languageclient/node')
const fs = require('fs')
const path = require('path')
const os = require('os')

/** @type {LanguageClient | null} */
let client = null
/** @type {vscode.OutputChannel | null} */
let output = null

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

  // Common local install fallbacks when VS Code PATH differs from shell PATH.
  const home = os.homedir()
  const manualCandidates = process.platform === 'win32'
    ? []
    : [
        path.join(home, '.bun', 'bin', 'deka'),
        path.join(home, '.local', 'bin', 'phpx_lsp')
      ]
  for (const candidate of manualCandidates) {
    if (fs.existsSync(candidate)) {
      if (candidate.endsWith('/deka')) {
        return { command: candidate, args: ['lsp'] }
      }
      return { command: candidate, args }
    }
  }

  return {
    command: 'deka',
    args: ['lsp']
  }
}

async function startClient(context) {
  if (!output) {
    output = vscode.window.createOutputChannel('PHPX')
    context.subscriptions.push(output)
  }
  const server = resolveServerCommand(context)
  output.appendLine(`[phpx] starting language server: ${server.command} ${server.args.join(' ')}`)

  // Early check for absolute/bundled paths to surface obvious launch issues.
  if (path.isAbsolute(server.command) && !fs.existsSync(server.command)) {
    const msg = `PHPX LSP binary not found: ${server.command}`
    output.appendLine(`[phpx] error: ${msg}`)
    vscode.window.showErrorMessage(msg)
    return
  }

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
    outputChannel: output,
    traceOutputChannel: output,
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

  client.onDidChangeState((event) => {
    output.appendLine(`[phpx] client state changed: ${event.oldState} -> ${event.newState}`)
    if (event.newState === State.StartFailed) {
      vscode.window.showErrorMessage(
        'PHPX language server failed to start. Open Output -> PHPX for details.'
      )
    }
  })

  try {
    const started = client.start()
    if (started && typeof started.then === 'function') {
      await started
    } else if (started && typeof started.dispose === 'function') {
      context.subscriptions.push(started)
    }
    output.appendLine(`[phpx] language server startup complete; state=${client.state}`)
  } catch (err) {
    const msg = `Failed to start PHPX language server: ${String(err)}`
    output.appendLine(`[phpx] error: ${msg}`)
    vscode.window.showErrorMessage(msg)
  }
}

async function restartClient(context) {
  if (client) {
    try {
      // stop() throws when client is already in StartFailed; ignore and recreate.
      if (client.state === State.Running || client.state === State.Starting) {
        await client.stop()
      }
    } catch (err) {
      if (output) {
        output.appendLine(`[phpx] stop warning: ${String(err)}`)
      }
    }
    client = null
  }
  await startClient(context)
  if (client && client.state === State.Running) {
    vscode.window.showInformationMessage('PHPX language server restarted')
  } else {
    vscode.window.showWarningMessage(
      'PHPX language server did not start. Open Output -> PHPX for details.'
    )
  }
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
