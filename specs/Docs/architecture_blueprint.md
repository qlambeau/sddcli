Architectural Blueprint: Agentic Coding Harness for SDD/BDD Integration

1. Executive Summary: The Paradigm Shift in AI Engineering

The transition from "Vibe Coding"—unstructured, non-deterministic chat interactions—to Harness-based Engineering is the mandatory evolution required for enterprise-grade AI systems. This architecture is predicated on the Binding Constraint Thesis: the quality, reliability, and safety of an agent’s output are dictated by the surrounding system infrastructure and deterministic guardrails, rather than the raw probabilistic weights of the Large Language Model (LLM).

In this paradigm, the Agent is the "Horse" (raw reasoning power and intelligence) while the Harness is the "System" (the controls, direction, and safety constraints).

Agentic Harnessing is defined as the formal engineering substrate that encapsulates an autonomous agent, mediating all interactions between the reasoning engine and the target compute environment to ensure alignment with rigorous business specifications.

Pure LLM interactions fail at enterprise scale due to the absence of these constraints:

* Operational Brittleness: Without a harness to manage environment state, success rates fluctuate wildly based on input resolution and state coordination.
* The Capability Cliff: Empirical data reveals that while agents achieve 87% human performance on trivial tasks, they drop to 15–32% on complex enterprise workflows without structured assistance.
* Security Vulnerabilities: Unconstrained agents are susceptible to prompt injection, secret exfiltration, and supply-chain attacks if execution is not hardware-isolated.

2. Core Harness Architecture: The Seven-Layer Taxonomy

System Architecture mandates a seven-layer decoupling to manage the Agent-Computer Interface (ACI). The Orchestration Layer serves as the central hub, mediating between external tools, internal memory, and the reasoning engine.

System Subsystem Diagram (C4 Model)

graph TD
    Layer1[Context Loading] --> Layer3
    Layer2[Tool Layer] <--> Layer3(Orchestration Hub)
    Layer4[Execution Hooks] <--> Layer3
    Layer5[Permission Layer] -.-> Layer3
    Layer6[Memory & State] <--> Layer3
    Layer7[Session Lifecycle] --> Layer3

    subgraph "Harness Guardrails"
    Layer5
    Layer4
    end


Architectural Layer Specifications:

1. Context Loading: Mission: Establish environment baseline. Components: Repository indexers, llm_profiles, and initial repo-level ingestion.
2. Tool Layer: Mission: Interface for machine-executable abstractions. Components: Model Context Protocol (MCP) servers and structured skill invocation APIs.
3. Orchestration: Mission: Manage the ACI control loop. Components: ACI transition managers and "Thought + Action" scaffolds.
4. Execution Hooks: Mission: Ensure syntactic and logical integrity. Components: Integrated linters and pre-commit validation triggers.
5. Permission Layer: Mission: Context-aware policy enforcement. Components: eBPF network filters (CubeVS) and OS-level access control.
6. Memory & State: Mission: Persistence of session continuity. Components: saved_agent_configs, KV caches, and agent_settings model dumps.
7. Session Lifecycle: Mission: Manage environment volatility. Components: Snapshot/restore logic and environment cleanup routines.

3. Hexagonal Architecture: Ports, Adapters, and Governing Roles

The harness must implement a Hexagonal Architecture to decouple the reasoning process from implementation details.

* Ports: Abstract interfaces for LLM interaction, Sandbox execution, and Repository management.
* Adapters: Specific implementations such as Anthropic API (LLM), CubeVM (Sandbox), and Git CLI (Repo).

Hexagonal Mapping

graph TD
    subgraph Adapters
        Anthropic[Anthropic API]
        Firecracker[CubeVM/KVM]
        Git[Git/GitHub]
    end
    subgraph Ports
        LLM_P[LLM Port]
        SBX_P[Sandbox Port]
        Repo_P[Repo Port]
    end
    SkinnyAgent((Skinny Agent))
    
    SkinnyAgent --- LLM_P --- Anthropic
    SkinnyAgent --- SBX_P --- Firecracker
    SkinnyAgent --- Repo_P --- Git


Responsibility Matrix: Governing Roles

Feature	Skinny Agent (LLM Reasoning)	Harness (Governing Role)
Logic	Generates code and strategic plans.	Enforces Gherkin syntax and protocol.
Validation	Proposes changes.	Executes linters; rejects invalid syntax.
Security	Requests network/file access.	Blocks private subnets via eBPF/KVM.
Persistence	Generates session data.	Snapshots state via EncryptedJSON.

4. The Agent-Computer Interface (ACI) & Tool Primitives

The ACI is formally specified as the tuple (S, A, T):

* S (State Space): Configurations including filesystem state and UI layouts.
* A (Action Space): A curated set of machine-readable atomic skills.
* T (Transition/Observation): Functions mapping (State x Action) to the next State + Structured Observation.

Mandatory Tool Primitives:

* Read/File Viewer: Strictly limited to 100-line chunks to prevent context window overflow.
* List/Glob: Succinct directory tree navigation to minimize token waste.
* Search (ripgrep): Returns filenames only. Full context is forbidden to prevent "semantic misgrounding" and "perceptual errors" where the agent confuses search results with actual file contents.
* Edit: Integrated linter execution; the harness must block syntactically incorrect code from being written to disk.
* Bash: Standardized feedback returns "Your command ran successfully and did not produce any output" to eliminate agent ambiguity.

Edit-Lint-Retry Loop

sequenceDiagram
    Agent->>Harness: Issue Edit Command
    Harness->>Linter: Run Syntax Check
    Linter-->>Harness: Syntax Error (Detailed)
    Harness->>Agent: Reject Edit (Provide Error Context)
    Agent->>Harness: Issue Corrected Edit
    Harness->>Linter: Run Syntax Check
    Linter-->>Harness: Success
    Harness->>Agent: Confirm Success


5. Sandboxing & Isolation: Multi-Tenant Security

Engineering standards require defense against the six core threat vectors:

1. Secret Exfiltration: Unauthorized CURL to attacker endpoints.
2. Supply-chain Attacks: Malicious packages pulled during build.
3. Host Compromise: Container escapes via kernel CVEs.
4. Data Corruption: Malicious filesystem mutations (e.g., rm -rf /).
5. Network Pivoting: Scanning internal private subnets.
6. Resource Exhaustion: Fork bombs consuming host CPU/RAM.

Isolation Technology Comparison

Dimension	Docker	gVisor	CubeSandbox (CubeVM)
Isolation Level	Shared Host Kernel	User-space "Fake" Kernel	Hardware-level KVM MicroVM
Cold Start	~200ms	Sub-second	<60ms
Memory Overhead	5-10MB	15-30MB	<5MB (CoW)
Production Suitability	Low (Trusted code only)	Medium	High (Untrusted/SaaS)

Advanced Isolation: CubeVM utilizes Copy-on-Write (CoW) memory sharing for sub-100ms checkpoints. Network security is mandated via CubeVS (eBPF), blocking all private subnets (10/8, 172.16/12, 192.168/16) by default.

6. SDD & BDD Core Integration: The AIUP Loop

The harness serves as the bridge between requirements and execution through the Specify, Generate, Validate, Review, Refine (AIUP) cycle.

* BDD Lifecycle: Speculate (Identify goals) -> Illustrate (Concrete examples) -> Formulate (Gherkin specs) -> Automate (Agent execution).
* Gherkin Syntax: The bridging language (Given/When/Then) ensures the business requirement is the executable task.

Mandatory Standard: Example Mapping Harness reliability is maximized through Example Mapping. Uncertainty must be surfaced via concrete examples (Rules, Examples, and Questions) before code generation to prevent "silent failures" and logical drift.

BDD-Driven AIUP Loop

graph TD
    A[Speculate: Goals] --> B[Illustrate: Examples]
    B --> C[Formulate: Gherkin]
    C --> D[Automate: Harness Execution]
    D --> E[Validate: Deterministic Oracles]
    E --> F[Review/Refine]
    F --> A


7. Sequential SDLC Orchestration Pipeline

The harness manages role-based orchestration to transition artifacts through the following evolutionary stages:

1. PO-Spec: Definition of high-level business vision.
2. Feature Design: Mapping capabilities to User Stories.
3. Tech Lead: Scaffolding, interface design, and run-state.md initialization.
4. Build: Agent execution within the hardware-isolated sandbox.
5. QA: Automated Gherkin validation and living documentation update.

Artifact Evolution Lifecycle:

1. User Story: High-level business intent.
2. Gherkin Feature: Behavioral specification (The "What").
3. Step Definition: The "Glue" connecting specification to code.
4. Passing Code: Verified implementation (The "How").

The run-state.md ensures state persistence, culminating in a Run Report that serves as Release Evidence.

8. Memory Management & Context Token Caching

Operational continuity requires structured persistence, particularly when toggling between agent types (e.g., OpenHands to ACP).

* SaaS Persistence Requirements:
  * Dedicated User Column: Stored in the settings backend to prevent silent data drops.
  * EncryptedJSON Storage: Mandatory for all snapshots containing api_key material.
  * Snapshot Targets: Must preserve llm_profiles, condenser, mcp_servers, acp_command, and acp_env.
* Logic: Snapshots are generated using model_dump(mode='json', context={'expose_secrets': True}) during agent kind switches to prevent credential loss.

Context and Snapshot Lifecycle

graph LR
    A[Active Session] -- Switch Kind --> B[model_dump + Secrets]
    B --> C[EncryptedJSON Column]
    C --> D[Restore New Snapshot]
    D --> E[Deep Merge Overrides]
    E --> F[Resume Session]


9. Operational Reliability & Final Directives

To mitigate the 72-point performance gap (87% vs 15%) in complex workflows, the system must prioritize deterministic oracles over "LLM-as-judge" patterns.

Final Directives for System Reliability:

* Hardware Isolation: Isolation via KVM (CubeVM) is non-negotiable for multi-tenant SaaS environments.
* Linter Enforcement: No code edit shall be committed without passing integrated syntax validation.
* Succinct Feedback: Tool outputs must be formatted to prevent "semantic misgrounding" (e.g., ripgrep filenames only).
* Release Evidence: Every agent session must produce a Run Report, serving as the Living Documentation of the system’s state.

The harness is not a wrapper; it is the foundational substrate that ensures AI agents operate with safety, clarity, and accuracy.
