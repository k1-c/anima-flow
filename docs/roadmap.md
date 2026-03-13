# Anima Flow — Roadmap

## 実装ロードマップ

認知ループ（観測→内在化→干渉）を段階的に構築する。

### Step 1: 内在化の基盤 + 最初の観測チャネル

Cortex（内在化）を構築し、最初の Connectors（観測）を接続する。

- [ ] Neon (Postgres) のセットアップ（nodes / edges テーブル、pgvector、FTS）
- [ ] Node テンプレート設計（daily, episodes, people, spaces, knowledge, meta, decisions）
- [ ] Google Calendar MCP Connector の導入・設定
- [ ] Gmail MCP Connector の導入・設定
- [ ] Linear MCP Connector の導入・設定
- [ ] `/briefing` スキルのプロトタイプ

### Step 1.5: Context Space の導入

Context Space（文脈空間）を導入し、情報収集のスコープ制御を実現する。

- [ ] space Node 型の定義と CRUD
- [ ] デフォルト Context Space "Personal" の自動作成
- [ ] ConnectorQuery にスコープパラメータを追加
- [ ] Skill 実行時の Context Space 指定（`--space` オプション）
- [ ] Context Space の作成・一時停止・アーカイブの CLI コマンド

### Step 2: 観測の拡充 + 干渉パターンの実装

観測チャネルを増やし、GTD ワークフローで干渉の形を定める。

- [ ] Slack MCP Connector の導入
- [ ] Todoist MCP Connector の導入
- [ ] `/inbox` スキルの実装
- [ ] `/breakdown` スキルの実装

### Step 3: 内在化の深化（記憶の定着 + SSoT）

観測された情報を Cortex に定着させ、内在化の質を高める。

- [ ] DailyNote テンプレート設計・自動生成
- [ ] `/review` スキルの実装
- [ ] 記憶の定着処理の実装（エピソード → 長期記憶への昇格ロジック）
- [ ] SSoT 同期処理の実装（Connector → space Node のステータス更新）
- [ ] Knowledge Base 更新の仕組み

### Step 4: 認知ループの自律化（常駐 + Heartbeat）

認知ループが自律的に回り続ける状態を実現する。

- [ ] Slack Bot（Slack Bolt）のセットアップ
- [ ] Claude Agent SDK によるエージェントロジックの実装
- [ ] Heartbeat（定期実行ループ）の実装
- [ ] 記憶の定着処理・SSoT 同期を Heartbeat に組み込み
- [ ] Notion、Chatwork 等の追加 Connector

### Step 5: 認知ループの精度向上 + 製品化準備（継続的）

- [ ] ワークフローの調整・改善
- [ ] Cortex 構造の最適化（Node の肥大化対策、検索性能）
- [ ] 一般ユーザー向けアーキテクチャ設計

---

## 検討事項・リスク

### 認証・セキュリティ

- 各 SaaS の OAuth / API キー管理が必要
- 特に Gmail・Slack は機密情報を含むため、ローカル実行を前提とする（フェーズ 1）

### Connector の品質

- 既存 OSS の MCP サーバーは成熟度にばらつきがある
- 必要に応じてフォーク・自作する判断が必要

### ユーザー体験の設計（フェーズ 2 向け）

- 一般向け提供時は、Claude Code ではなく独自 UI が必要
- タスク分割の対話 UX が製品の差別化ポイントになる
- GTD に馴染みのないユーザーへのオンボーディング設計

### Cortex の運用

- Node の肥大化: エピソード Node が増え続けるため、定期的な要約・圧縮・アーカイブ戦略が必要
- Edge の品質: エージェントが不適切な Edge を張る可能性 → 定着処理時にユーザーがレビューできる仕組み
- 検索のスケール: Node が数千になった場合 → pgvector + FTS + 再帰 CTE でグラフ探索
- プライバシー: Cortex には機密情報が蓄積されるため、フェーズ 1 はローカル保存を前提とする

### タスクの二重管理問題

- Linear（仕事）と Todoist（プライベート）の両方を扱う場合、エージェントが統合ビューを提供することで解決する
- ただし、書き込み先の判断ロジックは慎重に設計する
