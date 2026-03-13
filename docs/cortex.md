# Anima Flow — Cortex Design（内在化レイヤー）

## 概要

Cortex は認知ループにおける**内在化（Internalize）**の中核であり、3 つの役割を担う:

| 役割                               | 内容                                   | 例                                         |
| ---------------------------------- | -------------------------------------- | ------------------------------------------ |
| **Memory（記憶）**                 | エージェントの経験・記憶               | エピソード、人物認識、行動パターン         |
| **Knowledge Base（知識）**         | 蓄積されたナレッジ                     | 手順書、業務知識、学んだこと、参考情報     |
| **SSoT（Single Source of Truth）** | 散在する情報の正規化された全体像       | プロジェクトの現在状態、意思決定ログ       |

各ツール（Linear, Slack, Gmail 等）は**断片的な情報の実行・通信の場（System of Engagement）**であり、
Cortex が**それらを統合した正規化された全体像（System of Record）**を保持する。

Connectors（観測）から取り込まれた情報は Cortex で内在化され、
Context Engine を通じて想起された文脈が Gateway（干渉）へと流れる。
**Cortex は認知ループの中心に位置し、観測と干渉を繋ぐ**。

Cortex は **Node**（情報の最小単位）と **Edge**（Node 間の関連）で構成されるグラフ構造であり、
全体が**連想ネットワーク**として機能する。

### ストレージの実装

Cortex の概念設計はストレージ実装に依存しない:

| フェーズ | ストレージ | ビュー |
|---------|-----------|--------|
| フェーズ 1 | **Neon (Postgres)** | CLI / デバッグ UI |
| フェーズ 1.5 | Neon (Postgres) | Slack Bot |
| フェーズ 2 | Neon (Postgres) | Web UI |

Node は `nodes` テーブル、Edge は `edges` テーブルとして表現する。
pgvector でセマンティック検索、JSONB でメタデータ格納、再帰 CTE でグラフ探索を行う。

コンテキスト選択の詳細は [Context Engine](./context-engine.md) を参照。

## Cortex 構造

Node は `nodes` テーブルに格納され、`type` と `category` で分類される。
以下は概念的な分類体系:

```
Cortex (nodes テーブル)
│
│  ── Memory（記憶） ── category: 'memory' ──────
│   type: 'daily'        # ワーキングメモリ（例: 2026-03-13）
│   type: 'episode'      # 短期記憶（例: 顧客MTGでの要件変更）
│   type: 'person'       # 長期記憶（例: 田中太郎、A社様）
│   type: 'preference'   # メタ記憶（ユーザーの好み・スタイル）
│   type: 'pattern'      # メタ記憶（検出された行動パターン）
│
│  ── Knowledge Base（知識） ── category: 'knowledge'
│   type: 'procedure'    # 手順書・ハウツー（例: 月次レポートの手順）
│   type: 'domain'       # 業務ドメイン知識（例: GTDワークフロー）
│   type: 'learning'     # 学び・気づき（例: OAuth2導入で学んだこと）
│
│  ── SSoT（正規化された全体像） ── category: 'ssot'
│   type: 'space'        # Context Space — 文脈空間（観測スコープ + 状態 + 意思決定）
│   type: 'decision'     # 意思決定ログ（例: API認証方式の決定）
│
│  ── GTD ── category: 'gtd' ────────────────────
│   type: 'inbox'        # GTD Inbox（未整理）
```

---

## Memory（記憶）

### 記憶の階層構造

人間の神経科学における記憶の分類を模倣し、4 層の記憶階層を設ける:

```
記憶の階層          実装 (Node type)                    特性
──────────────────────────────────────────────────────────────────
ワーキングメモリ     type: 'daily'                       今日のコンテキスト
                                                        毎日生成・上書き

短期記憶            type: 'episode'                      直近の出来事
                                                        数日〜数週間で
                                                        重要度に応じて整理

長期記憶            type: 'person', 'space'              蓄積された知識
                    (アトミック Node 群)                  定期的に強化・更新

メタ記憶            type: 'preference', 'pattern'        自分自身についての
                                                        理解・行動パターン
```

### Node と Edge による連想

各 Node は独立した情報の最小単位であり、Edge で連想的に接続される:

```
Node: 田中太郎 (type: 'person')
  metadata: { role: 'プロジェクト担当', last_contact: '2026-03-13' }
  content: コミュニケーションスタイル: 結論から話すのを好む

  Edge → anima-flow開発 (space)        relation: 'belongs_to'
  Edge → A社様 (person)        relation: 'member_of'
  Edge → 月次レポートの手順 (procedure)  relation: 'related_to'
  Edge → 顧客MTGでの要件変更 (episode)  relation: 'participated_in'
```

エージェントが「田中さんに連絡して」と言われたとき、Edge を辿ることで
関連する Context Space・手順・直近のやりとりを芋づる式に想起できる。

### DailyNote（ワーキングメモリ）

```
Node: 2026-03-13 (type: 'daily')
  content:
    ## 今日のスケジュール
    - 10:00-11:00 チーム MTG
    - 14:00-15:00 顧客打ち合わせ（A社様）

    ## 完了したタスク
    - PR #123 レビュー
    - 月次レポート作成

    ## 進行中のタスク
    - 新機能設計ドキュメント（60%）

    ## メモ・気づき
    - A社様から新規要件あり → Linear にチケット化済み

  Edge → A社様 (person)    relation: 'mentioned'
  Edge → anima-flow開発 (space)    relation: 'related_to'
```

---

## Knowledge Base（知識）

蓄積されたナレッジを 3 カテゴリの Node で管理する:

| Node type | 内容 | 例 |
|-----------|------|---|
| `procedure` | 手順書・ハウツー | 月次レポートの手順、freee 月次締め |
| `domain` | 業務ドメイン知識 | GTD ワークフロー、技術仕様 |
| `learning` | 学び・気づき | 技術的な発見、失敗からの教訓 |

Knowledge Base の Node も Edge で Memory や SSoT と接続される。
例えば `[[月次レポートの手順]]` は `[[田中太郎]]` から Edge が張られ、
「誰がこの手順を必要としているか」が連想で辿れる。

---

## SSoT（Single Source of Truth）

### 設計思想

各外部ツールは**実行の場（System of Engagement）**であり、
Cortex は**記録の場（System of Record）**として位置づける。

```
System of Engagement          SSoT (Cortex)            備考
─────────────────────────────────────────────────────────────
Linear (タスク)         →     space Node               タスク実行はLinearが正
Slack (会話)            →     space Node               意思決定・要点だけ抽出
Gmail (メール)          →     person Node              やりとりの文脈を蓄積
Googleカレンダー (予定)  →     daily Node               スケジュールのスナップショット
Notion (ドキュメント)    →     space Node               ドキュメントへのリンク保持
```

### 原則

**外部ツールのデータを複製しない。正規化された状態・文脈・判断のみを記録する。**

例えば、Linear のタスク一覧を Cortex にコピーするのではなく、
Context Space（space Node）に「今どういう状態で、なぜそうなったか」を記録し、
詳細なタスク一覧は Linear へのリンクで参照する。

### Context Space Node の例

Context Space は SSoT カテゴリの Node として保存される。
従来の `project` Node 型は `space` に統合された。
Context Space は Project の上位互換であり、
「観測スコープ」「状態管理」「意思決定ログ」「ライフサイクル管理」を統合する。

```
Node: anima-flow開発 (type: 'space', category: 'ssot')
  metadata: {
    status: 'active',
    scopes: {
      slack: { channels: ['#anima-flow', '#anima-flow-dev'] },
      linear: { teams: ['Anima'], labels: ['anima-flow'] },
      calendar: { keywords: ['anima', 'Anima Flow'] }
    },
    linear: 'https://linear.app/team/project/anima-flow',
    notion: 'https://notion.so/anima-flow-design-doc',
    next_milestone: '2026-03-20'
  }
  content:
    ## 現在の状態
    - 設計フェーズ完了、実装に着手
    - 顧客MTGでの要件変更により一部スコープ変更あり

    ## 意思決定ログ
    - 2026-03-13: API認証方式をOAuth2に決定
    - 2026-03-10: リリース日を3/25→3/30に延期（理由: スコープ変更の影響）

  Edge → 田中太郎 (person)          relation: 'belongs_to'
  Edge → A社様 (person)     relation: 'belongs_to'
  Edge → API認証方式の決定 (decision) relation: 'decided_in'
  Edge → 顧客MTGでの要件変更 (episode) relation: 'related_to'
```

Context Space に関連する Node は `belongs_to` Edge で接続される。
認知ループは Context Space のスコープ内で回り、
取得された情報は自動的にこの Space に Edge で接続される。

詳細は [Context Space](./context-space.md) を参照。

---

## Cortex 更新プロセス（Heartbeat が実行）

人間の脳が睡眠中に記憶を整理するように、
Heartbeat が定期的に Cortex 全体の更新処理を行う。

これは認知ループの**内在化フェーズを自律的に回す**プロセスである。
観測された情報を整理し、Cortex の質を継続的に向上させる:

### Memory の定着

1. **DailyNote（ワーキングメモリ）を走査** — 今日の出来事を確認
2. **新しい人・プロジェクトを検出** → Node を自動生成
3. **既存 Node の Edge を強化** → 関連が見つかれば Edge を追加
4. **繰り返し出てくるパターンを抽出** → type: 'pattern' の Node に昇格
5. **古いエピソードを要約・圧縮** → 重要でなければアーカイブ

### Knowledge Base の更新

6. **新しい手順・知見を検出** → category: 'knowledge' の Node として生成
7. **既存の手順 Node が古くなっていないか確認** → 更新を提案

### SSoT の同期

8. **各 Connector から最新状態を取得** → type: 'space' の Node のステータスを更新
9. **重要な意思決定を検出** → type: 'decision' の Node として記録
10. **外部ツールへのリンク切れを検出** → 修正を提案

---

## 既存アプローチとの比較

| 方式                              | 検索性 | 説明可能性 | 連想             | 人間が編集可能 |
| --------------------------------- | ------ | ---------- | ---------------- | -------------- |
| ベクトル DB のみ                  | ○      | ×          | △ 類似度のみ     | ×              |
| フラット Markdown（OpenClaw 等）  | △      | ○          | × Edge なし      | ○              |
| **Anima Flow Cortex**             | **◎**  | **○**      | **◎ グラフ構造** | **◎**          |

Anima Flow Cortex は、グラフ構造（構造的検索）+ pgvector（セマンティック検索）+
FTS（全文検索）の 3 つの検索手段を組み合わせる。
かつ、ユーザーが記憶を直接閲覧・編集できる透明性を維持する。

---

## DB スキーマ（フェーズ 2）

```sql
-- Node（情報の最小単位）
CREATE TABLE nodes (
  id          TEXT PRIMARY KEY,
  user_id     UUID REFERENCES auth.users(id),  -- マルチテナント
  type        TEXT NOT NULL,    -- 'person', 'space', 'episode', 'decision',
                                -- 'daily', 'procedure', 'domain', 'learning',
                                -- 'inbox', 'preference', 'pattern'
  category    TEXT NOT NULL,    -- 'memory', 'knowledge', 'ssot', 'gtd'
  title       TEXT NOT NULL,
  content     TEXT,             -- Markdown 本文
  metadata    JSONB DEFAULT '{}',
  embedding   vector(1536),    -- pgvector: セマンティック検索用
  access_count INTEGER DEFAULT 0,
  created_at  TIMESTAMPTZ DEFAULT now(),
  updated_at  TIMESTAMPTZ DEFAULT now()
);

-- Edge（Node 間の関連）
CREATE TABLE edges (
  from_id     TEXT REFERENCES nodes(id) ON DELETE CASCADE,
  to_id       TEXT REFERENCES nodes(id) ON DELETE CASCADE,
  relation    TEXT,             -- 'member_of', 'decided_in', 'related_to' 等
  weight      REAL DEFAULT 1.0,
  context     TEXT,             -- Edge が張られた文脈
  created_at  TIMESTAMPTZ DEFAULT now(),
  PRIMARY KEY (from_id, to_id)
);

-- 全文検索インデックス
CREATE INDEX idx_nodes_fts ON nodes
  USING gin(to_tsvector('japanese', title || ' ' || content));

-- ベクトル検索インデックス
CREATE INDEX idx_nodes_embedding ON nodes
  USING ivfflat (embedding vector_cosine_ops);

-- Inbox の重複防止（source + external_id でユニーク）
CREATE UNIQUE INDEX idx_inbox_dedup ON nodes ((metadata->>'source'), (metadata->>'external_id'))
  WHERE type = 'inbox';

-- マルチテナント用 RLS
ALTER TABLE nodes ENABLE ROW LEVEL SECURITY;
CREATE POLICY nodes_user_policy ON nodes
  USING (user_id = auth.uid());
```
