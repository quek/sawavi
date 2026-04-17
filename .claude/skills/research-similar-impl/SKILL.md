---
name: research-similar-impl
description: |
  類似 DAW / CLAP ホスト / Rust オーディオプロジェクト（clap-host, clap-validator, clack,
  nih-plug, Meadowlark 等）のソースコードと CLAP / cpal / eframe の公式リファレンスを調査し、
  実装方針レポートを出力する。
  「実装して」「追加して」「修正して」「対応して」「機能を作って」「バグを直して」等、
  コード変更を伴う指示があったとき、または CLAP / cpal / Windows API の使い方が
  不明なときに発動。調査のみ行い、コードの編集は行わない。
argument-hint: "[調査対象の機能名]"
allowed-tools: Bash(git clone *), Bash(git pull *), Read, Grep, Glob, WebSearch, WebFetch, Agent
---

# 類似プロダクト & API リファレンス調査

$ARGUMENTS に関する調査を行い、sing_like_coding での実装方針を立てるためのレポートを出力する。

## 手順

### 1. 調査対象の特定

ユーザーの要求から実装対象の機能 / CLAP インターフェース / Windows API を特定する。
[references.md](references.md) の「機能と API の対応例」を参照。

### 2. リポジトリのクローン

[references.md](references.md) の調査対象プロジェクトから、機能に関連するものを `/tmp` にクローンする。

```bash
[ -d /tmp/clap-host ]      || git clone --depth 1 https://github.com/free-audio/clap-host.git /tmp/clap-host
[ -d /tmp/clap ]           || git clone --depth 1 https://github.com/free-audio/clap.git /tmp/clap
[ -d /tmp/clack ]          || git clone --depth 1 https://github.com/prokopyl/clack.git /tmp/clack
[ -d /tmp/nih-plug ]       || git clone --depth 1 https://github.com/robbert-vdh/nih-plug.git /tmp/nih-plug
[ -d /tmp/clap-validator ] || git clone --depth 1 https://github.com/free-audio/clap-validator.git /tmp/clap-validator
[ -d /tmp/meadowlark ]     || git clone --depth 1 https://github.com/MeadowlarkDAW/Meadowlark.git /tmp/meadowlark
```

- クローン先は `/tmp` 配下（作業ディレクトリを汚さない）
- `--depth 1` で軽量クローン
- 既存ならスキップ

### 3. 並列調査（Agent を並列起動）

以下の A) と B) を **Agent を並列起動して同時に** 実行する。

**A) 類似プロダクトのソースコード調査**

クローン済みリポジトリを Grep / Read で横断検索。調査ポイント:
- CLAP インターフェース（`clap_plugin_*`, `clap_host_*`, `clap_process`, `clap_event_*`）の呼び出し順序・契約
- RT オーディオスレッドの設計（ロックフリー・SPSC キュー・事前確保）
- プラグインのライフサイクル（`create` → `init` → `activate` → `start_processing` → `process` → `stop_processing` → `deactivate` → `destroy`）
- CLAP スキャン（`clap_plugin_factory`、バンドル列挙、プリセット）
- オートメーション / パラメータ変更の送信（`clap_event_param_value`、`clap_event_param_mod`）
- MIDI → CLAP イベント変換
- プラグインウィンドウ埋め込み（`clap_plugin_gui`、Win32 の `HWND` 親子関係）
- サブプロセス化（プラグインごとに別プロセスで動かす場合）とその IPC

**B) 公式 API リファレンス・ガイド調査**

[references.md](references.md) の API ドキュメント URL を WebFetch / WebSearch で調査:
- CLAP 公式仕様・拡張（`ext/*.h`）
- `cpal` / `eframe` のスレッドモデルと制約
- `windows` crate（0.61 系）の該当 API シグネチャ
- Rust 側のベストプラクティスとサンプル

### 4. clap-sys / windows crate の API 確認

C/C++ の API と Rust バインディングでシグネチャが異なる場合がある。
`~/.cargo/registry/src/` 内のソースを Grep して実際の Rust シグネチャを確認する。

確認ポイント:
- `clap-sys` の型定義（関数ポインタのシグネチャ、`*const`/`*mut`、配列長フィールド）
- `windows` crate 0.61 の COM メソッド引数型
- `Result<T>` のラッピング
- `ManuallyDrop` / `VARIANT` の扱い
- 定数名の違い

### 5. レポート出力

[report-template.md](report-template.md) の形式で日本語でまとめる。

## 制約

- **調査のみ**。ファイルの編集・作成・ビルド・インストール等は一切行わない
- CLAP 仕様に関わる機能では `clap-host` と `clap` 本体を最優先で参照する
- Rust 設計パターンは `clack` / `nih-plug` を参照する
- windows crate / clap-sys の API 確認は省略しない
