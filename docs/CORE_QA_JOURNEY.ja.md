# コア QA ジャーニー

評価者ジャーニーの正本は [`PRODUCT_GUIDE.md`](PRODUCT_GUIDE.md) です。

想定フローは次のとおりです。

1. 厳選されたシナリオをローカルまたは Docker で実行する。
2. timeline、observability、JSON、Markdown、JUnit の成果物を確認する。
3. 顧客供給の adapter contract を追加する。
4. 標準 MQTT、Modbus、HTTP client から `roomci serve` を駆動する。
5. エビデンスをもとに、実機、private spec、将来の protocol profile のどこが必要か判断する。
