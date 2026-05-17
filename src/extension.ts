import * as vscode from 'vscode';
import * as path from 'path';
import * as fs from 'fs';
import { AnalysisDiagnostic, analyzeSource, relativeLabel, toVscodeDiagnostic } from './analysis/wasmAnalyzer';
import { CertCMessageLanguage, localize, normalizeMessageLanguage } from './localization';

type CertCConfiguration = {
  fileGlobs: string[];
  excludeGlobs: string[];
  analyzeOnChange: boolean;
  analyzeOnSave: boolean;
  analyzeDebounceMs: number;
  messageLanguage: CertCMessageLanguage;
  standardLibraryDevelopment: StandardLibraryDevelopmentConfiguration;
};

type StandardLibraryDevelopmentConfiguration = {
  enabled: boolean;
  pathGlobs: string[];
};

type SharedCertCConfiguration = {
  standardLibraryDevelopment?: Partial<StandardLibraryDevelopmentConfiguration>;
};

const diagnosticCollectionName = 'cert-c';
const env33CEx1Comment = '/* cert-c: apply ENV33-C-EX1 */';
const pre31CEx1Comment = '/* cert-c: apply PRE31-C-EX1 */';
const sharedConfigurationRelativePath = '.vscode/cert-c_config.json';
const pendingAnalysis = new Map<string, NodeJS.Timeout>();
let diagnostics: vscode.DiagnosticCollection;

/**
 * VSCode拡張機能を有効化し、コマンドと自動解析イベントを登録する。
 *
 * @param context 拡張機能のライフサイクルとリソース管理に使うVSCodeコンテキスト。
 */
export function activate(context: vscode.ExtensionContext): void {
  diagnostics = vscode.languages.createDiagnosticCollection(diagnosticCollectionName);
  context.subscriptions.push(diagnostics);

  context.subscriptions.push(
    vscode.commands.registerCommand('cert-c.checkWorkspace', () => checkWorkspace(context.extensionUri)),
    vscode.commands.registerCommand('cert-c.clearDiagnostics', () => clearDiagnostics()),
    vscode.commands.registerCommand('cert-c.openSettings', () => openSettingsPanel(context)),
    vscode.commands.registerCommand('cert-c.applyStandardLibraryDevelopmentForFile', (uri: vscode.Uri) =>
      applyStandardLibraryDevelopmentForFile(context.extensionUri, uri)
    ),
    vscode.languages.registerCodeActionsProvider(
      [
        { language: 'c', scheme: 'file' },
        { language: 'cpp', scheme: 'file' }
      ],
      new CertCCodeActionProvider(),
      { providedCodeActionKinds: CertCCodeActionProvider.providedCodeActionKinds }
    ),
    vscode.workspace.onDidChangeTextDocument((event) => scheduleDocumentAnalysis(context.extensionUri, event.document)),
    vscode.workspace.onDidSaveTextDocument((document) => analyzeSavedDocument(context.extensionUri, document))
  );
}

/**
 * VSCode拡張機能の終了時に、予約済み解析と診断コレクションを破棄する。
 */
export function deactivate(): void {
  clearPendingAnalysis();
  diagnostics?.dispose();
}

/**
 * ワークスペース内の解析対象ファイルをすべて解析する。
 *
 * @param extensionUri 拡張機能のインストール位置。WASMファイルの探索に使う。
 * @returns 完了を表すPromise。
 */
async function checkWorkspace(extensionUri: vscode.Uri): Promise<void> {
  const folders = vscode.workspace.workspaceFolders;
  const config = getConfiguration();
  if (!folders || folders.length === 0) {
    void vscode.window.showWarningMessage(localize(config.messageLanguage, 'workspace.notOpen'));
    return;
  }

  clearDiagnostics();

  await vscode.window.withProgress(
    {
      location: vscode.ProgressLocation.Notification,
      title: localize(config.messageLanguage, 'workspace.progressTitle'),
      cancellable: true
    },
    async (progress, token) => {
      const files = await findCandidateFiles(config);
      if (files.length === 0) {
        void vscode.window.showInformationMessage(localize(config.messageLanguage, 'workspace.noFiles'));
        return;
      }

      const byFile = new Map<string, vscode.Diagnostic[]>();

      for (let index = 0; index < files.length; index += 1) {
        if (token.isCancellationRequested) {
          break;
        }

        const file = files[index];
        progress.report({
          message: `${relativeLabel(file)} (${index + 1}/${files.length})`,
          increment: 100 / files.length
        });

        const document = await vscode.workspace.openTextDocument(file);
        const items = await analyzeDocument(extensionUri, document);
        byFile.set(file.toString(), items);
      }

      void vscode.window.showInformationMessage(
        localize(config.messageLanguage, 'workspace.completed', { count: countDiagnostics(byFile) })
      );
    }
  );
}

/**
 * 編集中のドキュメントに対してdebounce付きの自動解析を予約する。
 *
 * @param extensionUri 拡張機能のインストール位置。WASMファイルの探索に使う。
 * @param document 変更されたVSCodeドキュメント。
 */
function scheduleDocumentAnalysis(extensionUri: vscode.Uri, document: vscode.TextDocument): void {
  const config = getConfiguration(document.uri);
  if (!config.analyzeOnChange || !isCandidateDocument(document, config)) {
    return;
  }

  const key = document.uri.toString();
  const existing = pendingAnalysis.get(key);
  if (existing) {
    clearTimeout(existing);
  }

  const timeout = setTimeout(() => {
    pendingAnalysis.delete(key);
    void analyzeDocument(extensionUri, document);
  }, config.analyzeDebounceMs);

  pendingAnalysis.set(key, timeout);
}

/**
 * 保存されたドキュメントを即時解析する。
 *
 * @param extensionUri 拡張機能のインストール位置。WASMファイルの探索に使う。
 * @param document 保存されたVSCodeドキュメント。
 */
function analyzeSavedDocument(extensionUri: vscode.Uri, document: vscode.TextDocument): void {
  const config = getConfiguration(document.uri);
  if (!config.analyzeOnSave || !isCandidateDocument(document, config)) {
    return;
  }

  const key = document.uri.toString();
  const existing = pendingAnalysis.get(key);
  if (existing) {
    clearTimeout(existing);
    pendingAnalysis.delete(key);
  }

  void analyzeDocument(extensionUri, document);
}

/**
 * 1つのVSCodeドキュメントをWASM解析コアで解析し、Problemsの診断を更新する。
 *
 * @param extensionUri 拡張機能のインストール位置。WASMファイルの探索に使う。
 * @param document 解析対象のVSCodeドキュメント。
 * @returns VSCodeへ登録した診断一覧。
 */
async function analyzeDocument(extensionUri: vscode.Uri, document: vscode.TextDocument): Promise<vscode.Diagnostic[]> {
  try {
    const config = getConfiguration(document.uri);
    const result = await analyzeSource(extensionUri, document.getText());
    const filteredDiagnostics = filterDiagnostics(result.diagnostics, document, config);
    const items = filteredDiagnostics.map((item) => toVscodeDiagnostic(item, config.messageLanguage));
    diagnostics.set(document.uri, items);
    return items;
  } catch (error) {
    const config = getConfiguration(document.uri);
    const message = error instanceof Error ? error.message : String(error);
    const diagnostic = new vscode.Diagnostic(
      new vscode.Range(new vscode.Position(0, 0), new vscode.Position(0, 1)),
      localize(config.messageLanguage, 'analysis.failed', { message }),
      vscode.DiagnosticSeverity.Error
    );
    diagnostic.source = diagnosticCollectionName;
    diagnostics.set(document.uri, [diagnostic]);
    return [diagnostic];
  }
}

/**
 * `certC`名前空間のVSCode設定を読み込む。
 *
 * @returns CERT-C拡張機能の実行設定。
 */
function getConfiguration(resourceUri?: vscode.Uri): CertCConfiguration {
  const config = vscode.workspace.getConfiguration('certC', resourceUri);
  const sharedConfig = readSharedConfiguration(resourceUri);
  return {
    fileGlobs: config.get('fileGlobs', ['**/*.c', '**/*.h']),
    excludeGlobs: config.get('excludeGlobs', [
      '**/.git/**',
      '**/.svn/**',
      '**/.hg/**',
      '**/build/**',
      '**/cmake-build-*/**',
      '**/Debug/**',
      '**/Release/**',
      '**/dist/**',
      '**/vendor/**',
      '**/third_party/**'
    ]),
    analyzeOnChange: config.get('analyzeOnChange', true),
    analyzeOnSave: config.get('analyzeOnSave', true),
    analyzeDebounceMs: Math.max(config.get('analyzeDebounceMs', 500), 0),
    messageLanguage: normalizeMessageLanguage(config.get('messageLanguage', 'en')),
    standardLibraryDevelopment: {
      enabled: sharedConfig.standardLibraryDevelopment?.enabled ?? false,
      pathGlobs: sharedConfig.standardLibraryDevelopment?.pathGlobs ?? []
    }
  };
}

/**
 * ワークスペース共有のCERT-C設定ファイルを読み込む。
 *
 * @param resourceUri 設定を適用する対象ファイルURI。
 * @returns 読み込めた共有設定。存在しない場合や不正な場合は空の設定。
 *
 * TODO: JSON構文エラーをProblemsやOutputへ通知する仕組みを追加する。
 */
function readSharedConfiguration(resourceUri?: vscode.Uri): SharedCertCConfiguration {
  const workspaceFolder = getWorkspaceFolder(resourceUri);
  if (!workspaceFolder) {
    return {};
  }

  const configPath = path.join(workspaceFolder.uri.fsPath, sharedConfigurationRelativePath);
  if (!fs.existsSync(configPath)) {
    return {};
  }

  try {
    const content = fs.readFileSync(configPath, 'utf8');
    const parsed = JSON.parse(content) as unknown;
    return isRecord(parsed) ? normalizeSharedConfiguration(parsed) : {};
  } catch {
    return {};
  }
}

/**
 * JSON由来の値を共有CERT-C設定として正規化する。
 *
 * @param value JSONから読み込んだオブジェクト。
 * @returns 型を確認した共有設定。
 */
function normalizeSharedConfiguration(value: Record<string, unknown>): SharedCertCConfiguration {
  const standardLibraryDevelopment = isRecord(value.standardLibraryDevelopment)
    ? {
        enabled:
          typeof value.standardLibraryDevelopment.enabled === 'boolean'
            ? value.standardLibraryDevelopment.enabled
            : undefined,
        pathGlobs: toStringArray(value.standardLibraryDevelopment.pathGlobs)
      }
    : undefined;
  return { standardLibraryDevelopment };
}

/**
 * ワークスペース設定に基づいて解析対象ファイルを探索する。
 *
 * @param config ファイル探索に使うCERT-C設定。
 * @returns 重複を除いた解析対象ファイルURI一覧。
 */
async function findCandidateFiles(config: CertCConfiguration): Promise<vscode.Uri[]> {
  const exclude = config.excludeGlobs.length > 0 ? `{${config.excludeGlobs.join(',')}}` : undefined;
  const seen = new Set<string>();
  const files: vscode.Uri[] = [];

  for (const glob of config.fileGlobs) {
    const matches = await vscode.workspace.findFiles(glob, exclude);
    for (const match of matches) {
      const key = match.toString();
      if (!seen.has(key)) {
        seen.add(key);
        files.push(match);
      }
    }
  }

  return files;
}

/**
 * ドキュメントが自動解析対象かを判定する。
 *
 * @param document 判定対象のVSCodeドキュメント。
 * @param config 除外設定を含むCERT-C設定。
 * @returns 自動解析対象の場合は`true`。
 */
function isCandidateDocument(document: vscode.TextDocument, config: CertCConfiguration): boolean {
  if (document.uri.scheme !== 'file') {
    return false;
  }

  const fileName = document.fileName.toLowerCase();
  if (!fileName.endsWith('.c') && !fileName.endsWith('.h')) {
    return false;
  }

  return !config.excludeGlobs.some((glob) => matchesSimpleExclude(document.uri.fsPath, glob));
}

/**
 * VSCode globのうち、現在利用している単純な除外パターンに一致するかを判定する。
 *
 * @param filePath 判定対象のファイルパス。
 * @param glob ディレクトリ名を含む除外glob。
 * @returns 除外対象として扱う場合は`true`。
 *
 * TODO: VSCodeのglob仕様に合わせた厳密なマッチングへ置き換える。
 */
function matchesSimpleExclude(filePath: string, glob: string): boolean {
  const normalized = filePath.replace(/\\/g, '/');
  const marker = glob.replace(/\\/g, '/').replace(/^\*\*\//, '').replace(/\/\*\*$/, '');
  return marker.length > 0 && normalized.includes(marker);
}

/**
 * 設定された例外規定を反映した診断一覧を返す。
 *
 * @param items WASM解析コアが返した診断一覧。
 * @param document 解析対象のVSCodeドキュメント。
 * @param config CERT-C拡張機能の実行設定。
 * @returns 例外規定を反映した診断一覧。
 */
function filterDiagnostics(
  items: AnalysisDiagnostic[],
  document: vscode.TextDocument,
  config: CertCConfiguration
): AnalysisDiagnostic[] {
  return filterStandardLibraryDevelopmentDiagnostics(items, document, config);
}

/**
 * 標準ライブラリ開発向け例外を適用した診断一覧を返す。
 *
 * @param items WASM解析コアが返した診断一覧。
 * @param document 解析対象のVSCodeドキュメント。
 * @param config CERT-C拡張機能の実行設定。
 * @returns 例外規定を反映した診断一覧。
 *
 * TODO: DCL37-C以外の標準ライブラリ開発向け例外も、ルール単位の方針が固まり次第追加する。
 */
function filterStandardLibraryDevelopmentDiagnostics(
  items: AnalysisDiagnostic[],
  document: vscode.TextDocument,
  config: CertCConfiguration
): AnalysisDiagnostic[] {
  if (!appliesStandardLibraryDevelopmentExceptions(document, config)) {
    return items;
  }

  return items.filter((item) => item.ruleId !== 'DCL37-C');
}

/**
 * 標準ライブラリ開発向け例外を現在のファイルへ適用するかを判定する。
 *
 * @param document 解析対象のVSCodeドキュメント。
 * @param config CERT-C拡張機能の実行設定。
 * @returns 例外規定の適用対象ファイルの場合は`true`。
 */
function appliesStandardLibraryDevelopmentExceptions(
  document: vscode.TextDocument,
  config: CertCConfiguration
): boolean {
  return (
    config.standardLibraryDevelopment.enabled &&
    config.standardLibraryDevelopment.pathGlobs.some((glob) => matchesPathGlob(document.uri, glob))
  );
}

/**
 * ファイルURIがワークスペース相対または絶対globパターンに一致するかを返す。
 *
 * @param uri 判定対象ファイルURI。
 * @param glob ワークスペース相対または絶対globパターン。
 * @returns globに一致する場合は`true`。
 */
function matchesPathGlob(uri: vscode.Uri, glob: string): boolean {
  const normalizedGlob = normalizePath(glob);
  if (normalizedGlob.length === 0) {
    return false;
  }

  const candidates = new Set<string>([normalizePath(uri.fsPath)]);
  const workspaceFolder = vscode.workspace.getWorkspaceFolder(uri);
  if (workspaceFolder) {
    candidates.add(normalizePath(path.relative(workspaceFolder.uri.fsPath, uri.fsPath)));
  }

  const pattern = globToRegExp(normalizedGlob);
  for (const candidate of candidates) {
    if (pattern.test(candidate)) {
      return true;
    }
  }

  return false;
}

/**
 * パス区切りをglob判定用に正規化する。
 *
 * @param value 正規化対象のパスまたはglob。
 * @returns `/`区切りへ正規化した文字列。
 */
function normalizePath(value: string): string {
  return value.trim().replace(/\\/g, '/');
}

/**
 * 最小限のglobパターンを正規表現へ変換する。
 *
 * @param glob 変換対象のglob。
 * @returns glob全体一致用の正規表現。
 *
 * TODO: VSCode glob仕様の全機能が必要になった場合は、共通glob実装へ置き換える。
 */
function globToRegExp(glob: string): RegExp {
  let source = '^';

  for (let index = 0; index < glob.length; index += 1) {
    const char = glob[index];
    const next = glob[index + 1];
    const afterNext = glob[index + 2];

    if (char === '*' && next === '*' && afterNext === '/') {
      source += '(?:.*/)?';
      index += 2;
    } else if (char === '*' && next === '*') {
      source += '.*';
      index += 1;
    } else if (char === '*') {
      source += '[^/]*';
    } else if (char === '?') {
      source += '[^/]';
    } else {
      source += escapeRegExp(char);
    }
  }

  return new RegExp(`${source}$`);
}

/**
 * 正規表現の特殊文字をエスケープする。
 *
 * @param value エスケープ対象の1文字。
 * @returns 正規表現リテラルとして扱える文字列。
 */
function escapeRegExp(value: string): string {
  return value.replace(/[\\^$.*+?()[\]{}|]/g, '\\$&');
}

/**
 * すべての診断と予約済み解析をクリアする。
 */
function clearDiagnostics(): void {
  clearPendingAnalysis();
  diagnostics.clear();
}

/**
 * debounce中の自動解析予約をすべて破棄する。
 */
function clearPendingAnalysis(): void {
  for (const timeout of pendingAnalysis.values()) {
    clearTimeout(timeout);
  }
  pendingAnalysis.clear();
}

/**
 * ファイル別診断マップに含まれる診断件数を合計する。
 *
 * @param byFile ファイルURI文字列をキーにした診断一覧マップ。
 * @returns 全ファイルの診断件数合計。
 */
function countDiagnostics(byFile: Map<string, vscode.Diagnostic[]>): number {
  let count = 0;
  for (const items of byFile.values()) {
    count += items.length;
  }
  return count;
}

/**
 * CERT-C共有設定を編集する独自設定画面を開く。
 *
 * @param context 拡張機能のライフサイクルとリソース管理に使うVSCodeコンテキスト。
 */
async function openSettingsPanel(context: vscode.ExtensionContext): Promise<void> {
  const config = getConfiguration();
  const workspaceFolder = getWorkspaceFolder();
  if (!workspaceFolder) {
    void vscode.window.showWarningMessage(localize(config.messageLanguage, 'sharedConfig.noWorkspace'));
    return;
  }

  const configUri = vscode.Uri.joinPath(workspaceFolder.uri, '.vscode', 'cert-c_config.json');
  const sharedConfig = await readSharedConfigurationFromUri(configUri);
  const panel = vscode.window.createWebviewPanel(
    'certCSettings',
    localize(config.messageLanguage, 'settings.title'),
    vscode.ViewColumn.One,
    {
      enableScripts: true,
      localResourceRoots: [context.extensionUri]
    }
  );

  panel.webview.html = renderSettingsHtml(panel.webview, config.messageLanguage, sharedConfig);
  panel.webview.onDidReceiveMessage((message: unknown) => {
    void handleSettingsWebviewMessage(message, configUri, config.messageLanguage);
  });
}

/**
 * 設定Webviewから受け取ったメッセージを処理する。
 *
 * @param message Webviewから送信されたメッセージ。
 * @param configUri 保存先の共有設定ファイルURI。
 * @param messageLanguage 表示に使うメッセージ言語。
 * @returns 完了を表すPromise。
 */
async function handleSettingsWebviewMessage(
  message: unknown,
  configUri: vscode.Uri,
  messageLanguage: CertCMessageLanguage
): Promise<void> {
  if (!isRecord(message) || message.command !== 'save' || !isRecord(message.value)) {
    return;
  }

  try {
    const sharedConfig = normalizeSharedConfiguration(message.value);
    await writeSharedConfiguration(configUri, sharedConfig);
    void vscode.window.showInformationMessage(localize(messageLanguage, 'settings.saved'));
  } catch (error) {
    const detail = error instanceof Error ? error.message : String(error);
    void vscode.window.showErrorMessage(localize(messageLanguage, 'sharedConfig.updateFailed', { message: detail }));
  }
}

/**
 * 共有設定編集画面のHTMLを生成する。
 *
 * @param webview HTMLを表示するVSCode Webview。
 * @param messageLanguage 表示に使うメッセージ言語。
 * @param config 表示する共有設定。
 * @returns Webviewへ設定するHTML。
 */
function renderSettingsHtml(
  webview: vscode.Webview,
  messageLanguage: CertCMessageLanguage,
  config: SharedCertCConfiguration
): string {
  const nonce = createNonce();
  const standardLibraryEnabled = config.standardLibraryDevelopment?.enabled ?? false;
  const standardLibraryPathGlobs = (config.standardLibraryDevelopment?.pathGlobs ?? []).join('\n');
  const title = localize(messageLanguage, 'settings.title');
  const saveLabel = messageLanguage === 'ja' ? '保存' : 'Save';
  const standardLibraryLabel =
    messageLanguage === 'ja'
      ? '標準ライブラリ開発向け例外を有効にする'
      : 'Enable standard-library-development exceptions';
  const standardLibraryPathsLabel =
    messageLanguage === 'ja'
      ? '標準ライブラリ開発向け例外の対象パス'
      : 'Paths for standard-library-development exceptions';

  return `<!DOCTYPE html>
<html lang="${messageLanguage}">
<head>
  <meta charset="UTF-8">
  <meta http-equiv="Content-Security-Policy" content="default-src 'none'; style-src ${webview.cspSource} 'unsafe-inline'; script-src 'nonce-${nonce}';">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>${escapeHtml(title)}</title>
  <style>
    body { font-family: var(--vscode-font-family); padding: 20px; color: var(--vscode-foreground); background: var(--vscode-editor-background); }
    h1 { font-size: 20px; font-weight: 600; margin: 0 0 20px; }
    section { margin-block: 18px; }
    label { display: block; margin-block-end: 8px; font-weight: 600; }
    textarea { box-sizing: border-box; width: 100%; min-height: 120px; padding: 8px; color: var(--vscode-input-foreground); background: var(--vscode-input-background); border: 1px solid var(--vscode-input-border); font-family: var(--vscode-editor-font-family); }
    .checkbox { display: flex; align-items: center; gap: 8px; font-weight: 400; }
    button { color: var(--vscode-button-foreground); background: var(--vscode-button-background); border: 0; padding: 6px 14px; cursor: pointer; }
    button:hover { background: var(--vscode-button-hoverBackground); }
  </style>
</head>
<body>
  <h1>${escapeHtml(title)}</h1>
  <section>
    <label class="checkbox">
      <input id="standardLibraryEnabled" type="checkbox" ${standardLibraryEnabled ? 'checked' : ''}>
      <span>${escapeHtml(standardLibraryLabel)}</span>
    </label>
  </section>
  <section>
    <label for="standardLibraryPathGlobs">${escapeHtml(standardLibraryPathsLabel)}</label>
    <textarea id="standardLibraryPathGlobs" spellcheck="false">${escapeHtml(standardLibraryPathGlobs)}</textarea>
  </section>
  <button id="save" type="button">${escapeHtml(saveLabel)}</button>
  <script nonce="${nonce}">
    const vscode = acquireVsCodeApi();
    const splitLines = (value) => value.split(/\\r?\\n/u).map((line) => line.trim()).filter((line) => line.length > 0);
    document.getElementById('save').addEventListener('click', () => {
      vscode.postMessage({
        command: 'save',
        value: {
          standardLibraryDevelopment: {
            enabled: document.getElementById('standardLibraryEnabled').checked,
            pathGlobs: splitLines(document.getElementById('standardLibraryPathGlobs').value)
          }
        }
      });
    });
  </script>
</body>
</html>`;
}

/**
 * CERT-C診断に対するクイックフィックスを提供する。
 */
class CertCCodeActionProvider implements vscode.CodeActionProvider {
  static readonly providedCodeActionKinds = [vscode.CodeActionKind.QuickFix];

  /**
   * 診断に対応するCode Action一覧を返す。
   *
   * @param document Code Actionを表示するVSCodeドキュメント。
   * @param range Code Action要求範囲。
   * @param context 対象範囲に含まれる診断情報。
   * @returns 適用可能なCode Action一覧。
   */
  provideCodeActions(
    document: vscode.TextDocument,
    range: vscode.Range,
    context: vscode.CodeActionContext
  ): vscode.CodeAction[] {
    void range;
    const actions: vscode.CodeAction[] = [];

    for (const diagnostic of context.diagnostics) {
      if (isEnv33CWarning(diagnostic) && !hasEnv33CEx1Comment(document, diagnostic.range.start.line)) {
        actions.push(createEnv33CEx1CodeAction(document, diagnostic));
      }

      if (isPre31CWarning(diagnostic) && !hasPre31CEx1Comment(document, diagnostic.range.start.line)) {
        actions.push(createPre31CEx1CodeAction(document, diagnostic));
      }

      if (isDcl37CWarning(diagnostic) && getWorkspaceFolder(document.uri)) {
        actions.push(createDcl37CEx3CodeAction(document, diagnostic));
      }
    }

    return actions;
  }
}

/**
 * DCL37-C-EX3用の標準ライブラリ開発対象パスを共有設定へ追加するCode Actionを作成する。
 *
 * @param document 例外適用対象のVSCodeドキュメント。
 * @param diagnostic 対象のDCL37-C診断。
 * @returns 共有設定更新用Code Action。
 */
function createDcl37CEx3CodeAction(document: vscode.TextDocument, diagnostic: vscode.Diagnostic): vscode.CodeAction {
  const config = getConfiguration(document.uri);
  const action = new vscode.CodeAction(
    localize(config.messageLanguage, 'codeAction.applyDcl37CEx3'),
    vscode.CodeActionKind.QuickFix
  );

  action.command = {
    command: 'cert-c.applyStandardLibraryDevelopmentForFile',
    title: action.title,
    arguments: [document.uri]
  };
  action.diagnostics = [diagnostic];
  return action;
}

/**
 * ENV33-C-EX1適用コメントを挿入するCode Actionを作成する。
 *
 * @param document コメント挿入対象のVSCodeドキュメント。
 * @param diagnostic 対象のENV33-C診断。
 * @returns コメント挿入用Code Action。
 */
function createEnv33CEx1CodeAction(document: vscode.TextDocument, diagnostic: vscode.Diagnostic): vscode.CodeAction {
  const config = getConfiguration(document.uri);
  return createLineCommentCodeAction(
    document,
    diagnostic,
    localize(config.messageLanguage, 'codeAction.applyEnv33CEx1'),
    env33CEx1Comment
  );
}

/**
 * PRE31-C-EX1適用コメントを挿入するCode Actionを作成する。
 *
 * @param document コメント挿入対象のVSCodeドキュメント。
 * @param diagnostic 対象のPRE31-C診断。
 * @returns コメント挿入用Code Action。
 */
function createPre31CEx1CodeAction(document: vscode.TextDocument, diagnostic: vscode.Diagnostic): vscode.CodeAction {
  const config = getConfiguration(document.uri);
  return createLineCommentCodeAction(
    document,
    diagnostic,
    localize(config.messageLanguage, 'codeAction.applyPre31CEx1'),
    pre31CEx1Comment
  );
}

/**
 * 診断行の直前に固定コメントを挿入するCode Actionを作成する。
 *
 * @param document コメント挿入対象のVSCodeドキュメント。
 * @param diagnostic 対象の診断。
 * @param title Code Actionの表示名。
 * @param comment 挿入する固定コメント。
 * @returns コメント挿入用Code Action。
 */
function createLineCommentCodeAction(
  document: vscode.TextDocument,
  diagnostic: vscode.Diagnostic,
  title: string,
  comment: string
): vscode.CodeAction {
  const action = new vscode.CodeAction(title, vscode.CodeActionKind.QuickFix);
  const edit = new vscode.WorkspaceEdit();
  const line = document.lineAt(diagnostic.range.start.line);
  const indent = line.text.match(/^\s*/u)?.[0] ?? '';

  edit.insert(document.uri, new vscode.Position(line.lineNumber, 0), `${indent}${comment}\n`);
  action.edit = edit;
  action.diagnostics = [diagnostic];
  action.isPreferred = true;
  return action;
}

/**
 * ENV33-Cの警告診断かを返す。
 *
 * @param diagnostic 判定対象のVSCode診断。
 * @returns ENV33-Cの警告診断の場合は`true`。
 */
function isEnv33CWarning(diagnostic: vscode.Diagnostic): boolean {
  return (
    diagnostic.source === diagnosticCollectionName &&
    diagnostic.code === 'ENV33-C' &&
    diagnostic.severity === vscode.DiagnosticSeverity.Warning
  );
}

/**
 * PRE31-Cの警告診断かを返す。
 *
 * @param diagnostic 判定対象のVSCode診断。
 * @returns PRE31-Cの警告診断の場合は`true`。
 */
function isPre31CWarning(diagnostic: vscode.Diagnostic): boolean {
  return (
    diagnostic.source === diagnosticCollectionName &&
    diagnostic.code === 'PRE31-C' &&
    diagnostic.severity === vscode.DiagnosticSeverity.Warning
  );
}

/**
 * DCL37-Cの警告診断かを返す。
 *
 * @param diagnostic 判定対象のVSCode診断。
 * @returns DCL37-Cの警告診断の場合は`true`。
 */
function isDcl37CWarning(diagnostic: vscode.Diagnostic): boolean {
  return (
    diagnostic.source === diagnosticCollectionName &&
    diagnostic.code === 'DCL37-C' &&
    diagnostic.severity === vscode.DiagnosticSeverity.Warning
  );
}

/**
 * 指定行または直前行にENV33-C-EX1適用コメントがあるかを返す。
 *
 * @param document 判定対象のVSCodeドキュメント。
 * @param lineIndex 診断位置の0始まり行番号。
 * @returns 適用コメントが存在する場合は`true`。
 */
function hasEnv33CEx1Comment(document: vscode.TextDocument, lineIndex: number): boolean {
  return lineHasFixedComment(document, lineIndex, env33CEx1Comment)
    || lineHasFixedComment(document, lineIndex - 1, env33CEx1Comment);
}

/**
 * 指定行または直前行にPRE31-C-EX1適用コメントがあるかを返す。
 *
 * @param document 判定対象のVSCodeドキュメント。
 * @param lineIndex 診断位置の0始まり行番号。
 * @returns 適用コメントが存在する場合は`true`。
 */
function hasPre31CEx1Comment(document: vscode.TextDocument, lineIndex: number): boolean {
  return lineHasFixedComment(document, lineIndex, pre31CEx1Comment)
    || lineHasFixedComment(document, lineIndex - 1, pre31CEx1Comment);
}

/**
 * 指定行に固定コメントがあるかを返す。
 *
 * @param document 判定対象のVSCodeドキュメント。
 * @param lineIndex 0始まり行番号。
 * @param comment 検索する固定コメント。
 * @returns 適用コメントが存在する場合は`true`。
 */
function lineHasFixedComment(document: vscode.TextDocument, lineIndex: number, comment: string): boolean {
  return lineIndex >= 0 && lineIndex < document.lineCount && document.lineAt(lineIndex).text.includes(comment);
}

/**
 * 現在のファイルを標準ライブラリ開発対象として共有設定ファイルへ追加する。
 *
 * @param uri 追加対象のファイルURI。
 * @returns 完了を表すPromise。
 */
async function applyStandardLibraryDevelopmentForFile(extensionUri: vscode.Uri, uri: vscode.Uri): Promise<void> {
  const config = getConfiguration(uri);
  const workspaceFolder = getWorkspaceFolder(uri);
  if (!workspaceFolder) {
    void vscode.window.showWarningMessage(localize(config.messageLanguage, 'sharedConfig.noWorkspace'));
    return;
  }

  try {
    const relativePath = normalizePath(path.relative(workspaceFolder.uri.fsPath, uri.fsPath));
    const configUri = vscode.Uri.joinPath(workspaceFolder.uri, '.vscode', 'cert-c_config.json');
    const sharedConfig = await readSharedConfigurationFromUri(configUri);
    const pathGlobs = sharedConfig.standardLibraryDevelopment?.pathGlobs ?? [];

    sharedConfig.standardLibraryDevelopment = {
      ...sharedConfig.standardLibraryDevelopment,
      enabled: true,
      pathGlobs: pathGlobs.includes(relativePath) ? pathGlobs : [...pathGlobs, relativePath]
    };

    await writeSharedConfiguration(configUri, sharedConfig);

    void vscode.window.showInformationMessage(
      localize(config.messageLanguage, 'sharedConfig.updated', { path: relativePath })
    );

    const document = vscode.workspace.textDocuments.find((item) => item.uri.toString() === uri.toString());
    if (document) {
      void analyzeDocument(extensionUri, document);
    }
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    void vscode.window.showErrorMessage(localize(config.messageLanguage, 'sharedConfig.updateFailed', { message }));
  }
}

/**
 * VSCode URIから共有CERT-C設定ファイルを読み込む。
 *
 * @param configUri 読み込み対象の共有設定ファイルURI。
 * @returns 読み込めた共有設定。存在しない場合や不正な場合は空の設定。
 */
async function readSharedConfigurationFromUri(configUri: vscode.Uri): Promise<SharedCertCConfiguration> {
  try {
    const bytes = await vscode.workspace.fs.readFile(configUri);
    const parsed = JSON.parse(new TextDecoder().decode(bytes)) as unknown;
    return isRecord(parsed) ? normalizeSharedConfiguration(parsed) : {};
  } catch {
    return {};
  }
}

/**
 * 共有CERT-C設定ファイルを書き込む。
 *
 * @param configUri 書き込み先の共有設定ファイルURI。
 * @param config 保存する共有設定。
 * @returns 完了を表すPromise。
 */
async function writeSharedConfiguration(configUri: vscode.Uri, config: SharedCertCConfiguration): Promise<void> {
  const directoryUri = vscode.Uri.file(path.dirname(configUri.fsPath));
  await vscode.workspace.fs.createDirectory(directoryUri);
  await vscode.workspace.fs.writeFile(configUri, new TextEncoder().encode(`${JSON.stringify(config, null, 2)}\r\n`));
}

/**
 * 対象URIを含むワークスペースフォルダを返す。
 *
 * @param resourceUri 判定対象URI。
 * @returns ワークスペースフォルダ。対象URIがない場合は先頭のワークスペースフォルダ。
 */
function getWorkspaceFolder(resourceUri?: vscode.Uri): vscode.WorkspaceFolder | undefined {
  if (resourceUri) {
    return vscode.workspace.getWorkspaceFolder(resourceUri);
  }

  return vscode.workspace.workspaceFolders?.[0];
}

/**
 * unknown値がJSONオブジェクトとして扱えるかを返す。
 *
 * @param value 判定対象の値。
 * @returns レコードとして扱える場合は`true`。
 */
function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null && !Array.isArray(value);
}

/**
 * unknown値を文字列配列へ正規化する。
 *
 * @param value 判定対象の値。
 * @returns 文字列のみを残した配列。配列でない場合は空配列。
 */
function toStringArray(value: unknown): string[] {
  return Array.isArray(value) ? value.filter((item): item is string => typeof item === 'string') : [];
}

/**
 * Webviewのscript nonceを生成する。
 *
 * @returns 英数字だけで構成したnonce。
 */
function createNonce(): string {
  const alphabet = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789';
  let nonce = '';
  for (let index = 0; index < 32; index += 1) {
    nonce += alphabet.charAt(Math.floor(Math.random() * alphabet.length));
  }
  return nonce;
}

/**
 * HTML本文へ埋め込む文字列をエスケープする。
 *
 * @param value エスケープ対象文字列。
 * @returns HTMLとして安全に埋め込める文字列。
 */
function escapeHtml(value: string): string {
  return value
    .replace(/&/gu, '&amp;')
    .replace(/</gu, '&lt;')
    .replace(/>/gu, '&gt;')
    .replace(/"/gu, '&quot;')
    .replace(/'/gu, '&#39;');
}
