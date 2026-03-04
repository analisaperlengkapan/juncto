#!/bin/bash
cd rust-app/tests/e2e
export PATH=$HOME/.cargo/bin:$PATH
npx playwright test always_on_top.spec.ts
