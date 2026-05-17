//! 診断データとJSON変換を扱うモジュール。

use crate::lexer::Span;
use crate::utils::line_column;

/// TypeScript側へ返すCERT-C診断。
pub struct Diagnostic {
    rule_id: &'static str,
    message: &'static str,
    severity: &'static str,
    exception: Option<&'static str>,
    line: usize,
    column: usize,
    length: usize,
}

impl Diagnostic {
    /// ソース範囲とルール情報から診断を作成する。
    ///
    /// # 引数
    ///
    /// - `source`: 診断位置を行番号・列番号へ変換するための元ソースコード。
    /// - `span`: 診断対象のソース範囲。
    /// - `rule_id`: CERT-CルールID。
    /// - `message`: 利用者へ表示する診断メッセージ。
    ///
    /// # 戻り値
    ///
    /// TypeScript側へ返却可能な診断データ。
    pub fn new(source: &str, span: Span, rule_id: &'static str, message: &'static str) -> Self {
        let (line, column) = line_column(source, span.start);
        Self {
            rule_id,
            message,
            severity: "warning",
            exception: None,
            line,
            column,
            length: (span.end - span.start).max(1),
        }
    }

    /// 診断の重要度を変更した診断を返す。
    ///
    /// # 引数
    ///
    /// - `severity`: TypeScript側へ返す診断重要度。
    ///
    /// # 戻り値
    ///
    /// 指定した重要度を持つ診断データ。
    pub fn with_severity(mut self, severity: &'static str) -> Self {
        self.severity = severity;
        self
    }

    /// 適用済み例外IDを付与した診断を返す。
    ///
    /// # 引数
    ///
    /// - `exception`: 適用済み例外ID。
    ///
    /// # 戻り値
    ///
    /// 例外IDを持つ診断データ。
    pub fn with_exception(mut self, exception: &'static str) -> Self {
        self.exception = Some(exception);
        self
    }
}

/// 診断一覧をTypeScript側で扱いやすいJSON文字列へ変換する。
///
/// # 引数
///
/// - `diagnostics`: JSONへ変換する診断一覧。
///
/// # 戻り値
///
/// `{"diagnostics":[...]}`形式のJSON文字列。
pub fn diagnostics_to_json(diagnostics: &[Diagnostic]) -> String {
    let mut output = String::from("{\"diagnostics\":[");

    for (index, diagnostic) in diagnostics.iter().enumerate() {
        if index > 0 {
            output.push(',');
        }
        output.push_str("{\"ruleId\":\"");
        output.push_str(diagnostic.rule_id);
        output.push_str("\",\"message\":\"");
        output.push_str(&escape_json(diagnostic.message));
        output.push_str("\",\"severity\":\"");
        output.push_str(diagnostic.severity);
        output.push_str("\",\"line\":");
        output.push_str(&diagnostic.line.to_string());
        output.push_str(",\"column\":");
        output.push_str(&diagnostic.column.to_string());
        output.push_str(",\"length\":");
        output.push_str(&diagnostic.length.to_string());
        if let Some(exception) = diagnostic.exception {
            output.push_str(",\"exception\":\"");
            output.push_str(&escape_json(exception));
            output.push('"');
        }
        output.push('}');
    }

    output.push_str("]}");
    output
}

/// JSON文字列値に埋め込むための最小エスケープを行う。
///
/// # 引数
///
/// - `value`: JSON文字列値として埋め込む文字列。
///
/// # 戻り値
///
/// JSON文字列値として安全に埋め込める文字列。
fn escape_json(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}
