const vscode = require('vscode')
const { LanguageClient, TransportKind } = require('vscode-languageclient/node')

/** @type {LanguageClient | null} */
let client = null

function resolveServerCommand() {
  const cfg = vscode.workspace.getConfiguration('phpx')
  const explicit = cfg.get('lsp.path', '').trim()
  const explicitArgs = cfg.get('lsp.args', [])

  if (explicit) {
    return {
      command: explicit,
      args: Array.isArray(explicitArgs) ? explicitArgs : []
    }
  }

  return {
    command: 'deka',
    args: ['lsp']
  }
}

async function startClient(context) {
  const server = resolveServerCommand()

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
