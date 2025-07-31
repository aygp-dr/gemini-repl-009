#!/bin/sh
# Find top K committers with percentage contributions

# Default to top 10
K=${1:-10}

# Get total commit count
TOTAL=$(git log --oneline | wc -l)

echo "=== Top $K Committers by Commit Count ==="
echo "Total commits: $TOTAL"
echo

# Analyze commit frequency
git log --format='%an' | \
  sort | \
  uniq -c | \
  sort -rn | \
  head -"$K" | \
  awk -v total="$TOTAL" '
    {
      percent = ($1 / total) * 100
      printf "%4d commits (%6.2f%%) - %s", $1, percent, $2
      for (i = 3; i <= NF; i++) printf " %s", $i
      printf "\n"
    }'