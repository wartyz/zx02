find src -name "*.rs" | sort | while read f; do
    echo "FILE: $f"
    cat "$f"
    echo
done > all_rs.txt

