// MavLang VS Code client — запускает `mavc lsp` (должен быть в PATH).
const { LanguageClient, TransportKind } = require('vscode-languageclient/node');

let client;

function activate(context) {
  const serverOptions = {
    command: 'mavc',
    args: ['lsp'],
    transport: TransportKind.stdio
  };
  const clientOptions = {
    documentSelector: [{ scheme: 'file', language: 'mavlang' }]
  };
  client = new LanguageClient('mavlang', 'MavLang LSP', serverOptions, clientOptions);
  client.start();
  context.subscriptions.push(client);
}

function deactivate() {
  return client ? client.stop() : undefined;
}

module.exports = { activate, deactivate };
