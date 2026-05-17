import type { CertCMessageLanguage } from '../localization';

const diagnosticMessages: Record<CertCMessageLanguage, Record<string, string>> = {
  en: {
    'DCL37-C': 'Do not declare or define a reserved identifier.',
    'ENV33-C': 'Do not call system().',
    'EXP44-C': 'Do not rely on side effects in operands to sizeof, _Alignof, or _Generic.',
    'EXP45-C': 'Do not perform assignments in selection statements.',
    'FIO41-C': 'Do not call getc(), putc(), getwc(), or putwc() with a stream argument that has side effects.',
    'INT33-C': 'Ensure that division and remainder operations do not result in divide-by-zero errors.',
    'MSC30-C': 'Do not use the rand() function for generating pseudorandom numbers.',
    'MSC38-C': 'Do not treat a predefined identifier as an object if it might only be implemented as a macro.',
    'POS33-C': 'Do not use vfork().',
    'PRE31-C': 'Avoid side effects in arguments to unsafe macros.',
    'PRE32-C': 'Do not use preprocessor directives in invocations of function-like macros.'
  },
  ja: {
    'DCL37-C': '予約済み識別子の宣言や定義をしない',
    'ENV33-C': 'コマンドプロセッサが必要ない場合は system() を呼び出さない',
    'EXP44-C': 'sizeof 演算子のオペランドは副作用を持たせない',
    'EXP45-C': '選択文に対して代入を行わない',
    'FIO41-C': '副作用を持つストリーム引数を getc()、putc()、getwc()、putwc() に渡さない',
    'INT33-C': '除算および剰余演算がゼロ除算エラーを引き起こさないことを保証する',
    'MSC30-C': '疑似乱数の生成に rand() 関数を使用しない',
    'MSC38-C': 'マクロとして実装されている可能性のある定義済みの識別子をオブジェクトとして扱わない',
    'POS33-C': 'vfork() を使用しない',
    'PRE31-C': '安全でないマクロの引数では副作用を避ける',
    'PRE32-C': '関数形式マクロの呼出しのなかで前処理指令を使用しない'
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
