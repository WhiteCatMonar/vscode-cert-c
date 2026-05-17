//! PRE31-Cの検出実装。

use crate::diagnostic::Diagnostic;
use crate::lexer::{Span, Token, tokenize};
use crate::rules::common::{find_call_arguments, find_side_effect};

const PRE31_C_EX1_COMMENT: &str = "/* cert-c: apply PRE31-C-EX1 */";

/// PRE31-Cの診断を返す。
///
/// 関数形式マクロの置換リストで同じ仮引数が複数回現れる場合、その仮引数は安全でないと
/// みなし、呼び出し側の対応する実引数に副作用があれば診断する。
///
/// # 引数
///
/// - `source`: 診断位置を算出するための元ソースコード。
/// - `tokens`: 解析対象ソースから得たトークン列。
///
/// # 戻り値
///
/// PRE31-Cに関する診断一覧。
pub fn check(source: &str, tokens: &[Token]) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let macros = find_unsafe_function_like_macros(source);

    for (macro_name, parameters) in macros {
        for (index, token) in tokens.iter().enumerate() {
            if token.text != macro_name {
                continue;
            }

            let Some(call) = find_call_arguments(tokens, index) else {
                continue;
            };

            for (argument_index, argument) in call.arguments.iter().enumerate() {
                if parameters.get(argument_index).copied().unwrap_or(false) {
                    if let Some(side_effect) = find_side_effect(argument) {
                        let mut diagnostic = Diagnostic::new(
                            source,
                            side_effect.span,
                            "PRE31-C",
                            "安全でないマクロの引数では副作用を避ける",
                        );

                        if has_pre31_c_ex1_comment(source, side_effect.span) {
                            diagnostic = diagnostic
                                .with_severity("information")
                                .with_exception("PRE31-C-EX1");
                        }

                        diagnostics.push(diagnostic);
                    }
                }
            }
        }
    }

    diagnostics
}

/// 関数形式マクロを探し、仮引数ごとに「複数回評価され得るか」を返す。
///
/// # 引数
///
/// - `source`: マクロ定義を探すCソースコード。
///
/// # 戻り値
///
/// マクロ名と、仮引数ごとの複数回評価可能性を表す配列。
fn find_unsafe_function_like_macros(source: &str) -> Vec<(String, Vec<bool>)> {
    let mut macros = vec![("assert".to_string(), vec![true])];

    for line in source.lines() {
        let trimmed = line.trim_start();
        if !trimmed.starts_with("#define ") {
            continue;
        }

        let rest = trimmed.trim_start_matches("#define ").trim_start();
        let Some(name_end) = rest.find('(') else {
            continue;
        };
        let name = rest[..name_end].trim();
        if name.is_empty() || name.contains(char::is_whitespace) {
            continue;
        }

        let Some(parameter_end) = rest[name_end + 1..].find(')') else {
            continue;
        };
        let parameters_text = &rest[name_end + 1..name_end + 1 + parameter_end];
        let replacement = &rest[name_end + 2 + parameter_end..];
        let parameters: Vec<&str> = parameters_text
            .split(',')
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .collect();
        let unsafe_parameters = parameters
            .iter()
            .map(|parameter| count_identifier_occurrences(replacement, parameter) > 1)
            .collect();

        macros.push((name.to_string(), unsafe_parameters));
    }

    macros
}

/// 指定した識別子がテキスト内にトークンとして現れる回数を返す。
///
/// # 引数
///
/// - `text`: 出現回数を調べるテキスト。
/// - `identifier`: 検索対象の識別子名。
///
/// # 戻り値
///
/// 識別子トークンとして現れた回数。
fn count_identifier_occurrences(text: &str, identifier: &str) -> usize {
    tokenize(text)
        .iter()
        .filter(|token| token.text == identifier)
        .count()
}

/// PRE31-C-EX1の適用コメントが診断位置に付与されているかを返す。
///
/// # 引数
///
/// - `source`: 解析対象のCソースコード。
/// - `span`: 診断対象のソース範囲。
///
/// # 戻り値
///
/// 同じ行または直前行に適用コメントがある場合は`true`。
fn has_pre31_c_ex1_comment(source: &str, span: Span) -> bool {
    let lines: Vec<&str> = source.lines().collect();
    let line_index = source[..span.start]
        .bytes()
        .filter(|byte| *byte == b'\n')
        .count();

    line_has_pre31_c_ex1_comment(&lines, line_index)
        || (line_index > 0 && line_has_pre31_c_ex1_comment(&lines, line_index - 1))
}

/// 指定行にPRE31-C-EX1の適用コメントが含まれるかを返す。
///
/// # 引数
///
/// - `lines`: 改行で分割したソース行。
/// - `line_index`: 判定対象の0始まり行番号。
///
/// # 戻り値
///
/// 固定形式の適用コメントが含まれる場合は`true`。
fn line_has_pre31_c_ex1_comment(lines: &[&str], line_index: usize) -> bool {
    lines
        .get(line_index)
        .is_some_and(|line| line.contains(PRE31_C_EX1_COMMENT))
}
