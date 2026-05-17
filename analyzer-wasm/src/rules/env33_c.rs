//! ENV33-Cの検出実装。

use crate::diagnostic::Diagnostic;
use crate::lexer::{Span, Token};
use crate::rules::common::{find_call_arguments, is_declared_name};

const ENV33_C_EX1_COMMENT: &str = "/* cert-c: apply ENV33-C-EX1 */";

/// ENV33-Cの診断を返す。
///
/// `system()`呼び出しを検出する。
///
/// # 引数
///
/// - `source`: 診断位置を算出するための元ソースコード。
/// - `tokens`: 解析対象ソースから得たトークン列。
///
/// # 戻り値
///
/// ENV33-Cに関する診断一覧。
///
/// TODO: コマンドプロセッサの存在確認など、許容条件を環境モデルで扱う。
pub fn check(source: &str, tokens: &[Token]) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    for (index, token) in tokens.iter().enumerate() {
        if token.text != "system"
            || is_declared_name(tokens, index)
            || find_call_arguments(tokens, index).is_none()
        {
            continue;
        }

        let mut diagnostic = Diagnostic::new(
            source,
            token.span,
            "ENV33-C",
            "コマンドプロセッサが必要ない場合は system() を呼び出さない",
        );

        if has_env33_c_ex1_comment(source, token.span) {
            diagnostic = diagnostic
                .with_severity("information")
                .with_exception("ENV33-C-EX1");
        }

        diagnostics.push(diagnostic);
    }

    diagnostics
}

/// ENV33-C-EX1の適用コメントが診断位置に付与されているかを返す。
///
/// # 引数
///
/// - `source`: 解析対象のCソースコード。
/// - `span`: 診断対象のソース範囲。
///
/// # 戻り値
///
/// 同じ行または直前行に適用コメントがある場合は`true`。
fn has_env33_c_ex1_comment(source: &str, span: Span) -> bool {
    let lines: Vec<&str> = source.lines().collect();
    let line_index = source[..span.start]
        .bytes()
        .filter(|byte| *byte == b'\n')
        .count();

    line_has_env33_c_ex1_comment(&lines, line_index)
        || (line_index > 0 && line_has_env33_c_ex1_comment(&lines, line_index - 1))
}

/// 指定行にENV33-C-EX1の適用コメントが含まれるかを返す。
///
/// # 引数
///
/// - `lines`: 改行で分割したソース行。
/// - `line_index`: 判定対象の0始まり行番号。
///
/// # 戻り値
///
/// 固定形式の適用コメントが含まれる場合は`true`。
fn line_has_env33_c_ex1_comment(lines: &[&str], line_index: usize) -> bool {
    lines
        .get(line_index)
        .is_some_and(|line| line.contains(ENV33_C_EX1_COMMENT))
}
