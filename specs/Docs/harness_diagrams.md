I have created the **Architectural Blueprint: The Agentic Coding Harness** report in your Studio panel, featuring full technical documentation and breakdown sections.

Below are the four detailed architecture and workflow diagrams for the coding agent harness, formatted in Mermaid.

---

### 1. Flowchart Diagram of a Coding Agent Harness Workflow

```mermaid
flowchart TD
    %% 1. Ingestion & Control Setup
    subgraph Governance [1. Governance & Control]
        TaskDef[Task Definition File <br> T-###.md]
        Orch[Orchestrator Agent]
        State[(run-state.md & Run Report)]
        
        TaskDef -->|Input| Orch
        Orch <-->|Tracks Stage & Logs Decisions| State
    end

    %% 2. Specification & Design Pipeline
    subgraph SpecPipeline [2. Specification & Design Pipeline]
        PO[PO-Spec Agent]
        Spec[Feature Specification]
        Design[Feature Design Agent]
        Note[Design Note]
        TL[Tech Lead Agent]
        Plan[Implementation Plan]

        PO -->|Generates| Spec
        Spec -->|Input| Design
        Design -->|Generates| Note
        Note -->|Input| TL
        Spec -->|Input| TL
        TL -->|Generates| Plan
    end

    %% 3. Bounded Execution & Testing
    subgraph Execution [3. Bounded Execution Sandbox]
        Build[Build Agent]
        
        subgraph Sandbox [Git Worktree Sandbox]
            Code[Source Code Changes]
            Tests[Automated Test Suite]
            ToolLoop[Tool Loop: Read, Search, Edit, Bash]
        end
        
        QA[QA Agent]
        QAReport[QA Review Artifact]

        Plan -->|Input| Build
        Note -->|Input| Build
        
        Build -->|Executes Tools| Sandbox
        Sandbox -->|Outputs| Code
        Sandbox -->|Outputs| Tests
        
        Code -->|Input| QA
        Tests -->|Input| QA
        QA -->|Generates| QAReport
    end

    %% 4. Decision Gate
    subgraph Gate [4. Quality & Decision Gate]
        Decision{QA & Build Passed?}
        Merge[Human Review & Git Merge]
        Retry[Feedback Loop to Build Agent]

        QAReport --> Decision
        Decision -->|Yes| Merge
        Decision -->|No: Attempts <= 3| Retry
        Retry --> Build
    end

    %% Orchestrator Control Connections
    Orch -.->|1. Invokes & Validates| PO
    Orch -.->|2. Invokes & Validates| Design
    Orch -.->|3. Invokes & Validates| TL
    Orch -.->|4. Invokes & Validates| Build
    Orch -.->|5. Invokes & Validates| QA

    %% Styling
    style Orch fill:#1e3d59,stroke:#fff,stroke-width:2px,color:#fff
    style Decision fill:#d66011,stroke:#fff,color:#fff
    style Sandbox fill:#f5f0e1,stroke:#1e3d59,stroke-width:1px
    style Governance fill:#e8f4f8,stroke:#1e3d59,stroke-width:1px
```

---

### 2. C4 Subsystem Diagram (Containers & Communication Interfaces)

```mermaid
graph TB
    %% External Actor
    Developer[Developer / Human Supervisor]

    %% System Boundaries
    subgraph Harness [Agentic Coding Harness Environment]
        
        subgraph Driving [Driving Adapters / Interfaces]
            CLI[CLI Runner / Term]
            IDE[IDE Extension]
        end

        subgraph Core [Isolated Harness Core]
            Orch[Orchestrator & State Manager]
            RunState[(run-state.md)]
            AgentRunner[Skinny Agent Executor]
        end

        subgraph ToolRegistry [Driven Tool Adapters]
            FSAdapter[Filesystem Adapter <br> Read, List, Edit]
            SearchAdapter[Ripgrep Search Adapter]
            ExecAdapter[Bash Sandbox Adapter]
            TreeSitter[Tree-sitter Parser]
        end

        subgraph Storage [Repository Workspace]
            Worktree[Git Worktree Sandbox]
            SpecsDir[specs Artifacts Directory]
        end
    end

    %% External Infrastructure
    subgraph External [External LLM Infrastructure]
        LLMAPI[LLM API / Inference Provider]
    end

    %% Communications & Interfaces
    Developer -->|1. Triggers task / Approves diff| CLI
    Developer -->|1. Triggers task / Approves diff| IDE

    CLI -->|2. Inbound Commands & Task IDs| Orch
    IDE -->|2. Inbound Commands & Task IDs| Orch

    Orch <-->|3. Reads/Writes Phase & Logs| RunState
    Orch -->|4. Instantiates Active Role| AgentRunner

    AgentRunner -->|5. Tool Call Invocation| FSAdapter
    AgentRunner -->|5. Tool Call Invocation| SearchAdapter
    AgentRunner -->|5. Tool Call Invocation| ExecAdapter
    AgentRunner -->|5. Structural Code Query| TreeSitter

    FSAdapter <-->|6. Reads/Writes Code & Artifacts| Worktree
    FSAdapter <-->|6. Persists Specs & Notes| SpecsDir
    ExecAdapter <-->|7. Runs Commands & Tests| Worktree
    SearchAdapter <-->|8. Scans Repository| Worktree

    AgentRunner <-->|9. Prompt Context & MCP Tool Schemas / Completion Responses| LLMAPI

    %% Styling
    style Core fill:#1f4e79,stroke:#fff,stroke-width:2px,color:#fff
    style ToolRegistry fill:#2e75b6,stroke:#fff,color:#fff
    style Storage fill:#f2f2f2,stroke:#333,color:#000
    style External fill:#e1f5fe,stroke:#0288d1,color:#000
```

---

### 3. Detailed Sequence Diagram: Harness & LLM Interaction with Tool Calling

```mermaid
sequenceDiagram
    autonumber
    actor Harness as Harness Core (Agent Runner)
    participant LLM as LLM API (Inference Engine)
    participant Tool as Tool Adapter (Ripgrep / Edit)
    participant FS as Git Worktree Sandbox

    Note over Harness: Binds System Prompt + Role Definition + Task Context
    Harness->>LLM: 1. POST /v1/chat/completions <br> (Messages, Tools: [read_file, search_code, edit_file, bash])
    
    Note over LLM: Evaluates Context & Decides Action
    LLM-->>Harness: 2. Response (finish_reason="tool_calls") <br> Tool Call: search_code(query="struct Task")

    Note over Harness: Intercepts Tool Call Request
    Harness->>Tool: 3. Invoke search_code(query="struct Task")
    Tool->>FS: 4. Execute `rg -n "struct Task"`
    FS-->>Tool: 5. Matches found in src/task.rs:12
    Tool-->>Harness: 6. Return Tool Output ("src/task.rs:12: pub struct Task...")

    Note over Harness: Appends Tool Result Message to Conversation History
    Harness->>LLM: 7. POST /v1/chat/completions <br> (Updated Context with Tool Result)

    Note over LLM: Formulates Targeted Code Edit
    LLM-->>Harness: 8. Response (finish_reason="tool_calls") <br> Tool Call: edit_file(path="src/task.rs", diff="...")

    Harness->>Tool: 9. Invoke edit_file(path="src/task.rs", diff="...")
    Tool->>FS: 10. Apply Precise File Edit
    FS-->>Tool: 11. File Modified Successfully
    Tool-->>Harness: 12. Return Tool Output ("OK")

    Harness->>LLM: 13. POST /v1/chat/completions <br> (Updated Context with Edit Confirmation)

    LLM-->>Harness: 14. Response (finish_reason="stop") <br> "I have updated src/task.rs to include the new field."

    Note over Harness: Verifies Artifact & Updates run-state.md
```

---

### 4. Flowchart Diagram of Context Token Caching

```mermaid
flowchart TD
    %% Input Prompt Context Layers
    subgraph Request [Incoming Agent Prompt Context]
        sys[1. System Prompt & Harness Guardrails]
        role[2. Static Role Definition Markdown]
        tools[3. MCP / Function Tool Schemas]
        repo[4. Base Repository Map & Specs]
        dynamic[5. Dynamic User Task & Turn Message]
        
        sys --> role --> tools --> repo --> dynamic
    end

    %% Tokenizer & Cache Key Verification
    Request --> Tokenizer[Tokenizer & Cache Hash Generator]
    Tokenizer --> CheckCache{Prefix Match in KV Cache?}

    %% Cache Hit Path
    subgraph CacheHitPath [Cache Hit Branch]
        CheckCache -->|Yes: Shared Prefix Matched| CacheHit[Reuse KV Cache for Layers 1-4]
        CacheHit --> EvalDynamic[Evaluate Only Dynamic Tokens - Layer 5]
    end

    %% Cache Miss Path
    subgraph CacheMissPath [Cache Miss Branch]
        CheckCache -->|No: First Call / Modified Prefix| CacheMiss[Full Prompt Context Tokenization]
        CacheMiss --> WriteCache[Store KV Cache for Layers 1-4]
        WriteCache --> EvalFull[Evaluate Full Context]
    end

    %% Completion Generation
    EvalDynamic --> Generator[LLM Token Generation Engine]
    EvalFull --> Generator
    Generator --> Response[Return Completion & Tool Call Stream]

    %% Styling
    style CacheHit fill:#d4edda,stroke:#28a745,stroke-width:2px,color:#155724
    style CacheMiss fill:#f8d7da,stroke:#dc3545,stroke-width:2px,color:#721c24
    style Generator fill:#1e3d59,stroke:#fff,stroke-width:2px,color:#fff
```

---

💡 Would you like to inspect a sample **Tech Lead Implementation Plan template** or explore how to configure **context token caching headers** for your API requests?