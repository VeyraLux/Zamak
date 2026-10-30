// Zamak for VS Code: starts `zam lsp` for diagnostics as you type, completion, hover,
// go to definition, and formatting. Syntax highlighting comes from the TextMate grammar
// and works even when `zam` isn't installed.
const vscode = require('vscode');
const { exec, execFile } = require('child_process');
const { LanguageClient } = require('vscode-languageclient/node');
const { locate } = require('./locate');

let client;
let status;
// 找到的 zam：{ command: 原生程序的完整路径, from: 在哪里找到的 }；没找到是 undefined。
let found;
// 启动时问一次 `zam --version`；装好之后重新问。
let version;
// 同一次会话里只提示一次“没装 zam”，之后靠状态栏和命令重试。
let warnedMissing = false;
// 一键安装之后，每隔几秒找一次 zam，找到就自动启动语言服务器。
let watching;

function serverLanguage() {
  // An explicit ZAMAK_LANG wins; otherwise follow the VS Code display language.
  if (process.env.ZAMAK_LANG) return process.env.ZAMAK_LANG;
  return vscode.env.language.toLowerCase().startsWith('zh') ? 'zh' : 'en';
}

function chinese() {
  return vscode.env.language.toLowerCase().startsWith('zh');
}

// 界面文字跟随 VS Code 的显示语言（与报错语言一致）。
function t(en, zh) {
  return chinese() ? zh : en;
}

function configuredPath() {
  return vscode.workspace.getConfiguration('zamak').get('path') || undefined;
}

// `npm root -g`：刚用 npm 装好时，VS Code 的 PATH 还是旧的，要问 npm 装到了哪里。
// npm 在 Windows 上也是 .cmd，所以用 exec 经过 shell 启动。
function npmRoot() {
  return new Promise((resolve) => {
    exec('npm root -g', { timeout: 15000 }, (error, stdout) => {
      resolve(error ? undefined : stdout.toString().trim() || undefined);
    });
  });
}

// 找 zam：先看设置和 PATH，找不到再问 npm。
async function find() {
  const configured = configuredPath();
  found = locate({ configured });
  if (!found && !configured) found = locate({ npmRoot: await npmRoot() });
  return found;
}

// 问一次 `zam --version`；没找到或跑不起来时返回 undefined。
function askVersion() {
  if (!found) return Promise.resolve(undefined);
  return new Promise((resolve) => {
    execFile(found.command, ['--version'], { timeout: 10000 }, (error, stdout) => {
      resolve(error ? undefined : stdout.toString().trim());
    });
  });
}

async function refresh() {
  await find();
  version = await askVersion();
  updateStatus();
  return version;
}

const INSTALL = ['zamak.install', '用 npm 安装', 'Install with npm'];
const INSTALLED = ['zamak.recheck', '已经装好了', 'I installed it'];
const SETTINGS = ['', '打开设置', 'Open settings'];

// 状态栏：装好了显示版本，没装显示一个可点的安装入口。
function updateStatus() {
  if (!status) return;
  if (version) {
    status.text = `$(check) zam ${version}`;
    status.tooltip = `Zamak ${version}\n${found.command}`;
    status.command = 'zamak.restartServer';
    status.backgroundColor = undefined;
  } else {
    status.text = '$(warning) zam';
    status.tooltip = t(
      'Zamak is not installed. Click to install it with npm.',
      '没有找到 zam。点击用 npm 安装。',
    );
    status.command = INSTALL[0];
    status.backgroundColor = new vscode.ThemeColor('statusBarItem.warningBackground');
  }
  status.show();
}

function stopWatching() {
  if (watching) clearInterval(watching);
  watching = undefined;
}

// 一键安装：在终端里跑 `npm install -g @veyralux/zamak`。拿不到终端里命令的退出码，所以每 3 秒
// 找一次 zam，最多找 5 分钟；找到就启动语言服务器，用户不用再点什么。
function install() {
  const terminal = vscode.window.createTerminal(t('Install Zamak', '安装 Zamak'));
  terminal.show();
  terminal.sendText('npm install -g @veyralux/zamak', true);
  stopWatching();
  const deadline = Date.now() + 5 * 60 * 1000;
  watching = setInterval(async () => {
    if (Date.now() > deadline) return stopWatching();
    if (!(await refresh())) return;
    stopWatching();
    await start({ quiet: true });
    vscode.window.showInformationMessage(
      t(`Zamak ${version} is ready.`, `Zamak ${version} 装好了，语言服务器已启动。`),
    );
  }, 3000);
}

// 重新找 zam；找到了就把语言服务器拉起来（之前因为没装而没启动）。
async function recheck() {
  if (!(await refresh())) {
    const choice = await vscode.window.showWarningMessage(
      t(
        'Zamak: still can\'t find zam. If "zam --version" works in a terminal, run "where zam" (Windows) or "which zam" (macOS and Linux) there, and put the path it prints in the "zamak.path" setting.',
        'Zamak：还是找不到 zam。如果终端里 zam --version 能用，请在终端运行 where zam（Windows）或 which zam（macOS、Linux），把显示的路径填进设置 "zamak.path"。',
      ),
      t(SETTINGS[2], SETTINGS[1]),
    );
    if (choice) vscode.commands.executeCommand('workbench.action.openSettings', 'zamak.path');
    return;
  }
  stopWatching();
  await start({ quiet: true });
}

async function start(options = {}) {
  if (client) return;
  if (!found) {
    if (!options.quiet) await missing();
    return;
  }
  client = new LanguageClient(
    'zamak',
    'Zamak',
    {
      command: found.command,
      args: ['lsp'],
      options: { env: { ...process.env, ZAMAK_LANG: serverLanguage() } },
    },
    { documentSelector: [{ scheme: 'file', language: 'zamak' }] },
  );
  try {
    await client.start();
    warnedMissing = false;
  } catch (error) {
    client = undefined;
    if (options.quiet) return;
    vscode.window.showWarningMessage(
      t(
        `Zamak: couldn't start "${found.command} lsp": ${error.message}. Highlighting still works.`,
        `Zamak：无法启动“${found.command} lsp”：${error.message}。语法高亮照常可用。`,
      ),
    );
  }
}

// 没装 zam：给一次一键安装和“已经装好了”，之后靠状态栏重试。
async function missing() {
  if (warnedMissing) return;
  warnedMissing = true;
  const choice = await vscode.window.showInformationMessage(
    t(
      'Zamak: the zam compiler is not installed. Install it with npm to get diagnostics, completion, and formatting.',
      'Zamak：没有找到 zam 编译器。用 npm 安装后才能有报错提示、补全和格式化。',
    ),
    t(INSTALL[2], INSTALL[1]),
    t(INSTALLED[2], INSTALLED[1]),
  );
  if (choice === t(INSTALL[2], INSTALL[1])) install();
  else if (choice) vscode.commands.executeCommand(INSTALLED[0]);
}

async function stop() {
  if (client) {
    const running = client;
    client = undefined;
    await running.stop();
  }
}

function activate(context) {
  status = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Right, 100);
  context.subscriptions.push(
    status,
    { dispose: stopWatching },
    vscode.commands.registerCommand('zamak.restartServer', async () => {
      await stop();
      await refresh();
      await start();
    }),
    vscode.commands.registerCommand(INSTALL[0], install),
    vscode.commands.registerCommand(INSTALLED[0], recheck),
    vscode.workspace.onDidChangeConfiguration(async (event) => {
      if (event.affectsConfiguration('zamak.path')) {
        await stop();
        await refresh();
        await start({ quiet: true });
      }
    }),
  );
  return (async () => {
    await refresh();
    // 没找到 zam 时不去启动，只留状态栏上的安装入口；语法高亮照常可用。
    await start();
  })();
}

function deactivate() {
  stopWatching();
  return stop();
}

module.exports = { activate, deactivate };
