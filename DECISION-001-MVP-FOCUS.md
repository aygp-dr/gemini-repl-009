# Decision Record: Focus on Core MVP Implementation

**Date**: 2025-07-31  
**Status**: Accepted  
**Decision**: Defer plugin system development until after MVP completion

## Context

The project scope expanded from an 8-week MVP REPL implementation to a 34-week ambitious self-hosting plugin system. This expansion occurred before completing the foundational phases (1-4) outlined in the original roadmap.

## Decision

We will:
1. **Focus exclusively on Phases 1-4** of the original IMPLEMENTATION-ROADMAP.org
2. **Defer all plugin system work** until after MVP completion
3. **Complete the core REPL** with basic functionality first
4. **Maintain plugin architecture** as future enhancement documentation only

## Rationale

- **Foundation First**: Cannot build advanced features on incomplete foundation
- **Timeline Reality**: 8-week MVP is achievable; 34-week system is speculative
- **User Value**: Working REPL delivers immediate value; plugin system is future enhancement
- **Technical Debt**: Avoid accumulating debt from premature complexity
- **Resource Focus**: Concentrated effort on core features ensures completion

## Consequences

### Positive
- Clear 8-week path to working product
- Reduced complexity and maintenance burden
- Faster time to user value
- Solid foundation for future enhancements
- Lower risk of project failure

### Negative
- Exciting plugin features delayed
- Some architectural decisions may need revisiting
- Model service work becomes future reference material

## Implementation Plan

### Immediate Actions
1. Archive model service code as future reference
2. Return focus to Phase 1-2 completion
3. Update todos to reflect MVP priorities
4. Complete core REPL features first

### Phase Completion Order
1. **Phase 1**: Foundation Layer (Weeks 1-2)
   - ✓ Core REPL Loop
   - ✓ API Client Infrastructure
   - ✓ Command System
   - ✓ Logging Foundation

2. **Phase 2**: Context Management (Weeks 3-4)
   - [ ] Conversation History
   - [ ] Session Persistence
   - [ ] Token Management
   - [ ] Context Commands

3. **Phase 3**: Tool System (Weeks 5-6)
   - [ ] Security Sandbox
   - [ ] Core Tools (read/write/list/search)
   - [ ] Tool Integration
   - [ ] Tool Commands

4. **Phase 4**: Production Features (Weeks 7-8)
   - [ ] Performance Optimization
   - [ ] Error Handling Polish
   - [ ] UI Enhancements
   - [ ] Configuration System

### Future Work (Post-MVP)
- Plugin system architecture (documented in SELF-HOSTING-PLUGINS-ROADMAP.md)
- Model service implementation (scaffolded in src/models/)
- Self-modification capabilities
- Advanced agent systems

## References

- Original roadmap: IMPLEMENTATION-ROADMAP.org
- Plugin design: SELF-HOSTING-PLUGINS-ROADMAP.md
- Scope analysis: .claude/observations/2025-07-30-scope-analysis.md

## Sign-off

This decision prioritizes delivering a working, production-ready REPL in 8 weeks over speculative advanced features. The plugin system remains a valuable future enhancement but should not block MVP delivery.