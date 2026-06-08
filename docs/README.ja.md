# roomci ドキュメント索引

このディレクトリは roomci 設計ノートの正本です。文書は番号付きで、
上から下へ小さな本のように読めるよう並んでいますが、各文書は独立して
参照できます。

## オリエンテーション

| # | 文書 | 内容 |
|---|---|---|
| 00 | [エグゼクティブサマリー](00_executive_summary.md) | roomci の 1 ページ概要、対象読者、使い方 |
| Guide | [プロダクトガイド](PRODUCT_GUIDE.md) | 正本カテゴリ、読者、ドメインパック、評価者ジャーニー、比較、非目標 |
| Positioning | [プロダクトポジショニング](PRODUCT_POSITIONING.md) | 正本プロダクトガイドへの安定リンク |
| Domains | [ドメインパック](DOMAIN_PACKS.md) | コアエミュレーターモジュールと再利用可能なドメインパック |
| MQTT | [汎用 MQTT コントラクト](GENERIC_MQTT_CONTRACTS.md) | 汎用 MQTT コマンド/状態の例と現在サポートされるサブセット |
| MQTT serve | [MQTT serve サブセット](MQTT_SERVE_SUBSET.md) | PoC テスト向けの最小 MQTT 3.1.1 CONNECT + QoS0 PUBLISH 入力 |
| Modbus serve | [Modbus TCP サブセット](MODBUS_TCP_SUBSET.md) | PoC テスト向けの最小 Modbus TCP 読み書きエンドポイント |
| Protocols | [プロトコルサポートマトリックス](PROTOCOL_SUPPORT_MATRIX.md) | 動作モデル、serve エンドポイント、テスト済み外部サーフェス、非目標の正本 |
| Protocols | [プロトコルコンフォーマンスレジストリ](PROTOCOL_CONFORMANCE_REGISTRY.md) | 公式リファレンス、実装サブセット、ブラックボックス検証コマンド、明示的な拒否 |
| Protocols | [B Tier プロトコルプロフィール](B_TIER_PROTOCOL_PROFILES.md) | Matter、BACnet、KNX、OPC UA の contract-profile フィクスチャと非目標 |
| Coverage | [ホスピタリティドメインパックカバレッジ](HOSPITALITY_STACK_COVERAGE.md) | ホスピタリティドメインパックのスコープ境界への安定リンク |
| Journey | [コア QA ジャーニー](CORE_QA_JOURNEY.md) | プロダクトガイドで維持される評価者ジャーニーへの安定リンク |
| Adapters | [アダプターコントラクトキット](ADAPTER_CONTRACT_KIT.md) | 企業固有プロトコルコントラクトのテンプレート、例、検証 |
| Adapters | [アダプター SDK サンプル](ADAPTER_SDK_SAMPLES.md) | HTTP、MQTT、Modbus アダプター配線向けの Go、TypeScript、Lua 風の小さな例 |
| Developers | [開発者ワークフロー](DEVELOPER_WORKFLOW.md) | 初回実行、Python 自動化、シナリオデバッグ、CI アーティファクトレビュー |
| Developers | [Python リファレンスクライアント](PYTHON_SDK.md) | Python HTTP リファレンスクライアントと MQTT/Modbus スモークパス |
| Developers | [VSCode ローカル作成アセット](../tools/vscode-roomci/README.md) | シナリオ作成向けのローカル schema association、スニペット、タスク |
| Evidence | [評価エビデンスパック](EVALUATION_EVIDENCE_PACK.md) | コマンド、PoC パック、スコアカード、レポートアーティファクト、非対応機能の開示 |
| Hardware CI | [実機ハードウェアを Docker CI に落とし込むユースケース](HARDWARE_TO_DOCKER_CI_USECASES.md) | 実機キャプチャのリプレイパターン、Docker ゲート、プレゼン骨子 |
| Onboarding | [統合オンボーディング](INTEGRATION_ONBOARDING.md) | 15 分パス、HTTP API、トラブルシューティング、クライアントスニペット |
| Category | [カテゴリ readiness](CATEGORY_READINESS.md) | プロダクトガイドで維持される比較への安定リンク |
| Positioning | [デュアルトラックポジショニング](DUAL_TRACK_POSITIONING.md) | プロダクトガイドで維持される業界/ドメインパックポジショニングへの安定リンク |
| Principles | [設計原則](DESIGN_PRINCIPLES.md) | プロダクト哲学とスコープ境界 |

## プロダクト

| # | 文書 | 内容 |
|---|---|---|
| 02 | [プロダクト要件](02_product_requirements.md) | QA エミュレーター向けのペルソナ、ジョブ、成功指標 |
| 18 | [MVP ロードマップ](18_mvp_roadmap.md) | Phase 0（コントラクト）から Phase 7（本番 readiness）までのフェーズ計画 |

## アーキテクチャ

| # | 文書 | 内容 |
|---|---|---|
| 03 | [アーキテクチャ](03_architecture.md) | crate 境界、ランタイムモデル、仮想時間クロック |
| 04 | [ローカルファースト MQTT アーキテクチャ](04_local_first_mqtt_architecture.md) | 保持状態、QoS1、再接続、ローカル vs クラウドブローカー |
| 05 | [エッジサーバーエミュレーター](05_edge_server_emulator.md) | 主系/副系冗長とフェイルオーバーモデル |
| 06 | [デバイスモデル](06_device_model.md) | デバイスごとの状態、コマンド能力マトリックス |
| 11 | [ネットワークとフェイルオーバー](11_network_and_failover.md) | WAN 障害、バックアップ回線の有効化、障害中の快適性 |

## プロトコルと統合

| # | 文書 | 内容 |
|---|---|---|
| 07 | [ビルディングオートメーションプロトコル戦略](07_building_automation_protocol_strategy.md) | Modbus/DALI/BACnet/KNX を採用する理由と統合方法 |
| 08 | [Modbus 戦略](08_modbus_strategy.md) | レジスタマップの形状、読み取り専用の強制、小数型 |
| 09 | [DALI 照明戦略](09_dali_lighting_strategy.md) | シーンターゲット、フィクスチャごとのレベル、コマンドドロップ障害 |
| 10 | [BMS 運用エミュレーション](10_bms_operations_emulation.md) | アラートパイプライン、Slack/電話エスカレーション、ランブック URL |
| 14 | [インターホンとアクセス制御](14_intercom_and_access_control.md) | 将来のインターホン/アクセス制御統合 |

## シナリオと障害

| # | 文書 | 内容 |
|---|---|---|
| 12 | [制御パネル障害モデル](12_control_panel_fault_model.md) | エミュレーターが注入できる iPad/コントローラー障害 |
| 13 | [快適性自動化](13_comfort_automation.md) | 到着前の空調、快適性指標、自動化コントラクト |
| 15 | [シナリオ仕様](15_scenario_spec.md) | `examples/*.yaml` 向け YAML 形式リファレンス |
| 16 | [障害注入](16_fault_injection.md) | 障害カタログ、ターゲット指定ルール、終了条件 |

## デリバリー

| # | 文書 | 内容 |
|---|---|---|
| 17 | [Docker / CI 設計](17_docker_ci_design.md) | コンテナレイアウトと GitHub Actions 統合 |
| Hardware CI | [実機ハードウェアを Docker CI に落とし込むユースケース](HARDWARE_TO_DOCKER_CI_USECASES.md) | 複数エンジニアの実機キャプチャを Docker CI シナリオに変換する |
| Serve | [HTTP serve MVP プラン](HTTP_SERVE_MVP_PLAN.md) | MQTT 互換 serve モード前の localhost バインド制御/レポート API 計画 |
| Serve | [HTTP serve 動作](HTTP_SERVE_BEHAVIOR.md) | 現在の HTTP 接続、タイムアウト、過負荷時の挙動 |
| Serve | [外部プロトコル深度](EXTERNAL_PROTOCOL_DEPTH.md) | MQTT サブセット境界と BMS/接点の外部エンドポイント |
| PoC | [導入前 PoC チェックリスト](PRE_ADOPTION_POC_CHECKLIST.md) | 外部プロトコルコントラクトと受入基準向けの統合チェックリスト |
| PoC | [汎用スマートホーム評価者チェックリスト](GENERIC_SMARTHOME_EVALUATOR_CHECKLIST.md) | 汎用 IoT、スマートホーム、エッジデバイス、ビルディングオートメーションチーム向け評価チェックリスト |
| Release | [リリースチェックリスト](RELEASE_CHECKLIST.md) | CI、Docker、Compose、レポート、ドキュメント、カバレッジの再現可能ゲート |
| Release | [S Tier エビデンスガイド](S_TIER_EVIDENCE_GUIDE.md) | アダプター CI、タイムラインエクスポート、トレースメタデータ、可観測性アーティファクト、GitHub Actions エビデンス向けのコピー&ペースト評価者パス |
| 20 | [付録: 将来の統合](20_appendix_future_integrations.md) | スコープ外の作業と将来の所在 |

API レベルのリファレンスは、リポジトリルートから `cargo doc --no-deps --open` を
実行してください。ワークスペース内の public type と function はすべて文書化されています。
