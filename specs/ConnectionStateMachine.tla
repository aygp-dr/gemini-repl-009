------------------------- MODULE ConnectionStateMachine -------------------------
(* State machine for Gemini REPL connection management *)

EXTENDS Integers, FiniteSets

CONSTANTS
    MaxConnections,     \* Maximum concurrent connections
    ConnectionTimeout   \* Connection timeout value

VARIABLES
    connections,        \* Set of active connections
    connectionStates,   \* Mapping of connection ID to state
    nextConnId         \* Next connection ID to assign

\* Connection states
States == {"connecting", "connected", "authenticated", "disconnecting", "closed"}

\* Type invariant
TypeOK ==
    /\ connections \subseteq 1..MaxConnections
    /\ connectionStates \in [connections -> States]
    /\ nextConnId \in 1..(MaxConnections + 1)

\* Initial state
Init ==
    /\ connections = {}
    /\ connectionStates = [c \in {} |-> ""]  \* Empty function
    /\ nextConnId = 1

\* Create new connection
CreateConnection ==
    /\ Cardinality(connections) < MaxConnections
    /\ nextConnId <= MaxConnections
    /\ connections' = connections \union {nextConnId}
    /\ connectionStates' = [c \in connections' |-> 
        IF c = nextConnId THEN "connecting" ELSE connectionStates[c]]
    /\ nextConnId' = nextConnId + 1

\* Establish connection
EstablishConnection(connId) ==
    /\ connId \in connections
    /\ connectionStates[connId] = "connecting"
    /\ connectionStates' = [connectionStates EXCEPT ![connId] = "connected"]
    /\ UNCHANGED <<connections, nextConnId>>

\* Authenticate connection
AuthenticateConnection(connId) ==
    /\ connId \in connections
    /\ connectionStates[connId] = "connected"
    /\ connectionStates' = [connectionStates EXCEPT ![connId] = "authenticated"]
    /\ UNCHANGED <<connections, nextConnId>>

\* Start disconnection
StartDisconnect(connId) ==
    /\ connId \in connections
    /\ connectionStates[connId] \in {"connected", "authenticated"}
    /\ connectionStates' = [connectionStates EXCEPT ![connId] = "disconnecting"]
    /\ UNCHANGED <<connections, nextConnId>>

\* Close connection
CloseConnection(connId) ==
    /\ connId \in connections
    /\ connectionStates[connId] = "disconnecting"
    /\ connections' = connections \ {connId}
    /\ connectionStates' = [c \in connections' |-> connectionStates[c]]
    /\ UNCHANGED nextConnId

\* Handle connection timeout
HandleTimeout(connId) ==
    /\ connId \in connections
    /\ connectionStates[connId] \in {"connecting", "connected"}
    /\ connections' = connections \ {connId}
    /\ connectionStates' = [c \in connections' |-> connectionStates[c]]
    /\ UNCHANGED nextConnId

\* Next state relation
Next ==
    \/ CreateConnection
    \/ \E c \in connections : 
        \/ EstablishConnection(c)
        \/ AuthenticateConnection(c)
        \/ StartDisconnect(c)
        \/ CloseConnection(c)
        \/ HandleTimeout(c)

\* Specification
Spec == Init /\ [][Next]_<<connections, connectionStates, nextConnId>>

\* Safety properties
ConnectionLimit ==
    \* Never exceed maximum connections
    Cardinality(connections) <= MaxConnections

ValidTransitions ==
    \* Connections follow valid state transitions
    \A c \in connections :
        /\ connectionStates[c] = "connecting" => 
            connectionStates'[c] \in {"connected", "closed"}
        /\ connectionStates[c] = "connected" => 
            connectionStates'[c] \in {"authenticated", "disconnecting", "closed"}
        /\ connectionStates[c] = "authenticated" => 
            connectionStates'[c] \in {"disconnecting"}
        /\ connectionStates[c] = "disconnecting" => 
            connectionStates'[c] = "closed"

\* Liveness properties
EventualClose ==
    \* All connections eventually close
    \A c \in connections : <>(c \notin connections)

NoOrphans ==
    \* All tracked connections are in the active set
    DOMAIN connectionStates = connections

=============================================================================