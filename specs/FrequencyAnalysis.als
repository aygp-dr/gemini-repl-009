/* Alloy specification for Frequency Analysis patterns */

module FrequencyAnalysis

/* Items being analyzed */
sig Item {}

/* A frequency counter */
sig Counter {
    counts: Item -> one Int,
    total: one Int
} {
    /* Total must equal sum of all counts */
    total = sum i: Item | counts[i]
    /* All counts are non-negative */
    all i: Item | counts[i] >= 0
}

/* Top-K result */
sig TopK {
    k: one Int,
    items: seq Item,
    percentages: seq Int,
    fromCounter: one Counter
} {
    /* K must be positive */
    k > 0
    
    /* Items and percentages have same length */
    #items = #percentages
    
    /* At most K items */
    #items <= k
    
    /* Items are sorted by count (descending) */
    all i: items.inds - items.lastIdx |
        fromCounter.counts[items[i]] >= fromCounter.counts[items[i+1]]
    
    /* Percentages are calculated correctly */
    all i: items.inds |
        percentages[i] = mul[fromCounter.counts[items[i]], 100] / fromCounter.total
}

/* Frequency analysis operation */
pred analyze[input: seq Item, c: Counter] {
    /* Count occurrences */
    all i: Item |
        c.counts[i] = #{j: input.inds | input[j] = i}
}

/* Top-K extraction */
pred extractTopK[c: Counter, k: Int, result: TopK] {
    result.k = k
    result.fromCounter = c
    
    /* Result contains the actual top K items */
    all i: Item |
        (i in result.items.elems) iff 
        (#{j: Item | c.counts[j] > c.counts[i]} < k)
}

/* Composability property */
pred composable[in1, in2: seq Item, c1, c2, c12: Counter] {
    analyze[in1, c1] and
    analyze[in2, c2] and
    analyze[in1 + in2, c12] implies
    /* Combined counts equal sum of individual counts */
    all i: Item |
        c12.counts[i] = c1.counts[i] + c2.counts[i]
}

/* Invariants */
assert NoNegativeCounts {
    all c: Counter, i: Item | c.counts[i] >= 0
}

assert TopKOrdering {
    all t: TopK, i: t.items.inds - t.items.lastIdx |
        t.fromCounter.counts[t.items[i]] >= t.fromCounter.counts[t.items[i+1]]
}

assert PercentageSum {
    /* Sum of all percentages <= 100 */
    all t: TopK |
        (sum i: t.items.inds | t.percentages[i]) <= 100
}

/* Example run commands */
run analyze for 5 Item, 10 Int
run extractTopK for 5 Item, 3 Int
check NoNegativeCounts for 5
check TopKOrdering for 5
check PercentageSum for 5