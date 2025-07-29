// Alloy specification for Gemini REPL conversation model
// This demonstrates formal modeling of our conversation context

module gemini_conversation

// Basic message types
abstract sig Role {}
one sig User extends Role {}
one sig Model extends Role {}
one sig System extends Role {}

// Message content
sig Message {
    role: one Role,
    content: one String,
    timestamp: one Int
} {
    // Messages must have positive timestamps
    timestamp > 0
}

// Conversation is a sequence of messages
sig Conversation {
    messages: seq Message,
    context_window: one Int
} {
    // Context window must be positive
    context_window > 0
    // Can't exceed context window
    #messages <= context_window
}

// Tool calls and responses
sig ToolCall {
    name: one String,
    parameters: set String
}

sig ToolResponse {
    call: one ToolCall,
    result: one String
}

// Extended message with tool support
sig ToolMessage extends Message {
    tool_call: lone ToolCall,
    tool_response: lone ToolResponse
} {
    // Can't have both call and response
    no tool_call or no tool_response
    // Tool responses must reference a call
    some tool_response => some tool_response.call
}

// Facts about conversations
fact ConversationRules {
    // Messages alternate between user and model
    all c: Conversation | all i: c.messages.inds - c.messages.lastIdx |
        let m1 = c.messages[i], m2 = c.messages[i.next] |
            (m1.role = User => m2.role = Model) and
            (m1.role = Model => m2.role = User)
    
    // First message is from user
    all c: Conversation | some c.messages => c.messages[0].role = User
    
    // Timestamps are monotonic
    all c: Conversation | all i: c.messages.inds - c.messages.lastIdx |
        c.messages[i].timestamp < c.messages[i.next].timestamp
}

// Predicates for interesting scenarios
pred show_basic_conversation {
    some c: Conversation | #c.messages >= 4
}

pred show_tool_usage {
    some tm: ToolMessage | some tm.tool_call
    some tm: ToolMessage | some tm.tool_response
}

pred context_overflow {
    some c: Conversation | #c.messages = c.context_window
}

// Run commands
run show_basic_conversation for 5
run show_tool_usage for 4
run context_overflow for 6

// Assertions to verify
assert no_self_conversation {
    // Can't have consecutive messages from same role
    no c: Conversation | some i: c.messages.inds - c.messages.lastIdx |
        c.messages[i].role = c.messages[i.next].role
}

assert tool_response_has_call {
    // Every tool response references a valid call
    all tr: ToolResponse | some tc: ToolCall | tr.call = tc
}

check no_self_conversation for 10
check tool_response_has_call for 10