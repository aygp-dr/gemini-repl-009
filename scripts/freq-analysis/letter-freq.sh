#!/bin/sh
# Letter frequency analysis (recreating the example)
# Usage: letter-freq.sh < input

cat | \
  tr '[:upper:]' '[:lower:]' | \
  tr -cd '[:alpha:]' | \
  fold -w1 | \
  sort | \
  uniq -c | \
  sort -rn | \
  awk '
    {
      sum += $1
      count[NR] = $1
      letter[NR] = $2
    }
    END {
      for (i = 1; i <= NR; i++) {
        printf "%6.2f%% %s\n", count[i]/sum*100, letter[i]
      }
    }'