# Phase 26 Goal: Adapter Contract Generalization

Phase 26 tracks customer-independent product work that improves `roomci`
without guessing a specific user's private protocol or communication contract.

The goal is to make the adapter contract layer more expressive so future
customer PoCs can map real specs with less custom runtime work:

- validate payload shape beyond `required_fields`;
- improve topic-template and identity diagnostics;
- map acceptance criteria to scenario assertions explicitly;
- provide an evaluator intake kit for collecting the minimum useful protocol
  details from a real user.

This phase must not implement private vendor compatibility, production auth,
TLS, broker semantics, or full protocol stacks without supplied specs.
