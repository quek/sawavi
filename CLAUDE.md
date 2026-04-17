# sing_like_coding

CLAP プラグインをホストする Rust 製のトラッカー / DAW。Renoise ライクなレーン・パターン・オートメーションを持ち、MIDI 入出力と sidechain に対応。

## プロジェクト構成

Cargo workspace (Edition 2024)。

```
common/                 -- プロセス間で共有する型・プロトコル
  src/
    audio_buffer.rs     -- オーディオバッファ
    clap_manager.rs     -- CLAP ファクトリ/スキャンの共通ロジック
    dsp.rs              -- DSP ヘルパ
    event.rs            -- ノート/CC/オートメーションイベント
    plugin.rs / plugin_ref.rs  -- プラグイン参照型
    process_data.rs     -- process() に渡すデータ
    protocol.rs         -- sing_like_coding <-> plugin ホスト間 IPC
    shmem.rs            -- 共有メモリ
sing_like_coding/       -- メインアプリ (eframe/egui)
  src/
    app.rs / app_state.rs   -- アプリケーション状態
    command/                -- ユーザーアクション (plugin_load, song_save, track_add, ...)
    commander.rs            -- コマンドディスパッチ
    communicator.rs         -- プラグインサブプロセスとの通信
    config.rs / device.rs / midi_device.rs
    eval.rs                 -- 式評価（セルの自動生成）
    model/                  -- Song / Track / Lane / LaneItem / Note
    plugin/window.rs        -- CLAP プラグインウィンドウ埋め込み
    singer.rs               -- 再生スレッド（オーディオコールバック）
    song_state.rs / undo_history.rs
    view/                   -- egui UI（MainView, knob, meter 等）
sing_like_coding_plugin/ -- CLAP プラグインホスト側プロセス（別プロセスで各プラグインを動かす）
```

## 技術スタック

| 用途 | クレート |
|---|---|
| UI | `eframe` / `egui` 0.32 |
| オーディオ I/O | `cpal` |
| CLAP ホスト | `clap-sys` + `libloading` |
| MIDI | `midir` / `midly` / `wmidi` |
| IPC | `shared_memory`, `miow`（名前付きパイプ） |
| ウィンドウハンドル | `raw-window-handle`, `windows` crate |
| 並列 | `tokio`, `rayon`, `futures` |

## Development Workflow

ワーキングディレクトリは常に `F:\dev\sing_like_coding`。

```bash
# ビルド＆実行（ワークスペース全体）
make           # cargo build --workspace && cargo run -p sing_like_coding
make release   # リリースビルドで実行

# 個別
cargo build --workspace
cargo run -p sing_like_coding
cargo test --workspace
cargo clippy --workspace -- -D warnings
```

Note: デバッグは Emacs + dap-mode + CodeLLDB（`README.md` 参照）。

## 応答・コミット

- **応答は日本語**
- **コミットメッセージは日本語**
- 既存コミットのスタイル（短い日本語、例: `refactoring`、`CLAP_PROCESS_CONTINUE 以外戻ってこないね`）を踏襲する

## Coding Principles

### ベストプラクティスを追求する
- Rust Edition 2024 / 各 crate は最新版
- モダンな Rust イディオムを採用:
  - `let-else`: 早期リターンに活用
  - `?` 演算子を `match` より優先
  - `unsafe extern` ブロック（Edition 2024 で必須）
  - `ManuallyDrop` / union フィールド書き込みは `(*field)` 経由
  - `#[unsafe(no_mangle)]` 記法

### KISS / DRY
- 最小限の実装で目的を達成する。不要な抽象化を作らない
- 1 関数 1 責務
- 3 回繰り返されたら抽象化を検討

### Single Source of Truth
- 同じデータを複数箇所に複製しない。共有データは 1 箇所で管理するか、dirty tracking で変更箇所のみ永続化
- 「この値は誰が所有し、誰が更新するか」を明確にしてから実装する（Song / SongState / UndoHistory の責務分離）

### 保存と復元のライフサイクル対称性
- 「いつ保存するか」と「いつ復元するか」は対で設計する
- Config / Song のシリアライズは、イベント駆動だけでなく起動・終了の境界でも検討する

### 外部 API の挙動を先に理解する
- 推測で実装→失敗→修正のサイクルは、調査→実装より遅い
- CLAP, `clap-sys`, `cpal`, `eframe`, `windows` crate の挙動はドキュメント・ソースで確認してから組み合わせる
- `windows` crate の API は `~/.cargo/registry/src/` を Grep して実シグネチャを確認する

### エラーを握りつぶさない
- `?` を安易に `ok()` / `unwrap_or_default()` / `unwrap_or()` に置き換えない
- FFI・CLAP コールバック・IPC のエラーは根本原因を調査してから対処
- 特にプラグイン初期化（`create_plugin` → `activate` → `start_processing`）の連鎖失敗は、各ステップの成功を個別に検証する

### 要件にない変更を入れない
- 既存の挙動（デフォルト値、初期状態、キーバインド）を勝手に変えない
- バグ修正ついでのリファクタリング・スタイル変更は別コミット

## Real-Time Audio の制約（最重要）

オーディオコールバック（`singer.rs` の再生スレッド、および CLAP `process()` に至るパス）では以下を**厳守**する。違反するとドロップアウト・クラックルが起きる。

- **ヒープ確保禁止**: `Vec::new()`、`Vec::with_capacity()`、`format!()`、`String`、`.to_vec()`、`.collect()`、`Box::new()` を呼ばない
  - 必要なバッファは再生開始前に確保し、使い回す（`AudioBuffer` を活用）
- **ロック禁止 / `Mutex` NG**: 再生スレッドでブロッキングロックを取らない
  - UI ↔ 再生スレッド間はロックフリーキューや `AtomicXxx`、または共有メモリの snapshot 更新で渡す
- **I/O 禁止**: ファイル I/O・ログ出力・`println!` を呼ばない
  - デバッグは `env_logger` をオフラインフェーズで有効化するか、lock-free リングバッファに溜めて UI スレッドで吐く
- **システムコール最小化**: `Instant::now()` は許容、`SystemTime::now()` や `thread::sleep` は避ける
- **CLAP プロセス契約**: `process()` の戻り値は `CLAP_PROCESS_CONTINUE` 以外ほぼ返ってこない想定。idle 判定（`Song::idle_p`）でプラグインの `process()` 呼び出しをスキップする設計を崩さない

## FFI / CLAP 境界のセキュリティ

CLAP はホスト側が `*const`/`*mut` 構造体を直接渡すため、Rust の通常の所有権モデルの外側で振る舞う。

- **ポインタの null / 境界チェック**: `clap_process`, `clap_event_header` 等の配列長は必ず検証
- **整数キャスト**: `as u32` / `as usize` は切り捨て・オーバーフローを起こす。`saturating_add`、`try_from` を優先
- **外部入力のバッファ**: MIDI デバイス、プラグインが書き込むイベント配列、共有メモリ内容はサイズ上限を検証
- **`from_raw_parts` / `copy_nonoverlapping`**: 長さの妥当性を検証してから使う
- **ハンドル/ポインタの寿命**: HWND・`raw-window-handle`・プラグイン参照のスレッド間共有は所有モデルを明示
- **Song / Config のデシリアライズ**: 読み込み後に `sanitize` でバリデーション（トラック数、レーン数、BPM、サンプルレート等の値域）

## データ整合性の事後検証（Write-then-Verify）

Song を保存する操作は、ユーザーがデータを失う致命ケースを避けるため「書き込み→検証→成功報告」の順を守る。

- `song_save` 完了後に再読み込み相当のパース検証を行い、失敗時は元ファイルを残す
- 外部ライブラリ（serde_json 等）がサイレント失敗する可能性がある操作は検証必須

## Debugging Methodology

- **実データから始める**: save/restore バグではまず永続化済みの `song.json` / `config.json` を直接確認する。コードパス推論より実データ観察が速く正確
- **フルサイクルで検証する**: 個別関数が正しくても、パイプライン全体（UI → コマンド → model → serialize → load → restore）が壊れていれば無意味
- **修正と検証を分離する**: 修正したらユーザーに渡す前に自分で動作確認する
- **動いている既存コードを先に参照する**: 類似機能があれば、ライブラリソースを深追いするより自コードベース内の成功パターンとの差分を見る

### オーディオ/プラグインバグの調査順序（上流→下流）

1. **UI / コマンド**: イベントが発火しているか、引数は正しいか（`command/` 配下）
2. **Model**: `Song` / `Track` / `Lane` / `LaneItem` のデータが期待通りか
3. **Communicator / Protocol**: ホスト ↔ プラグインサブプロセスの IPC で何が送受信されているか
4. **Plugin ホスト側**: `sing_like_coding_plugin` での CLAP 呼び出しの戻り値
5. **CLAP プラグイン本体**: ここは基本触れない。ホスト側の契約違反を疑う

推測で修正するな。各ステップに log を仕込み、実際の呼び出し順序・引数・戻り値を検証してから直す。

### GUI / eframe のバグ

- egui の `key_pressed()` はイベントを消費しないため、同一フレームで複数箇所が true を返す。競合する場合は `input_mut(|i| i.consume_key(...))`
- ウィンドウサイズ / 位置の永続化は毎フレーム `viewport().outer_rect` / `inner_rect` で追跡する

## 振り返りワークフロー（コミット前に実施）

### 1. コードレビュー
コード変更が完了しビルドが通った後、コミット前に全変更箇所をレビューする。

- 変更した全ファイルを読み、以下の観点でチェック:
  - **セキュリティ**: FFI 境界の検証、未検証の外部入力（MIDI、CLAP イベント）、整数キャスト
  - **リアルタイム性**: 再生スレッドのホットパスでヒープ確保・ロック・I/O が入っていないか
  - **正確性**: エッジケース（空レーン、トラック 0 個、サンプルレート変更、idle 状態）
  - **DRY**: 重複コード・マジックナンバーが増えていないか
  - **設計意図との整合**: 本ファイルの Coding Principles と Design Decisions に違反していないか
- 問題があれば修正 → 再ビルド → 再レビュー
- `.claude/skills/review` を呼び出してもよい

### 2. 事実の確認
- 何を変更したか（ファイル・行数・影響範囲）
- 最初のアプローチで解決できたか、途中で方針転換したか

### 3. プロセスの反省
- 実データ（song.json, ログ等）を最初に確認したか？ コード推論に頼りすぎなかったか？
- フルサイクルで検証したか？
- 修正後、ユーザーに渡す前に自分で動作確認したか？

### 4. 原則の抽出
- 汎用的な教訓があるか？ あれば CLAUDE.md の Coding Principles / Debugging Methodology に追記

### 5. 記録の更新
- **CLAUDE.md**: 設計判断、原則、デバッグ手法
- **MEMORY.md**（`~/.claude` 側）: ユーザーの好みや環境固有の注意点
- **.claude/settings.local.json**: 許可設定

## リファクタリングワークフロー

1. **分析**: 探索エージェントで並列調査（構造把握、アンチパターン、RT 制約違反、FFI リスク）
2. **計画**: 問題をフェーズに分割、依存関係順に並べ、各フェーズを独立コミット単位にする
3. **実行**: フェーズ毎に `cargo build --workspace && cargo test --workspace` で検証
4. **検証ルール**:
   - 再生スレッドの `Vec::new()` / `format!()` / `lock()` は削減対象
   - `unwrap()` 排除（本番コードのみ、テストは許容）、`let-else` 優先
   - 重複関数の統合時は呼び出し元を全て更新
5. **完了条件**: `cargo build --release --workspace` が警告なしでパス、`cargo test --workspace` 全件グリーン、手動動作確認

## 類似プロジェクト（参考）

| プロジェクト | 参考ポイント |
|---|---|
| [Renoise](https://www.renoise.com/) | トラッカー UI、オートメーション、パターン構造（クローズドソースだがドキュメント・動画が豊富） |
| [clap-host (free-audio)](https://github.com/free-audio/clap-host) | CLAP ホスト実装のリファレンス (C++) |
| [clap-validator](https://github.com/free-audio/clap-validator) | CLAP プラグイン検証ツール（ホスト側契約の確認） |
| [clack](https://github.com/prokopyl/clack) | Rust 製 CLAP ホスト/プラグインライブラリ |
| [Meadowlark](https://github.com/MeadowlarkDAW/Meadowlark) | Rust 製 DAW、RT オーディオと egui/Dropseed の参考 |
| [nih-plug](https://github.com/robbert-vdh/nih-plug) | Rust 製プラグインフレームワーク（CLAP/VST3）、FFI 設計が参考になる |
