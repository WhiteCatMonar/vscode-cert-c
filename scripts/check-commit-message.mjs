import { promises as fs } from 'node:fs';

const allowedTypes = new Set([
  'feature',
  'fix',
  'documentation',
  'test',
  'refactor',
  'optimize',
  'change',
  'maintenance',
  'release'
]);

const messagePath = process.argv[2];

if (!messagePath) {
  console.error('コミットメッセージファイルのパスを指定してください。');
  process.exit(2);
}

const message = await fs.readFile(messagePath, 'utf8');
const firstLine = findFirstMessageLine(message);
const result = validateSubject(firstLine);

if (!result.valid) {
  console.error(result.message);
  console.error('');
  console.error('コミットメッセージの1行目は以下の形式にしてください。');
  console.error('');
  console.error('[type]: summary');
  console.error('');
  console.error(`type: ${Array.from(allowedTypes).map((type) => `[${type}]`).join(', ')}`);
  process.exit(1);
}

/**
 * Gitのコメント行を除外し、最初のコミットメッセージ行を返す。
 *
 * @param {string} message Gitが渡すコミットメッセージ全体。
 * @returns {string} 最初の非空・非コメント行。見つからない場合は空文字列。
 */
function findFirstMessageLine(message) {
  return message
    .split(/\r?\n/)
    .map((line) => line.trim())
    .find((line) => line.length > 0 && !line.startsWith('#')) ?? '';
}

/**
 * コミットメッセージの1行目を検証する。
 *
 * @param {string} subject 検証対象の1行目。
 * @returns {{ valid: boolean, message: string }} 検証結果とエラーメッセージ。
 */
function validateSubject(subject) {
  if (subject.length === 0) {
    return {
      valid: false,
      message: 'コミットメッセージが空です。'
    };
  }

  const match = /^\[([a-z]+)\]:\s+(.+)$/.exec(subject);
  if (!match) {
    return {
      valid: false,
      message: `コミットメッセージ形式が不正です: ${subject}`
    };
  }

  const [, type, summary] = match;
  if (!allowedTypes.has(type)) {
    return {
      valid: false,
      message: `未定義のtypeです: [${type}]`
    };
  }

  if (summary.trim().length === 0) {
    return {
      valid: false,
      message: 'summaryを記載してください。'
    };
  }

  if (summary.trim() === '<summary>') {
    return {
      valid: false,
      message: 'summaryのテンプレート文字列を実際の説明に置き換えてください。'
    };
  }

  return {
    valid: true,
    message: ''
  };
}
