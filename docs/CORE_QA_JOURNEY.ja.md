# Core QA Journey

Evaluator journey の正本は [`PRODUCT_GUIDE.md`](PRODUCT_GUIDE.md) です。

想定フローは次の通りです。

1. curated scenario を local または Docker で実行する。
2. timeline、observability、JSON、Markdown、JUnit artifacts を確認する。
3. customer-supplied adapter contract を追加する。
4. 標準 MQTT、Modbus、HTTP client から `roomci serve` を叩く。
5. evidence をもとに、real hardware、private specs、future protocol
   profile のどこが必要か判断する。
