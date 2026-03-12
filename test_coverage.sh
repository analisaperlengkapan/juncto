#!/bin/bash
cd rust-app

# Find all rust files in frontend/src
echo "Frontend missing tests:"
for file in $(find frontend/src -name "*.rs" | grep -v "main.rs" | grep -v "lib.rs" | grep -v "mod.rs"); do
  if ! grep -q "#\[cfg(test)\]" "$file" && ! grep -q "#\[test\]" "$file"; then
    echo "  $file"
  fi
done

echo "Backend missing tests:"
for file in $(find backend/src -name "*.rs" | grep -v "main.rs" | grep -v "lib.rs" | grep -v "mod.rs"); do
  if ! grep -q "#\[cfg(test)\]" "$file" && ! grep -q "#\[test\]" "$file"; then
    echo "  $file"
  fi
done

echo "Shared missing tests:"
for file in $(find shared/src -name "*.rs" | grep -v "main.rs" | grep -v "lib.rs" | grep -v "mod.rs"); do
  if ! grep -q "#\[cfg(test)\]" "$file" && ! grep -q "#\[test\]" "$file"; then
    echo "  $file"
  fi
done
