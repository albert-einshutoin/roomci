---
name: Evaluator PoC consultation
about: Start with the six first-contact questions before sharing detailed specifications
---

> Use the first six questions from the [Evaluator Intake Kit](https://github.com/albert-einshutoin/roomci/blob/main/docs/EVALUATOR_INTAKE_KIT.md). Answer only what can be shared publicly; `Not obtained / 未取得` is acceptable. Do not post credentials, private keys, personal information, unpublished customer configurations, or confidential logs. You can consult without providing sensitive material. Detailed specifications are considered only after the target and acceptance owner are identified.

1. Which past recovery bug should be reproduced, and what was its impact?
2. What should recovery do, and what deadline is acceptable?
3. How is that bug reproduced and its fix verified today?
4. How is a non-production SUT started, and which fault and publish operations may the test perform?
5. Can topics change per run? Provide only **shareable redacted** desired/reported payload examples, the publisher identity, and a way to correlate a response to this run. Leave sensitive facts unprovided.
6. Who accepts the result and its evidence? A role is enough; do not post personal details.

After the target is chosen, continue with the [detailed intake](https://github.com/albert-einshutoin/roomci/blob/main/docs/EVALUATOR_INTAKE_KIT.md#detailed-intake-after-target-selection) outside the public issue as appropriate.
