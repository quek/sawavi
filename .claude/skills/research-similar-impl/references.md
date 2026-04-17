# 調査対象プロジェクト

| プロジェクト | 言語 | 特徴 | クローン先 | URL |
|---|---|---|---|---|
| clap | C (ヘッダ) | **CLAP 仕様そのもの**。拡張ヘッダ (`ext/*.h`) でセマンティクスを確認する。**最優先** | /tmp/clap | https://github.com/free-audio/clap |
| clap-host | C++ | CLAP ホストのリファレンス実装。ライフサイクル・スレッド設計の模範 | /tmp/clap-host | https://github.com/free-audio/clap-host |
| clack | Rust | Rust 製 CLAP ホスト/プラグインライブラリ。安全な Rust ラッパの参考 | /tmp/clack | https://github.com/prokopyl/clack |
| nih-plug | Rust | Rust 製プラグインフレームワーク (CLAP/VST3)。FFI とイベント変換の設計 | /tmp/nih-plug | https://github.com/robbert-vdh/nih-plug |
| clap-validator | Rust | CLAP プラグインを検証するホスト。ホスト側契約の確認に有用 | /tmp/clap-validator | https://github.com/free-audio/clap-validator |
| Meadowlark | Rust | Rust 製 DAW、RT オーディオと UI の参考 | /tmp/meadowlark | https://github.com/MeadowlarkDAW/Meadowlark |

全プロジェクトを調査する必要はない。機能に最も関連するものを優先する。

# API リファレンス・ガイド

| ドキュメント | URL |
|---|---|
| CLAP 公式 | https://github.com/free-audio/clap |
| CLAP ホスト実装ガイド | https://github.com/free-audio/clap/blob/main/include/clap/plugin.h |
| cpal | https://docs.rs/cpal |
| eframe / egui | https://docs.rs/eframe / https://docs.rs/egui |
| windows crate (Rust) | https://microsoft.github.io/windows-docs-rs/ |
| Win32 API | https://learn.microsoft.com/en-us/windows/win32/api/ |
| MIDI (wmidi / midly) | https://docs.rs/wmidi / https://docs.rs/midly |

# 機能と API の対応例

| 機能 | 主な CLAP / Win32 インターフェース |
|---|---|
| プラグインスキャン | `clap_plugin_factory::get_plugin_descriptor` / `create_plugin` |
| 初期化・破棄 | `clap_plugin::init`, `activate`, `start_processing`, `stop_processing`, `deactivate`, `destroy` |
| 音声処理 | `clap_plugin::process`, `clap_process`, `clap_audio_buffer` |
| パラメータ | `clap_plugin_params` (`count`, `get_info`, `get_value`, `text_to_value`, `value_to_text`, `flush`) |
| オートメーション | `clap_event_param_value`, `clap_event_param_mod` (input events) |
| MIDI I/O | `clap_event_note`, `clap_event_midi`, `clap_event_midi_sysex` |
| プリセット | `clap_plugin_preset_load`, `clap_preset_discovery_factory` |
| プラグイン GUI | `clap_plugin_gui` (`create`, `set_parent`, `set_size`, `show`, `hide`, `destroy`) |
| スレッドチェック | `clap_host_thread_check` (main thread / audio thread の判定) |
| ログ | `clap_host_log` (RT スレッドから UI への安全な伝達) |
| ウィンドウ埋め込み | `SetParent`, `SetWindowLongPtrW(GWL_STYLE)`, `raw-window-handle` |
| 低レイテンシ I/O | `cpal::Stream`, WASAPI exclusive mode |
| MIDI 入出力 | `midir::MidiInput` / `MidiOutput`, `wmidi::MidiMessage` |

# 実装で特に注意するポイント

- CLAP の **main thread / audio thread** の区別（各関数のスレッド要件を `plugin.h` で確認）
- `clap_process` のイベント配列は時刻順にソートされている必要がある
- オーディオバッファは CLAP 側が所有する場合と、ホスト側が貸し出す場合があるため `flags` を確認
- サブプロセスでプラグインを動かす場合、共有メモリのレイアウトとシグナリング順を厳密に設計する
