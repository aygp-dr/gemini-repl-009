/* Alloy specification for Gemini REPL response codes and transitions */

module ResponseCodes

/* Response code categories */
abstract sig ResponseCode {
    category: one Category,
    retriable: one Bool
}

/* Categories of responses */
enum Category { Success, Redirect, Error, RateLimit }

/* Boolean values */
enum Bool { True, False }

/* Specific response codes */
one sig Code200 extends ResponseCode {} {
    category = Success
    retriable = False
}

one sig Code400 extends ResponseCode {} {
    category = Error
    retriable = False
}

one sig Code429 extends ResponseCode {} {
    category = RateLimit
    retriable = True
}

one sig Code500 extends ResponseCode {} {
    category = Error
    retriable = True
}

one sig Code503 extends ResponseCode {} {
    category = Error
    retriable = True
}

/* Request/Response flow */
sig Request {
    response: lone Response,
    retryAttempts: one Int
}

sig Response {
    code: one ResponseCode,
    nextRequest: lone Request
}

/* Constraints */
fact ValidRetries {
    /* Only retry if response code is retriable */
    all r: Response | 
        r.nextRequest != none implies r.code.retriable = True
}

fact RetryLimit {
    /* Maximum 3 retry attempts */
    all req: Request | req.retryAttempts <= 3
}

fact RetryIncrement {
    /* Each retry increments the counter */
    all r: Response, req: Request |
        r.nextRequest = req implies 
            req.retryAttempts = r.~response.retryAttempts + 1
}

/* Predicates */
pred SuccessfulFlow[req: Request] {
    /* A request that eventually succeeds */
    some r: Response | 
        r in req.^(response.nextRequest.response) and 
        r.code.category = Success
}

pred FailureFlow[req: Request] {
    /* A request that ultimately fails */
    all r: Response |
        r in req.^(response.nextRequest.response) implies
        r.code.category != Success
}

/* Assertions */
assert NoInfiniteRetries {
    /* No request can retry forever */
    no req: Request |
        req in req.^(response.nextRequest)
}

assert EventualTermination {
    /* All request chains eventually terminate */
    all req: Request |
        some r: Response |
            r in req.^(response.nextRequest.response) and
            no r.nextRequest
}

/* Run commands */
run SuccessfulFlow for 5
run FailureFlow for 5
check NoInfiniteRetries for 10
check EventualTermination for 10