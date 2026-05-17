import { promises as fs } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const wasmPath = path.join(root, 'analyzer-wasm', 'target', 'wasm32-unknown-unknown', 'release', 'analyzer_wasm.wasm');
const testdataRoot = path.join(root, 'testdata', 'cert-c', 'rules');

const bytes = await fs.readFile(wasmPath);
const module = await WebAssembly.instantiate(bytes, {});
const analyzer = module.instance.exports;
const encoder = new TextEncoder();
const decoder = new TextDecoder();

const files = await collectCFiles(testdataRoot);
let failures = 0;

for (const file of files) {
  const source = await fs.readFile(file, 'utf8');
  const result = analyze(source);
  const basename = path.basename(file);
  const expectedDiagnostics = expectedDiagnosticsFor(basename);
  const passed = expectedDiagnostics === 0 ? result.diagnostics.length === 0 : result.diagnostics.length > 0;

  if (!passed) {
    failures += 1;
    console.error(`NG ${path.relative(root, file)}: expected ${expectedDiagnostics > 0 ? 'diagnostic' : 'no diagnostic'}, got ${result.diagnostics.length}`);
    continue;
  }

  console.log(`OK ${path.relative(root, file)}: ${result.diagnostics.length} diagnostics`);
}

/**
 * fixtureファイル名から期待診断数を返す。
 *
 * @param {string} basename fixtureファイル名。
 * @returns {number} 違反例の場合は1、適合例の場合は0。
 */
function expectedDiagnosticsFor(basename) {
  if (/_ng\.[ch]$/u.test(basename)) {
    return 1;
  }
  if (/_ok\.[ch]$/u.test(basename)) {
    return 0;
  }

  throw new Error(`fixtureファイル名は {2桁数字}_{説明}_{ok|ng}.{c|h} 形式にしてください: ${basename}`);
}

if (failures > 0) {
  process.exitCode = 1;
}

/**
 * WASM解析コアでCソース文字列を解析する。
 *
 * @param {string} source 解析対象のCソースコード。
 * @returns {{ diagnostics: Array<object> }} WASM解析コアが返した診断結果。
 */
function analyze(source) {
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

  return JSON.parse(json);
}

/**
 * 指定ディレクトリ以下のCソースファイルを再帰的に収集する。
 *
 * @param {string} directory 探索を開始するディレクトリ。
 * @returns {Promise<string[]>} 見つかった`.c`または`.h`ファイルの絶対パス一覧。
 */
async function collectCFiles(directory) {
  const result = [];
  const entries = await fs.readdir(directory, { withFileTypes: true });

  for (const entry of entries) {
    const fullPath = path.join(directory, entry.name);
    if (entry.isDirectory()) {
      result.push(...await collectCFiles(fullPath));
    } else if (entry.isFile() && (entry.name.endsWith('.c') || entry.name.endsWith('.h'))) {
      result.push(fullPath);
    }
  }

  return result.sort();
}
