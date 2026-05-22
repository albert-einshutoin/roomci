# Phase 24 Goal: Core Architecture Hardening

Phase 24 tracks structural engineering work surfaced by the product review.

The goal is to improve long-term extensibility without changing the public
product promise:

- assertions should become typed and exhaustively checked;
- runtime state should be composed from domain sub-states instead of a flat
  catch-all object;
- parser-like protocol code should gain property-style regression coverage;
- serve-mode concurrency and overlay ordering should be tested more deeply;
- protocol-server library adoption should be decided with an ADR instead of
  implicit inertia.

