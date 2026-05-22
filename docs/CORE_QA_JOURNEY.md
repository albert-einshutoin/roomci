# Core QA Journey

The canonical evaluator journey now lives in
[`PRODUCT_GUIDE.md`](PRODUCT_GUIDE.md).

The intended flow remains:

1. Run a curated scenario locally or in Docker.
2. Inspect timeline, observability, JSON, Markdown, and JUnit artifacts.
3. Add customer-supplied adapter contracts.
4. Drive `roomci serve` with standard MQTT, Modbus, or HTTP clients.
5. Use the evidence to decide what still needs real hardware, private specs, or
   future protocol-profile work.
