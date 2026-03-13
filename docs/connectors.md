# Anima Flow — Connectors（観測・干渉レイヤー）

## 概要

Connectors は認知ループにおける**観測（Observe）**と**干渉（Intervene）**を担うレイヤーであり、
外部サービスとの接続を統一概念で抽象化する。

**Connectors は Cortex を育てるために存在する。**
「何を観測するか」が Cortex に蓄積される情報の質を決め、
最終的に干渉（Gateway 経由の FB）の質を決定する。

## Query / Mutation の分離

Connectors は **Query（読み取り）** と **Mutation（書き込み）** を明確に分離する。
これは技術的な設計上の理由だけでなく、ユーザーにとっての権限モデルの根幹である。

```
認知ループにおける役割:

  Query  = 観測（Observe）   → 外部サービスの状態を知覚する
  Mutation = 干渉（Intervene） → 外部サービスに対してアクションを実行する
```

### 権限モデル

ユーザーは **サービスごとに** Query と Mutation を個別に許可する:

| サービス         | Query（読み取り）          | Mutation（書き込み）          |
| ---------------- | -------------------------- | ----------------------------- |
| Google Calendar  | スケジュール取得           | 予定作成                      |
| Gmail            | 未読メール取得             | —（Phase 1 対象外）          |
| Linear           | アサインされたイシュー取得 | ステータス変更、コメント追加  |
| Slack            | DM/メンション取得          | メッセージ送信、リアクション  |
| Todoist          | 今日のタスク取得           | タスク完了、タスク作成        |
| Chatwork         | メッセージ取得             | メッセージ送信                |

**例:** 「Linear の情報は見ていいけど、勝手にステータスは変えないで」
→ Linear Query: ✅ / Linear Mutation: ❌

### トレイト構造

```rust
/// サービス識別（Query/Mutation 共通）
trait ConnectorMeta: Send + Sync {
    fn source_name(&self) -> &str;
}

/// 読み取り専用（観測レイヤー）
trait ConnectorQuery: ConnectorMeta {
    async fn fetch_inbox(&self) -> Result<Vec<InboxItem>>;
    async fn is_pending(&self, external_id: &str) -> Result<bool>;
}

/// 書き込み（干渉レイヤー）
trait ConnectorMutation: ConnectorMeta {
    fn supported_actions(&self) -> &[&str];
    async fn execute(&self, request: &MutationRequest) -> Result<MutationResult>;
}
```

一つのサービスが Query のみ、Mutation のみ、または両方を実装できる。
Gmail のように Phase 1 では Query のみのサービスもある。

## Connector の種類

| 種別                 | 接続方向                    | 方式                  | 対象サービス                                    |
| -------------------- | --------------------------- | --------------------- | ----------------------------------------------- |
| **MCP Connector**    | 双方向（R/W）               | MCP Protocol          | Google Calendar, Gmail, Linear, Todoist, Notion |
| **CLI Connector**    | 双方向（R/W）               | ローカルコマンド実行  | （将来の拡張用）                                |
| **Events Connector** | インバウンド（外部 → Agent）| Webhook / Events API  | Slack（Slack Bolt）                             |
| **REST Connector**   | 双方向（R/W）               | HTTP API 直接呼び出し | Chatwork, freee（MCP 未対応サービス）           |

## 設計原則

- **Query/Mutation 分離**: 読み取りと書き込みは概念・コード・権限すべてのレベルで分離する
- **差し替え可能**: 同じサービスでも接続方式を変更できる（例: Slack MCP → Slack Bolt への移行）
- **段階的追加**: 新サービスはまず Query のみで最小限接続し、必要に応じて Mutation を追加する
- **フェーズ 2 での拡張**: 一般向け提供時にユーザーが自分のコネクタを設定できる仕組みへ発展させる

## コネクタ一覧（優先度順）

| 優先度 | サービス        | Query | Mutation | 備考                                        |
| ------ | --------------- | ----- | -------- | ------------------------------------------- |
| P0     | Neon (Postgres) | —     | —        | Cortex ストレージ（内部 DB）                |
| P0     | Google Calendar | ✅    | 🔜       | スケジュールの読み取り、予定作成は後日      |
| P0     | Gmail           | 🔜    | —        | メール読み取りのみ（Phase 1）              |
| P0     | Linear          | ✅    | ✅       | イシュー取得 + ステータス変更/コメント      |
| P1     | Slack           | ✅    | ✅       | メッセージ受信 + 送信                       |
| P1     | Todoist         | ✅    | ✅       | タスク取得 + 完了/作成                      |
| P2     | Notion          | 🔜    | 🔜       | ドキュメント参照                            |
| P2     | Chatwork        | 🔜    | ✅       | メッセージ送信                              |
| P3     | freee 会計      | 🔜    | 🔜       | 経理業務支援                                |
| P3     | freee 人事労務  | 🔜    | 🔜       | 勤怠・給与管理                              |

## Connector と Cortex の関係

Query Connectors は外部サービスから情報を取得し、Cortex に正規化して保存する:

```
ConnectorQuery (観測)          Cortex (内在化)
────────────────────────────────────────────────────
Gmail Query         →      person Node にやりとり文脈を蓄積
Slack Query         →      episode Node にエピソードとして記録
                           space Node に意思決定を抽出
Calendar Query      →      daily Node にスケジュールを記録
Linear Query        →      space Node のステータスを更新
Todoist Query       →      daily Node に今日のタスクを記録
```

Mutation Connectors は Brain の判断に基づいて外部サービスに干渉する:

```
Brain (判断)                   ConnectorMutation (干渉)
────────────────────────────────────────────────────
タスク完了判断      →      Todoist Mutation: complete_task
リマインダー        →      Slack Mutation: send_message
ステータス更新      →      Linear Mutation: update_issue_state
スケジュール提案    →      Calendar Mutation: create_event
```

## Connector と Gateway の関係

Slack Connector は特殊で、**認知ループの観測と干渉の両方**を担う:

- **観測（Query）として**: Slack 上の会話・メンション・リアクションを知覚し、Cortex に内在化する
- **干渉（Mutation）として**: エージェントの応答を Slack に送信する

ユーザーからの FB も Slack 経由で Cortex に戻るため、認知ループが閉じる。

## 認証・セキュリティ

- 各 SaaS の OAuth / API キー管理が必要
- 特に Gmail・Slack は機密情報を含むため、ローカル実行を前提とする（フェーズ 1）
- Connector の認証情報は Vault に保存しない（環境変数 or シークレットマネージャ）
- **Query と Mutation は独立した認証スコープで管理する**
  - OAuth の場合: read-only scope と write scope を分けて取得可能にする
  - API Key の場合: 将来的にユーザー設定ファイルで Query/Mutation 個別に有効化
