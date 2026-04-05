# Runenwerk

Documentation is under docs-site/src/content/docs

Used for getting AI context of the content contained in the target folder.
```pws
out="./$(basename "$PWD")-content.txt"

find . \
\( -path './target' -o -path './node_modules' -o -path './.git' -o -path './dist' -o -path './build' \) -prune -o \
\( -name '*.rs' -o -name 'Cargo.toml' -o -name '*.md' \) \
-type f -print0 \
| sort -z \
| xargs -0 -I{} sh -c '
printf "\n===== FILE: %s =====\n" "$1"
nl -ba "$1"
printf "\n===== END FILE: %s =====\n" "$1"
' sh {} \
> "$out"

echo "Wrote $out"
```