# 開発ガイド

## 要件

- VSCode 1.92以降
- Node.js
- Rust toolchain
- `wasm32-unknown-unknown`ターゲット

Rustターゲットが未導入の場合は、以下で追加する。

```powershell
rustup target add wasm32-unknown-unknown
```

## セットアップ

```powershell
npm.cmd install
```

## Gitフック

コミットメッセージの1行目は、以下の形式にする。

```text
[type]: summary
```

使用できるtypeは、`feature`、`fix`、`documentation`、`test`、`refactor`、`optimize`、`change`、`maintenance`、`release`とする。

コミットメッセージテンプレートは`.github/COMMIT_TEMPLATE.md`に配置する。コミットメッセージ検証フックは`.githooks/commit-msg`に配置し、`scripts/check-commit-message.mjs`で検証する。

Gitフックとコミットテンプレートを有効化する場合は、内容を確認したうえで以下を手動実行する。

```powershell
git config core.hooksPath .githooks
git config commit.template .github/COMMIT_TEMPLATE.md
```

## ビルド

```powershell
npm.cmd run compile
```

このコマンドは、Rust/WASM解析コアのビルドとTypeScriptコンパイルを実行する。

## fixture確認

```powershell
npm.cmd run check:fixtures
```

`testdata/cert-c/rules/`以下の検出テスト用CコードをWASM解析コアで解析し、違反例と適合例の期待結果を確認する。

## VSCode拡張機能デバッグ

VSCodeでこのワークスペースを開き、`F5`で拡張機能開発ホストを起動する。

拡張機能開発ホスト側でコマンドパレットから以下を実行する。

```text
CERT-C: ワークスペースをチェック
```

`certC.analyzeOnChange`が有効な場合は、Cファイル編集中に変更されたファイルだけを自動再解析する。

`certC.analyzeOnSave`が有効な場合は、Cファイル保存時に保存されたファイルだけを自動再解析する。

診断メッセージと拡張機能の実行時メッセージは、`certC.messageLanguage`で切り替える。対応言語は`en`と`ja`で、リリース物の既定値は`en`とする。

標準ライブラリ開発向け例外など、チームで共有するCERT-C検出設定は`.vscode/cert-c_config.json`に記載する。コマンドパレットから`CERT-C: 検出設定を開く`を実行すると、独自設定画面で編集できる。VSCode個人設定を混入させずに共有したい場合は、このファイルをGit管理対象にする。`.vscode`自体を共有しないチームでは、`.vscode/`をGit管理対象外にするだけでよい。

```json
{
  "standardLibraryDevelopment": {
    "enabled": true,
    "pathGlobs": [
      "libc/**/*.h",
      "libc/**/*.c"
    ]
  }
}
```

DCL37-C診断のクイックフィックスから`標準ライブラリ開発コードとしてDCL37-C-EX3を適用`を選択すると、対象ファイルのワークスペース相対パスを`.vscode/cert-c_config.json`の`standardLibraryDevelopment.pathGlobs`へ追加し、`standardLibraryDevelopment.enabled`を`true`にする。

`standardLibraryDevelopment.enabled`が有効で、対象ファイルが`standardLibraryDevelopment.pathGlobs`に一致する場合、DCL37-C-EX3を含む標準ライブラリ開発向け例外規定を適用する。通常のアプリケーションコードでは有効にしない。

ENV33-C-EX1を適用する場合は、ENV33-C診断のクイックフィックスから`ENV33-C-EX1を適用`を選択する。拡張機能は対象行の直前に固定形式のコメントを挿入する。

```c
/* cert-c: apply ENV33-C-EX1 */
```

このコメントが同じ行または直前行にあるENV33-C診断は、警告レベルではなく情報レベルとして表示し、診断メッセージにENV33-C-EX1適用済みであることを追記する。

PRE31-C-EX1を適用する場合は、PRE31-C診断のクイックフィックスから`PRE31-C-EX1を適用`を選択する。拡張機能は対象行の直前に固定形式のコメントを挿入する。

```c
/* cert-c: apply PRE31-C-EX1 */
```

このコメントが同じ行または直前行にあるPRE31-C診断は、警告レベルではなく情報レベルとして表示し、診断メッセージにPRE31-C-EX1適用済みであることを追記する。

このリポジトリ自身を解析対象にする場合は、拡張機能開発用の生成物を除外するため、必要に応じてワークスペース設定で`certC.excludeGlobs`を上書きする。

```json
{
  "certC.messageLanguage": "ja",
  "certC.excludeGlobs": [
    "**/.git/**",
    "**/.svn/**",
    "**/.hg/**",
    "**/build/**",
    "**/cmake-build-*/**",
    "**/Debug/**",
    "**/Release/**",
    "**/dist/**",
    "**/vendor/**",
    "**/third_party/**",
    "**/analyzer-wasm/target/**",
    "**/node_modules/**",
    "**/out/**"
  ]
}
```

