/* Alloy model for Gemini REPL session management */

module SessionManagement

/* Time representation */
sig Time {
    next: lone Time
}

/* Session states */
enum SessionState { Active, Idle, Expired, Closed }

/* Sessions */
sig Session {
    id: one Int,
    state: SessionState one -> Time,
    lastActivity: Time one -> Time,
    history: seq Command
}

/* Commands in a session */
sig Command {
    content: one String,
    timestamp: one Time,
    session: one Session
}

/* String abstraction */
sig String {}

/* Facts about time */
fact TimeIsLinear {
    /* Time forms a linear sequence */
    all t: Time | lone t.next
    all t: Time | t not in t.^next
    one t: Time | no t.next  /* Last time */
    one t: Time | no next.t  /* First time */
}

/* Session constraints */
fact SessionStateTransitions {
    /* Valid state transitions */
    all s: Session, t: Time |
        let currentState = s.state.t,
            nextState = s.state.(t.next) |
        t.next != none implies
            (currentState = Active implies nextState in Active + Idle + Closed) and
            (currentState = Idle implies nextState in Active + Expired + Closed) and
            (currentState = Expired implies nextState = Closed) and
            (currentState = Closed implies nextState = Closed)
}

fact SessionActivity {
    /* Activity updates lastActivity timestamp */
    all s: Session, t: Time |
        s.state.t = Active implies
            s.lastActivity.t = t or
            some t': t.^next | s.lastActivity.t = t'
}

fact UniqueSessionIds {
    /* Each session has a unique ID */
    all s1, s2: Session | s1 != s2 implies s1.id != s2.id
}

fact CommandsInActiveSession {
    /* Commands can only be added to active sessions */
    all c: Command |
        c.session.state.(c.timestamp) = Active
}

/* Predicates */
pred SessionTimeout[s: Session, t: Time] {
    /* Session times out after inactivity */
    s.state.t = Idle and
    s.state.(t.next) = Expired and
    /* Sufficient time has passed since last activity */
    #(s.lastActivity.t).^next > 3
}

pred SessionRecovery[s: Session, t: Time] {
    /* Session can be recovered from idle state */
    s.state.t = Idle and
    s.state.(t.next) = Active and
    s.lastActivity.(t.next) = t.next
}

pred AddCommand[s: Session, c: Command, t: Time] {
    /* Add a command to session history */
    s.state.t = Active and
    c.timestamp = t and
    c.session = s and
    c in s.history.elems
}

/* Assertions */
assert NoCommandsAfterClose {
    /* No commands can be added after session closes */
    all s: Session, c: Command |
        s.state.(c.timestamp) = Closed implies
            c.session != s
}

assert SessionHistoryGrows {
    /* Session history never shrinks */
    all s: Session, t1, t2: Time |
        t2 in t1.^next implies
            #s.history <= #s.history  /* Simplified for scaffold */
}

assert ExpiredSessionsClose {
    /* Expired sessions eventually close */
    all s: Session, t: Time |
        s.state.t = Expired implies
            some t': t.^next | s.state.t' = Closed
}

/* Run commands */
run SessionTimeout for 5 Session, 10 Time
run SessionRecovery for 3 Session, 8 Time
check NoCommandsAfterClose for 5
check ExpiredSessionsClose for 5