// Zamak for VS Code: starts `zam lsp` for diagnostics as you type, completion, hover,
// go to definition, and formatting. Syntax highlighting comes from the TextMate grammar
// and works even when `zam` isn't installed.
const vscode = require('vscode');
const { LanguageClient } = require('vscode-languageclient/node');

let client;

function serverLanguage() {
  // An explicit ZAMAK_LANG wins; otherwise follow the VS Code display language.
  if (process.env.ZAMAK_LANG) return process.env.ZAMAK_LANG;
  return vscode.env.language.toLowerCase().startsWith('zh') ? 'zh' : 'en';
}

async function start() {
  const command = vscode.workspace.getConfiguration('zamak').get('path') || 'zam';
  client = new LanguageClient(
    'zamak',
    'Zamak',
    { command, args: ['lsp'], options: { env: { ...process.env, ZAMAK_LANG: serverLanguage() } } },
    { documentSelector: [{ scheme: 'file', language: 'zamak' }] },
  );
  try {
    await client.start();
  } catch (error) {
    client = undefined;
    const choice = await vscode.window.showWarningMessage(
      `Zamak: couldn't start "${command} lsp". Install Zamak or set "zamak.path". Highlighting still works.`,
      'Open settings',
    );
    if (choice) vscode.commands.executeCommand('workbench.action.openSettings', 'zamak.path');
  }
}

async function stop() {
  if (client) {
    const running = client;
    client = undefined;
    await running.stop();
  }
}

function activate(context) {
  context.subscriptions.push(
    vscode.commands.registerCommand('zamak.restartServer', async () => {
      await stop();
      await start();
    }),
    vscode.workspace.onDidChangeConfiguration(async (event) => {
      if (event.affectsConfiguration('zamak.path')) {
        await stop();
        await start();
      }
    }),
  );
  return start();
}

function deactivate() {
  return stop();
}

module.exports = { activate, deactivate };
