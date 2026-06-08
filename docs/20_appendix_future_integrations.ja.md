# 20. 付録: 将来の統合

## Matter / CSA Aliro

Matter と CSA Aliro は将来有望な標準ですが、MVP の優先事項ではありません。

将来の位置づけは次のとおりです。

```txt
roomci には、評価者が供給するエンドポイント、クラスター、属性、コマンド、
期待状態マップ向けの Matter ゲートウェイ contract_profile がある。
Matter fabric、コミッショニング、トランスポート、SDK 統合、認証は実装しない。
```

## Home Assistant discovery

Home Assistant MQTT Discovery 相当のサポートは、スマートホーム開発者向けデモを馴染みやすくできますが、ホスピタリティやビルディングオートメーションの評価者は、一般的な Home Assistant 規約よりもローカル MQTT、Modbus、DALI、BMS、ネットワーク運用を優先する傾向があります。

## AWS IoT Core 相当のブリッジ

ローカル／クラウド MQTT アーキテクチャが AWS IoT Core 相当の broker、rule router、queue を利用できるため、一般的な Azure Device Twin サポートよりも関連性が高いです。

想定コンポーネント:

- publish endpoint
- rule routing
- queueing
- Cloud Run 相当サービスへの callback
- delayed delivery
- 障害復旧後の replay

## Azure Device Twin 相当

一般的な IoT 参照としては有用ですが、対象企業が Azure IoT を利用していない限り優先度は低いです。

## BACnet 相当の BMS オブジェクトモデル

将来の BMS 成熟度向上に有用:

- analogInput
- analogValue
- binaryInput
- binaryValue
- device status
- alarm state

Phase 21 では、評価者が供給する object/property マップ向けに BACnet contract_profile を追加します。BACnet/IP endpoint、object services、COV、BBMD、routing、certification は実装しません。

## intercom API / ONVIF / SIP / DTMF

将来モジュール:

- PIN check-in
- photo capture event
- relay contact output
- staff call
- SIP/DTMF simulation
- local PBX fallback
- ONVIF recording event

## アクセス制御と identity provider のドリフト

将来モジュール:

- identity group source
- access-system group target
- drift detection
- deletion job canary test
- staging compatibility report

## Grafana / InfluxDB / Prometheus

MVP では JSON/Prometheus 相当のメトリクスをエクスポートできます。後続バージョンでは次を提供可能:

- Influx line protocol
- Grafana dashboard JSON
- alert rule templates

## CAD/BIM と floorplan

将来の方向性:

- floorplan からの fixture/device マップ
- 図面上のアラート位置
- 客室／エリア別のコミッショニングチェックリスト
- 制御盤端子マッピング

## AI 支援運用

将来の AI 統合:

- インシデント要約の生成
- ランブック提案
- 異常の説明
- コミッショニングレポート草案
- 現場問題のクラスタリング

AI は core logic ではなく assistant-layer として位置づけるべきです。
