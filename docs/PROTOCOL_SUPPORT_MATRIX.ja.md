# プロトコルサポートマトリックス

このマトリックスは、`roomci` が現在エミュレートする内容、外部クライアントが駆動できる内容、および非目標として残る内容の正本です。

公式仕様リファレンス、実装済みサブセット、ブラックボックス検証コマンド、明示的な拒否については [`PROTOCOL_CONFORMANCE_REGISTRY.md`](PROTOCOL_CONFORMANCE_REGISTRY.md) を参照してください。

サポートレベル:

- `scenario_model`: YAML シナリオ run 内に実行可能な挙動が存在する。
- `serve_endpoint`: `roomci serve` 経由で挙動を駆動または観測できる。
- `external_client_tested`: black-box client がテストまたは Compose で serve surface を実行する。
- `conformance_subset`: `roomci` が文書化された wire-level サブセットを実装する。
- `contract_profile`: adapter-contract フィールドと dry-run fixture が、評価者が後から実 gateway または protocol stack をマップする方法を記述する。
- `unsupported`: 未実装、または意図的にプロダクトスコープ外。

## マトリックス

| ドメイン / プロトコル | 現在のサポートレベル | 実装エビデンス | 外部サーフェス | 顧客が提供すべき入力 | 本番での非目的 |
|---|---|---|---|---|---|
| MQTT コマンド/状態コントラクト | `conformance_subset`, `serve_endpoint`, `external_client_tested` | `crates/roomci-mqtt`, `crates/roomci-serve`, `examples/generic_mqtt_retained_state.yaml`, `examples/generic_mqtt_duplicate_delivery.yaml`, `docs/MQTT_SERVE_SUBSET.md`, CLI と serve テスト、protocol smoke | MQTT 3.1.1 `CONNECT`; QoS0 `PUBLISH`; 設定済み状態トピック向け QoS0 `SUBSCRIBE`/`SUBACK` 保持リプレイ; HTTP 状態/レポート観測 | トピックテンプレート、デバイス ID 抽出ルール、必須 JSON フィールド、保持状態の期待値、QoS/セッションの期待値 | 本番ブローカー、ACL、TLS、永続化、クラスタリング、MQTT 5、QoS1/QoS2 ワイヤーセマンティクス |
| Modbus レジスタ挙動 | `conformance_subset`, `serve_endpoint`, `external_client_tested` | `crates/roomci-device-model`, `crates/roomci-serve`, `examples/modbus_floor_heating.yaml`, `docs/08_modbus_strategy.md`, serve Modbus TCP テスト、protocol smoke | Modbus TCP MBAP; 連続ホールディング/入力レジスタの読み取り; 単一レジスタ書き込み; HTTP 状態/レポート観測 | ユニット ID、レジスタマップ、型、スケール、単位、書き込み可能/読み取り専用モード、コミッショニング閾値 | 完全な Modbus TCP サーバー、RTU 電気挙動、ベンダー固有デバイスエミュレーション、電気コミッショニングの代替 |
| DALI 風照明シーン | `scenario_model` | `crates/roomci-device-model`, `examples/dali_scene_partial_failure.yaml`, `docs/09_dali_lighting_strategy.md`, 障害/アサーションテスト | DALI ワイヤーエンドポイントなし | フィクスチャ ID、シーンターゲット、期待レベル、障害モード、受入閾値 | 完全な DALI バス実装、ゲートウェイ認証、光度検証 |
| 接点 I/O | `scenario_model` | `crates/roomci-device-model`, `examples/bms_sauna_emergency_alert.yaml`, `docs/06_device_model.md`, `docs/10_bms_operations_emulation.md` | 接点変化をシナリオステップで表現し、レポートで観測可能 | 接点 ID、正常/開/閉セマンティクス、重大度マッピング、デバウンス期待値 | 実リレー I/O、ハードウェア安全検証 |
| BMS / 運用アラート | `scenario_model`, `serve_endpoint`, `external_client_tested` | `crates/roomci-ops`, `crates/roomci-serve`, `examples/bms_sauna_emergency_alert.yaml`, `docs/10_bms_operations_emulation.md`, `docs/EXTERNAL_PROTOCOL_DEPTH.md`, ops と serve テスト | `POST /external/bms/contact`; 状態/タイムライン/レポート観測 | アラートソース、重大度、通知ルーティング、チケットライフサイクル、確認コントラクト、ランブック URL | 本番 BMS、実 Slack/電話/チケット統合、インシデント対応保証 |
| エッジコントローラーフェイルオーバー | `scenario_model` | `crates/roomci-edge`, `examples/edge_server_failover.yaml`, `docs/05_edge_server_emulator.md`, エッジテスト | シナリオと serve レポートによる状態/レポート観測 | エッジ ID、フェイルオーバーポリシー、タイムアウト期待値、コマンドルーティング前提 | 実ホームコントローラーの代替、ハードウェア冗長検証 |
| WAN / ネットワークフェイルオーバー | `scenario_model` | `examples/local_first_cloud_outage.yaml`, `examples/starlink_failover.yaml`, `docs/11_network_and_failover.md`, 障害モデル | HTTP `POST /fault` で障害を駆動可能; レポートが結果を公開 | 回線名、劣化/オフラインセマンティクス、フォールバックタイミング、ゲスト影響閾値 | ネットワークシミュレーター、パケットレベル WAN エミュレーション、ISP 検証 |
| アクセス制御 / インターホン | アクセスドリフトは `scenario_model`; インターホンワイヤー挙動は `unsupported` | `examples/access_permission_drift.yaml`, `docs/14_intercom_and_access_control.md`, レポート推奨ロジック | アクセス/インターホンプロトコルエンドポイントなし | ID ソース、アクセスシステムグループ、古いユーザ基準、インターホンイベント/webhook コントラクト | 実ロック/インターホンコントローラー、SIP/DTMF ゲートウェイ、物理アクセス安全保証 |
| 快適性 / HVAC 自動化 | `scenario_model` | `examples/comfort_auto_mode.yaml`, `docs/13_comfort_automation.md`, コアテスト | レポートが快適性アサーション結果を公開 | 快適性ターゲット、ルームモデル前提、センサー名、オーバーライド挙動、合格/不合格閾値 | 物理 HVAC 制御、エネルギー最適化、熱工学検証 |
| Matter ゲートウェイプロフィール | `contract_profile` | `adapter-contracts/examples/matter_gateway_profile.yaml`, `examples/matter_gateway_profile.yaml`, `docs/B_TIER_PROTOCOL_PROFILES.md`, `make protocol-profile-smoke` | アダプターコントラクト検証とドライランシナリオエビデンスのみ | ゲートウェイマップ、デバイス ID、エンドポイント ID、クラスター/属性/コマンドマッピング、期待状態、受入基準 | Matter fabric、コミッショニング、CASE/PASE、Thread/Wi-Fi トランスポート、Matter SDK 統合、認証 |
| BACnet コントラクトプロフィール | `contract_profile` | `adapter-contracts/examples/bacnet_contract_profile.yaml`, `examples/bacnet_contract_profile.yaml`, `docs/B_TIER_PROTOCOL_PROFILES.md`, `make protocol-profile-smoke` | アダプターコントラクト検証とドライランシナリオエビデンスのみ | デバイス ID、オブジェクト型/インスタンス、プロパティ、期待値、イベント/アラームクラス、受入基準 | BACnet/IP エンドポイント、オブジェクトサービス、COV、BBMD、ルーティング、認証 |
| KNX グループアドレスプロフィール | `contract_profile` | `adapter-contracts/examples/knx_group_address_profile.yaml`, `examples/knx_group_address_profile.yaml`, `docs/B_TIER_PROTOCOL_PROFILES.md`, `make protocol-profile-smoke` | アダプターコントラクト検証とドライランシナリオエビデンスのみ | ゲートウェイラベル、グループアドレスマップ、データポイント型、方向、期待値、機能/ルームマッピング、受入基準 | ETS インポート、KNX/IP トンネリングまたはルーティング、テレグラムタイミング、バス挙動、デバイス認証 |
| OPC UA ノードプロフィール | `contract_profile` | `adapter-contracts/examples/opcua_contract_profile.yaml`, `examples/opcua_contract_profile.yaml`, `docs/B_TIER_PROTOCOL_PROFILES.md`, `make protocol-profile-smoke` | アダプターコントラクト検証とドライランシナリオエビデンスのみ | エンドポイントラベル、名前空間、ノード ID、ブラウズ名、属性、期待値、イベント型、受入基準 | OPC UA サーバーエンドポイント、サブスクリプション、セキュリティポリシー、証明書、NodeSet/アドレス空間コンフォーマンス、認証 |

## 互換性の位置づけ

企業が private contract をシナリオおよび adapter 定義にマップすれば、`roomci` は一般的な企業評価に利用できます。企業が実際の protocol contract を供給するまで、特定企業環境との互換性があると記述すべきではありません。

任意の組織において不足している入力は、実際の MQTT トピックとペイロード、Modbus/レジスタマップ、BMS または webhook コントラクト、認証/TLS モデル、デバイス ID モデル、受入基準です。現在の価値は、プライベートシステム互換性の主張ではなく、独自コントラクトを統合するための強固な基盤です。

## ロードマップでの利用

- 行が `scenario_model` のみの場合、次のプロダクトステップは serve endpoint または adapter contract です。
- 行が `serve_endpoint` だが `external_client_tested` でない場合、black-box test または Compose controller を追加します。
- 行が `conformance_subset` の場合、サブセットを文書化し、より広い protocol claim を避けます。
- 実際の評価者が private spec を供給する場合、顧客挙動を core runtime にハードコードするのではなく adapter contract としてエンコードします。
