------------------------- MODULE GeminiProtocol -------------------------
(* Formal specification of the Gemini REPL protocol interactions *)

EXTENDS Integers, Sequences, TLC

CONSTANTS 
    MaxRetries,      \* Maximum number of retry attempts
    TimeoutMs        \* Request timeout in milliseconds

VARIABLES
    state,           \* Current state of the REPL
    request,         \* Current request being processed
    response,        \* Response from API
    retryCount,      \* Number of retries attempted
    history          \* Command history

\* Type definitions
TypeOK == 
    /\ state \in {"idle", "sending", "waiting", "processing", "error"}
    /\ retryCount \in 0..MaxRetries
    /\ history \in Seq(STRING)

\* Initial state
Init ==
    /\ state = "idle"
    /\ request = <<>>
    /\ response = <<>>
    /\ retryCount = 0
    /\ history = <<>>

\* Send a request to the API
SendRequest(req) ==
    /\ state = "idle"
    /\ state' = "sending"
    /\ request' = req
    /\ retryCount' = 0
    /\ UNCHANGED <<response, history>>

\* Receive response from API
ReceiveResponse ==
    /\ state = "waiting"
    /\ state' = "processing"
    /\ response' \in [status: {"success", "error"}, data: STRING]
    /\ UNCHANGED <<request, retryCount, history>>

\* Handle timeout
HandleTimeout ==
    /\ state = "waiting"
    /\ retryCount < MaxRetries
    /\ state' = "sending"
    /\ retryCount' = retryCount + 1
    /\ UNCHANGED <<request, response, history>>

\* Complete processing
CompleteProcessing ==
    /\ state = "processing"
    /\ state' = "idle"
    /\ history' = Append(history, <<request, response>>)
    /\ request' = <<>>
    /\ response' = <<>>
    /\ UNCHANGED retryCount

\* Error handling
HandleError ==
    /\ state \in {"sending", "waiting", "processing"}
    /\ retryCount >= MaxRetries
    /\ state' = "error"
    /\ UNCHANGED <<request, response, retryCount, history>>

\* Next state relation
Next ==
    \/ \E req \in STRING : SendRequest(req)
    \/ ReceiveResponse
    \/ HandleTimeout
    \/ CompleteProcessing
    \/ HandleError

\* Fairness conditions
Fairness ==
    /\ WF_<<state, request, response>>(ReceiveResponse)
    /\ WF_<<state, history>>(CompleteProcessing)

\* Specification
Spec == Init /\ [][Next]_<<state, request, response, retryCount, history>> /\ Fairness

\* Safety properties
NoDoubleProcessing ==
    \* Cannot process while already processing
    state = "processing" => state' # "processing"

EventualResponse ==
    \* Every request eventually gets a response or error
    state = "sending" ~> (state = "idle" \/ state = "error")

\* Invariants
HistoryConsistent ==
    \* History only grows
    Len(history') >= Len(history)

=============================================================================