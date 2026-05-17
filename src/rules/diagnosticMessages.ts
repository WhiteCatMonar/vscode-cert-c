import type { CertCMessageLanguage } from '../localization';

const diagnosticMessages: Record<CertCMessageLanguage, Record<string, string>> = {
  en: {
    'DCL37-C': 'Do not declare or define reserved identifiers.',
    'EXP31-C': 'Do not include side effects in assert arguments.',
    'EXP45-C': 'Do not use assignment in the controlling expression of a selection or iteration statement.',
    'INT33-C': 'Ensure that the right operand of division or remainder is not zero.',
    'PRE31-C': 'Do not pass arguments with side effects to unsafe function-like macros.'
  },
  ja: {
    'DCL37-C': '予約済み識別子を宣言または定義しないでください。',
    'EXP31-C': 'assertの引数に副作用を含めないでください。',
    'EXP45-C': '選択文または反復文の制御式で代入を使用しないでください。',
    'INT33-C': '除算または剰余演算の右辺が0にならないことを保証してください。',
    'PRE31-C': '安全でない関数形式マクロへ副作用を持つ引数を渡さないでください。'
  }
};

/**
 * CERT-CルールIDに対応する診断メッセージを返す。
 *
 * @param messageLanguage 表示に使うメッセージ言語。
 * @param ruleId CERT-CルールID。
 * @param fallback ルールIDが未登録の場合に使用する解析コア由来のメッセージ。
 * @returns ローカライズ済み診断メッセージ。
 */
export function localizeRuleDiagnosticMessage(
  messageLanguage: CertCMessageLanguage,
  ruleId: string,
  fallback: string
): string {
  return diagnosticMessages[messageLanguage][ruleId] ?? diagnosticMessages.en[ruleId] ?? fallback;
}
