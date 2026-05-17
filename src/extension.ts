import * as vscode from 'vscode';
import { analyzeSource, relativeLabel, toVscodeDiagnostic } from './analysis/wasmAnalyzer';
import { CertCMessageLanguage, localize, normalizeMessageLanguage } from './localization';

type CertCConfiguration = {
  fileGlobs: string[];
  excludeGlobs: string[];
  analyzeOnChange: boolean;
  analyzeOnSave: boolean;
  analyzeDebounceMs: number;
  messageLanguage: CertCMessageLanguage;
};

const diagnosticCollectionName = 'cert-c';
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
  const config = getConfiguration();
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
  const config = getConfiguration();
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
    const config = getConfiguration();
    const result = await analyzeSource(extensionUri, document.getText());
    const items = result.diagnostics.map((item) => toVscodeDiagnostic(item, config.messageLanguage));
    diagnostics.set(document.uri, items);
    return items;
  } catch (error) {
    const config = getConfiguration();
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
function getConfiguration(): CertCConfiguration {
  const config = vscode.workspace.getConfiguration('certC');
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
    messageLanguage: normalizeMessageLanguage(config.get('messageLanguage', 'en'))
  };
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
