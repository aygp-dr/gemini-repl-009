#!/bin/sh
# Generic frequency analysis tool
# Usage: freq-analyze.sh [options] < input
#
# Options:
#   -k N    Show top N items (default: all)
#   -p      Show percentages
#   -c      Case insensitive
#   -w      Word mode (split on whitespace)
#   -l      Letter mode (single character frequency)
#   -d DELIM Use custom delimiter

# Parse options
TOPK=""
SHOW_PERCENT=false
CASE_INSENSITIVE=false
MODE="line"
DELIMITER=""

while getopts "k:pcwld:" opt; do
  case $opt in
    k) TOPK="$OPTARG" ;;
    p) SHOW_PERCENT=true ;;
    c) CASE_INSENSITIVE=true ;;
    w) MODE="word" ;;
    l) MODE="letter" ;;
    d) DELIMITER="$OPTARG" ;;
    *) echo "Usage: $0 [-k N] [-p] [-c] [-w|-l] [-d DELIM]" >&2; exit 1 ;;
  esac
done

# Build pipeline based on mode
case $MODE in
  word)
    # Split on whitespace
    PIPELINE="tr -s '[:space:]' '\n'"
    ;;
  letter)
    # Single character frequency
    PIPELINE="tr -cd '[:alpha:][:space:]' | fold -w1 | grep -v '^[[:space:]]*$'"
    ;;
  line)
    # Default: analyze whole lines
    PIPELINE="cat"
    ;;
esac

# Add case conversion if requested
if [ "$CASE_INSENSITIVE" = true ]; then
  PIPELINE="$PIPELINE | tr '[:upper:]' '[:lower:]'"
fi

# Add custom delimiter processing
if [ -n "$DELIMITER" ]; then
  PIPELINE="$PIPELINE | tr '$DELIMITER' '\n'"
fi

# Execute analysis pipeline
eval "$PIPELINE" | \
  sort | \
  uniq -c | \
  sort -rn | \
  (if [ -n "$TOPK" ]; then head -"$TOPK"; else cat; fi) | \
  if [ "$SHOW_PERCENT" = true ]; then
    # Calculate percentages
    awk '
      {
        sum += $1
        count[NR] = $1
        rest[NR] = $0
      }
      END {
        for (i = 1; i <= NR; i++) {
          n = count[i]
          sub(/^[[:space:]]*[0-9]+[[:space:]]+/, "", rest[i])
          printf "%6d (%6.2f%%) %s\n", n, n/sum*100, rest[i]
        }
      }'
  else
    # Just show counts
    cat
  fi