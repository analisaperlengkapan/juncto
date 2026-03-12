#!/bin/bash
cd rust-app

echo "Frontend missing tests:"
for file in $(find frontend/src -name "*.rs" | grep -v "main.rs" | grep -v "lib.rs" | grep -v "mod.rs"); do
  if ! grep -q "#\[cfg(test)\]" "$file" && ! grep -q "#\[test\]" "$file"; then
    echo "  $file"
  fi
done
