------------------------- MODULE FrequencyAnalysis -------------------------
(* Formal specification of frequency analysis and Top-K algorithms *)

EXTENDS Integers, Sequences, FiniteSets, TLC

CONSTANTS 
    Items,          \* Set of possible items to analyze
    K,              \* Top K items to return
    MaxCount        \* Maximum count for any item

VARIABLES
    input,          \* Input sequence of items
    counts,         \* Mapping from items to their counts
    sorted,         \* Sorted list of (count, item) pairs
    topK            \* Top K items with percentages

\* Type invariants
TypeOK == 
    /\ input \in Seq(Items)
    /\ counts \in [Items -> 0..MaxCount]
    /\ sorted \in Seq(Nat \X Items)
    /\ topK \in Seq([item: Items, count: Nat, percent: 0..100])

\* Initialize with empty state
Init ==
    /\ input = <<>>
    /\ counts = [i \in Items |-> 0]
    /\ sorted = <<>>
    /\ topK = <<>>

\* Count occurrences of each item
CountItems ==
    /\ counts' = [i \in Items |-> 
        Cardinality({j \in 1..Len(input) : input[j] = i})]
    /\ UNCHANGED <<input, sorted, topK>>

\* Sort items by frequency (descending)
SortByFrequency ==
    /\ sorted' = SortSeq(
        SetToSeq({<<counts[i], i>> : i \in Items}),
        LAMBDA x, y: x[1] > y[1]  \* Descending order
      )
    /\ UNCHANGED <<input, counts, topK>>

\* Calculate top K with percentages
CalculateTopK ==
    LET total == Sum([i \in Items |-> counts[i]])
        topItems == SubSeq(sorted, 1, Min(K, Len(sorted)))
    IN
    /\ topK' = [j \in 1..Len(topItems) |->
        [item |-> topItems[j][2],
         count |-> topItems[j][1],
         percent |-> IF total > 0 
                     THEN (topItems[j][1] * 100) \div total
                     ELSE 0]]
    /\ UNCHANGED <<input, counts, sorted>>

\* Full frequency analysis pipeline
FrequencyAnalysisPipeline ==
    /\ CountItems
    /\ SortByFrequency
    /\ CalculateTopK

\* Properties and Invariants

\* Conservation of items
ConservationOfCounts ==
    LET inputCount == Len(input)
        totalCount == Sum([i \in Items |-> counts[i]])
    IN inputCount = totalCount

\* Top K correctness
TopKCorrect ==
    \A i \in 1..Len(topK)-1 :
        topK[i].count >= topK[i+1].count

\* Percentage validity
PercentageValid ==
    LET totalPercent == Sum([i \in 1..Len(topK) |-> topK[i].percent])
    IN totalPercent <= 100

\* Composability: Can process in chunks
Composable ==
    \* Processing input1 then input2 gives same result as input1 ++ input2
    TRUE  \* Simplified for scaffold

\* Guards: Input validation
ValidInput ==
    /\ Len(input) <= MaxCount * Cardinality(Items)
    /\ \A i \in 1..Len(input) : input[i] \in Items

\* Helper functions (would be implemented in TLC)
Sum(f) == CHOOSE n \in Nat : TRUE  \* Placeholder
SortSeq(s, cmp) == s                \* Placeholder
SetToSeq(s) == CHOOSE seq \in Seq(Items) : TRUE  \* Placeholder
Min(a, b) == IF a < b THEN a ELSE b

\* Specification
Next ==
    \/ /\ input' \in Seq(Items)  \* New input arrives
       /\ UNCHANGED <<counts, sorted, topK>>
    \/ FrequencyAnalysisPipeline

Spec == Init /\ [][Next]_<<input, counts, sorted, topK>>

\* Temporal properties
EventuallyTopK ==
    <>(Len(topK) > 0)

StableTopK ==
    \* Once computed, top K remains stable until new input
    (Len(topK) > 0) => UNCHANGED topK

=============================================================================