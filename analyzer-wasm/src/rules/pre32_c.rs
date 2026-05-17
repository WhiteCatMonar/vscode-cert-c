//! PRE32-Cの検出実装。

use crate::diagnostic::Diagnostic;
use crate::lexer::{Span, Token};
use crate::rules::common::find_call_arguments;

/// PRE32-Cの診断を返す。
///
/// 関数形式マクロ呼び出しの実引数範囲内に前処理指令行が現れる場合に診断する。
///
/// # 引数
///
/// - `source`: 診断位置を算出するための元ソースコード。
/// - `tokens`: 解析対象ソースから得たトークン列。
///
/// # 戻り値
///
/// PRE32-Cに関する診断一覧。
///
/// TODO: マクロ展開前後の位置関係を保持するプリプロセッサモデルへ置き換える。
pub fn check(source: &str, tokens: &[Token]) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let macro_names = find_function_like_macro_names(source);

    for (index, token) in tokens.iter().enumerate() {
        if !macro_names.iter().any(|name| name == &token.text) {
            continue;
        }
        if index > 0 && tokens[index - 1].text == "define" {
            continue;
        }

        let Some(call) = find_call_arguments(tokens, index) else {
            continue;
        };
        let start = tokens[call.open_index].span.start;
        let end = tokens[call.close_index].span.end;

        if let Some(span) = find_preprocessor_directive_span(source, start, end) {
            diagnostics.push(Diagnostic::new(
                source,
                span,
                "PRE32-C",
                "関数形式マクロの呼出しのなかで前処理指令を使用しない",
            ));
        }
    }

    diagnostics
}

/// 関数形式マクロとして定義された名前を返す。
///
/// # 引数
///
/// - `source`: マクロ定義を探すCソースコード。
///
/// # 戻り値
///
/// 関数形式マクロ名の一覧。
fn find_function_like_macro_names(source: &str) -> Vec<String> {
    let mut names = vec![
        "assert".to_string(),
        "memcpy".to_string(),
        "printf".to_string(),
    ];

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
        if !name.is_empty()
            && !name.contains(char::is_whitespace)
            && !names.iter().any(|item| item == name)
        {
            names.push(name.to_string());
        }
    }

    names
}

/// 指定範囲内の前処理指令行の範囲を返す。
///
/// # 引数
///
/// - `source`: 検索対象のCソースコード。
/// - `start`: 検索開始バイト位置。
/// - `end`: 検索終了バイト位置。
///
/// # 戻り値
///
/// 前処理指令行の`#`位置。見つからない場合は`None`。
fn find_preprocessor_directive_span(source: &str, start: usize, end: usize) -> Option<Span> {
    let mut line_start = source[..start].rfind('\n').map_or(0, |index| index + 1);

    while line_start < end {
        let line_end = source[line_start..]
            .find('\n')
            .map_or(source.len(), |index| line_start + index);
        let search_end = line_end.min(end);
        let line = &source[line_start..search_end];
        let leading = line.len() - line.trim_start().len();
        if line[leading..].starts_with('#') {
            let offset = line_start + leading;
            return Some(Span {
                start: offset,
                end: offset + 1,
            });
        }

        line_start = line_end.saturating_add(1);
    }

    None
}
