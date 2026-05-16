import * as fs from 'fs/promises';
import * as path from 'path';
import * as vscode from 'vscode';

type WasmExports = {
  memory: WebAssembly.Memory;
  alloc: (length: number) => number;
  dealloc: (pointer: number, length: number) => void;
  analyze_source: (pointer: number, length: number) => bigint | number;
};

export type AnalysisDiagnostic = {
  ruleId: string;
  message: string;
  severity: 'warning';
  line: number;
  column: number;
  length: number;
};

type AnalyzeResult = {
  diagnostics: AnalysisDiagnostic[];
};

let analyzerPromise: Promise<WasmExports> | undefined;

/**
 * Rust/WASM解析コアでCソース文字列を解析する。
 *
 * @param extensionUri 拡張機能のインストール位置。WASMファイルの探索に使う。
 * @param source 解析対象のCソースコード。
 * @returns WASM解析コアが返した診断結果。
 */
export async function analyzeSource(extensionUri: vscode.Uri, source: string): Promise<AnalyzeResult> {
  const analyzer = await loadAnalyzer(extensionUri);
  const encoder = new TextEncoder();
  const decoder = new TextDecoder();
  const input = encoder.encode(source);
  const inputPointer = analyzer.alloc(input.length);
  new Uint8Array(analyzer.memory.buffer, inputPointer, input.length).set(input);

  const packed = analyzer.analyze_source(inputPointer, input.length);
  analyzer.dealloc(inputPointer, input.length);

  const packedValue = typeof packed === 'bigint' ? packed : BigInt(packed);
  const outputPointer = Number(packedValue >> 32n);
  const outputLength = Number(packedValue & 0xffffffffn);
  const output = new Uint8Array(analyzer.memory.buffer, outputPointer, outputLength);
  const json = decoder.decode(output);
  analyzer.dealloc(outputPointer, outputLength);

  return JSON.parse(json) as AnalyzeResult;
}

/**
 * WASM解析コアの診断をVSCode Problems用の診断へ変換する。
 *
 * @param item WASM解析コアが返した1件の診断。
 * @returns VSCode APIへ渡す診断。
 */
export function toVscodeDiagnostic(item: AnalysisDiagnostic): vscode.Diagnostic {
  const start = new vscode.Position(Math.max(item.line - 1, 0), Math.max(item.column - 1, 0));
  const end = start.translate(0, Math.max(item.length, 1));
  const diagnostic = new vscode.Diagnostic(
    new vscode.Range(start, end),
    item.message,
    vscode.DiagnosticSeverity.Warning
  );
  diagnostic.code = item.ruleId;
  diagnostic.source = 'cert-c';
  return diagnostic;
}

/**
 * ワークスペースから見た相対パス表示を返す。
 *
 * @param uri 表示名を作る対象URI。
 * @returns ワークスペース相対パス。ワークスペース外の場合はファイル名。
 */
export function relativeLabel(uri: vscode.Uri): string {
  const workspaceFolder = vscode.workspace.getWorkspaceFolder(uri);
  if (!workspaceFolder) {
    return path.basename(uri.fsPath);
  }
  return path.relative(workspaceFolder.uri.fsPath, uri.fsPath);
}

/**
 * WASM解析コアを読み込む。読み込み済みの場合はキャッシュしたインスタンスを返す。
 *
 * @param extensionUri 拡張機能のインストール位置。WASMファイルの探索に使う。
 * @returns WASM export群。
 */
async function loadAnalyzer(extensionUri: vscode.Uri): Promise<WasmExports> {
  analyzerPromise ??= instantiateAnalyzer(extensionUri);
  return analyzerPromise;
}

/**
 * WASMファイルを読み込み、WebAssemblyインスタンスを生成する。
 *
 * @param extensionUri 拡張機能のインストール位置。WASMファイルの探索に使う。
 * @returns WASM export群。
 */
async function instantiateAnalyzer(extensionUri: vscode.Uri): Promise<WasmExports> {
  const wasmPath = vscode.Uri.joinPath(
    extensionUri,
    'analyzer-wasm',
    'target',
    'wasm32-unknown-unknown',
    'release',
    'analyzer_wasm.wasm'
  ).fsPath;
  const bytes = await fs.readFile(wasmPath);
  const module = await WebAssembly.instantiate(bytes, {});
  return module.instance.exports as WasmExports;
}

