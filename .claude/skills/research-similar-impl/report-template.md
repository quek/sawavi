# レポートテンプレート

調査結果は以下の形式で日本語でまとめる。

```markdown
## 調査結果: [機能名]

### 各プロジェクトの実装

#### clap / clap-host
- ファイル: ...
- パターン: ...
- CLAP インターフェース呼び出し順序: ...
- スレッド要件: ...

#### clack / nih-plug / clap-validator
- Rust 側の安全ラッパの作り方: ...
- 型変換（`*const` → `&`）の境界処理: ...

#### Meadowlark / その他 DAW
- UI 層とオーディオ層の分離: ...
- ロックフリーキュー設計: ...

### API リファレンスからの知見
- CLAP 公式仕様で確認したセマンティクス・スレッド制約
- cpal / eframe の該当 API のベストプラクティス
- Windows API の使用上の注意点

### clap-sys / windows crate の API シグネチャ
- 確認した関数とそのシグネチャ（C との差異）
- 関数ポインタの呼び出し方（`NonNull` / `Option<unsafe extern ...>`）

### 推奨アプローチ
- sing_like_coding での実装方針
- 採用する設計パターンとその理由
- `common/` と `sing_like_coding/` と `sing_like_coding_plugin/` のどこに何を置くか

### リアルタイム安全性
- ホットパスでのヒープ確保・ロック・I/O を回避する方法
- UI ↔ オーディオスレッド間のデータ受け渡し（lock-free / SPSC / Atomic）

### 注意点
- エッジケース（空バッファ、サンプルレート変更、idle、プラグインクラッシュ）
- スレッドセーフティ（main thread / audio thread）
- FFI 境界のバリデーション

### 参考コード
- 具体的なコード例（C++/Rust → sing_like_coding への変換ポイント）
```
