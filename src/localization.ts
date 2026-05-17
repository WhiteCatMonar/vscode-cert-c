import { localizeRuleDiagnosticMessage } from './rules/diagnosticMessages';

export type CertCMessageLanguage = 'en' | 'ja';

type MessageKey =
  | 'workspace.notOpen'
  | 'workspace.progressTitle'
  | 'workspace.noFiles'
  | 'workspace.completed'
  | 'analysis.failed';

const messages: Record<CertCMessageLanguage, Record<MessageKey, string>> = {
  en: {
    'workspace.notOpen': 'Open a workspace before running CERT-C checks.',
    'workspace.progressTitle': 'Running CERT-C checks',
    'workspace.noFiles': 'No C source or header files were found for CERT-C checks.',
    'workspace.completed': 'CERT-C check completed: {count} diagnostic(s).',
    'analysis.failed': 'CERT-C analysis failed: {message}'
  },
  ja: {
    'workspace.notOpen': 'ワークスペースを開いてからCERT-Cチェックを実行してください。',
    'workspace.progressTitle': 'CERT-Cチェックを実行中',
    'workspace.noFiles': 'CERT-Cチェック対象のCソースまたはヘッダファイルが見つかりませんでした。',
    'workspace.completed': 'CERT-Cチェック完了: {count}件の診断。',
    'analysis.failed': 'CERT-C解析に失敗しました: {message}'
  }
};

/**
 * 設定値を表示言語として扱える値に正規化する。
 *
 * @param value VSCode設定から読み込んだメッセージ言語設定値。
 * @returns 対応済みのメッセージ言語。未対応値の場合は英語。
 */
export function normalizeMessageLanguage(value: string): CertCMessageLanguage {
  return value === 'ja' ? 'ja' : 'en';
}

/**
 * 設定言語に応じたメッセージを返す。
 *
 * @param messageLanguage 表示に使うメッセージ言語。
 * @param key メッセージ辞書のキー。
 * @param values メッセージ内のプレースホルダーへ埋め込む値。
 * @returns ローカライズ済みメッセージ。
 */
export function localize(
  messageLanguage: CertCMessageLanguage,
  key: MessageKey,
  values: Record<string, string | number> = {}
): string {
  const template = messages[messageLanguage][key] ?? messages.en[key];
  return Object.entries(values).reduce(
    (text, [name, value]) => text.replaceAll(`{${name}}`, String(value)),
    template
  );
}

/**
 * ルールIDに対応する診断メッセージを返す。
 *
 * @param messageLanguage 表示に使うメッセージ言語。
 * @param ruleId CERT-CルールID。
 * @param fallback ルールIDが未登録の場合に使用する解析コア由来のメッセージ。
 * @returns ローカライズ済み診断メッセージ。
 */
export function localizeDiagnosticMessage(
  messageLanguage: CertCMessageLanguage,
  ruleId: string,
  fallback: string
): string {
  return localizeRuleDiagnosticMessage(messageLanguage, ruleId, fallback);
}
