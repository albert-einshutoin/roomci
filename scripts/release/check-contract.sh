#!/bin/sh
set -eu

exec ruby scripts/release/check-contract.rb
