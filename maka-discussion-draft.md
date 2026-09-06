# LayerFS × Maka: Recursive File States for Multi-Agent Collaboration and MCTS RSI

> Status: Local revision of [published Discussion #4881](https://github.com/apache/maka/discussions/4881); these changes are not yet published.

Hi Maka community,

I'm building [LayerFS](https://github.com/Ephemeral-AI-Lab/layerfs), an open-source recursive filesystem for agents. Your article, [“From Copy-on-Write to Mailbox: Two Paths for Multi-Agent Scheduling”](https://github.com/apache/maka/blob/main/docs/blogs/multi-agent-scheduling.zh-CN.md), resonated with how we think about execution boundaries and state.

The article asks what a subagent should inherit, how work should be scheduled, and how results should be delivered. It also describes an execution graph that evolves as intermediate results arrive. We would like to explore the filesystem counterpart: **what if each execution result could reference a retained file state that subsequent work can read, continue, or recursively branch?**

Maka already supports parallel work and an evolving DAG. Our proposed contribution is a branchable file-state layer underneath that execution topology.

## 1. 🧱 The mental model: LayerStack → Branch → Commit → Workspace

Start with a project at a known filesystem state. Agents can work independently from that point, preserve their results, and explore alternative continuations. Selected work becomes the next shared checkpoint.

LayerFS gives each part of that process an explicit place:

![LayerFS architecture: LayerStack checkpoints, Branch histories, Commits, and independent Workspaces](https://raw.githubusercontent.com/Ephemeral-AI-Lab/layerfs/4ef99dcc7cff7d2faf6ee7ff0ba343fa920b014e/docs/assets/diagrams/layerstack-branches-workspaces.png)

*Workspaces under the same Branch represent successive executions. The illustration's upper-right outward “Add” arrow should read Fork/Create; Add publishes a selected Branch head back into the LayerStack.*

- **LayerStack / Layer — shared checkpoints.** A LayerStack records the checkpoint history of a project or environment. Each Layer describes a *complete logical filesystem snapshot*: L0 might be the initial project, L1 one development round, and L2 the next accepted checkpoint. Common contents are shared underneath.
- **Branch — an independent line of progress.** Starts from a Layer or an eligible historical Commit and can represent a task, collaboration unit, or candidate solution. A Branch can last for an entire task.
- **Commit — a retained immutable file state.** Preserves intermediate progress within a Branch. A Commit partway through a task can seed another Branch.
- **Workspace — a temporary writable view.** Presents a pinned Branch state through materialization or FUSE. Tools operate on ordinary files, and changes can be retained as a Commit or discarded.

Publishing a selected Branch head with **Add** creates the next shared Layer.

> **The execution space is temporary; its retained state can outlive it and become the basis of further work.**

Independent executors use identified snapshots, immutable objects carry shared content, and conditional head updates establish an explicit publication boundary.

## 2. 🛠️ Why Workspace per tool call

We want each filesystem-affecting tool call to have **a known input state, an isolated writable view, and an explicit result**.

A subagent task contains many decisions. Suppose it edits an interface, builds the project, revises a caller, and runs tests. After the first edit, an alternative might be worth exploring: keep the interface change but try a different caller implementation. Tool-call checkpoints preserve that exact starting point without requiring the whole subtask to be repeated.

This gives the runtime three useful capabilities:

- **Trace each change:** connect file transitions with task and tool-call identities, providing a basis for *audit and blame*.
- **Preserve progress:** keep earlier results when a later attempt fails, and branch an alternative from a useful checkpoint.
- **Validate exact states:** attach feedback to the file state actually tested.

Private changes remain in their Workspace's execution view until the runtime decides how to retain or integrate them. An asynchronous command keeps its Workspace through its actual execution lifetime, including polling; calls without retained file changes can share the same state identity.

> **Fine-grained execution does not force fine-grained global publication.**

Branches accumulate *intermediate work*; LayerStacks publish *deliberate shared checkpoints*. Tool-call isolation coexists with longer-lived task and collaboration histories.

### ↩️ Reversibility as an executable filesystem capability

The DeepSeek-affiliated Cordis paper, [*A Programming Paradigm for Spatiotemporal Composability*](https://arxiv.org/abs/2608.25512), gives a formal account of revertible effects. Its guarantees are scoped to a model whose effects have suitable inverses: section 6.1 places state outside the recovery boundary when the system cannot exclusively control and restore it. Section 6.7 identifies **COW or immutable storage** as an infrastructure direction for recovering earlier state. [System boundary](https://arxiv.org/pdf/2608.25512#page=70) · [Storage co-design](https://arxiv.org/pdf/2608.25512#page=77)

**A formal recovery guarantee does not, by itself, make arbitrary Agent tool execution reversible.** The filesystem still needs to preserve the earlier bytes and structure, contain new writes, and provide an actual way to resume from the retained state. That engineering obligation is the part LayerFS is building:

- **Preserve the starting point:** retain the Layer or Commit from which execution begins.
- **Contain the attempt:** keep managed file changes inside a private Workspace.
- **Recover by state:** discard an unaccepted attempt, or create a fresh Branch and Workspace from an earlier retained snapshot.
- **Try another future:** run an alternative continuation while keeping useful history available.

For these managed file changes, recovery does not require an inverse command for every edit, overwrite, or generated file. The earlier state remains available, and execution can resume from it. COW and deduplication make preserving those recovery points economical.

> **Reversibility becomes an operation the Agent can invoke: preserve, attempt, return to a retained state, and branch again.**

Our claim is scoped to **file state managed by LayerFS**. Process memory, network requests, and external effects require their own mechanisms. We want reversibility at this boundary to be exercised and checked through real tool executions and state recovery, rather than inferred from a harness's formal model.

## 3. ♻️ Making fine-grained state economical

Tool-call checkpoints, parallel candidates, and recursive branching all increase the number of retained states. Yet most of those states still contain the same project. Dozens of agents may use a large codebase while each changes only a few lines.

> **Every state is complete logically and incremental physically.**

We want new states to reuse existing content and structure, concentrating storage growth in **unique changes and their metadata**. Deduplication supports growth in three dimensions:

- **More checkpoints** across adjacent tool calls.
- **More candidates** across parallel branches.
- **Deeper exploration** across successive generations.

Four mechanisms make that sharing possible:

### Zero-copy fork — reuse the starting state

A new Branch references the immutable root of an existing Layer or eligible Commit. It receives an independent identity and head while retaining shared starting contents. **Creating the Branch copies no canonical content objects.**

### Structural COW — reuse unchanged structure

Copy-on-write preserves earlier states while new states reuse unchanged file and directory structure. Retaining an edit **rebuilds affected structure** and shares unaffected regions and subtrees. One path can modify a file while another continues to read its earlier version.

### CAS — share identical content

Content-addressed storage identifies canonical objects by their contents. Identical objects can be shared across files, Branches, and LayerStacks within one Store, including identical results produced independently. *Execution identities remain distinct even when their contents can be deduplicated.*

### CDC — reuse regions inside edited files

Content-defined chunking establishes boundaries from file content. Local insertions, deletions, and replacements can therefore reuse unchanged regions within a changed file. **CDC exposes reusable regions; CAS identifies and shares their objects.** The amount of reuse depends on the edit and chunking rules.

Together, these mechanisms cover branching, mutation, and retention: reference an existing state, execute independently, preserve the changed content and structure, and make the resulting state available for another continuation.

Our ambition is **COW wherever execution branches in the task tree**—between tool calls, between agents, and across exploration levels. Zero-copy applies to the state fork; executable projections, I/O, and metadata still contribute to end-to-end cost.

## 4. 🔍 What LayerFS adds to Git + worktree

Git + worktree provides repository versions and independent working directories. **LayerFS makes the files needed for Agent execution a reusable state resource: isolated, retained, recoverable, and recursively branchable.**

The advantage is concentrated in three capabilities:

- **Tool-call-level isolation:** Workspace creation, execution, capture, Commit, and End are available through one SDK. A runtime can give each call a known baseline, a private writable view, and a retained result for attribution, recovery, or another continuation.
- **Execution file-state retention:** snapshots can include source, installed dependencies, generated artifacts, and supported filesystem information such as modification times, permission bits, and hard-link relationships.
- **Shared content across independent execution:** zero-copy fork reuses the starting state, COW reuses structure, and CAS + CDC reuse content. FUSE Workspaces read the shared immutable base and record their own changes independently.

| Capability | Git + worktree itself | LayerFS |
| --- | --- | --- |
| Files inherited by a new view | Files from the selected commit; uncommitted, untracked, or ignored source-directory contents are not automatically inherited. | Files included in the retained Layer / Commit, independently of Git tracking status. |
| Filesystem information retained | Content, trees, and limited mode information; ordinary commits do not preserve modification times, full permission bits, or hard-link relationships. | File content and supported directory, timestamp, permission, and hard-link semantics. |
| Prepared dependencies | Uncommitted `node_modules` needs separate installation, copying, or sharing. | An installed dependency tree can be retained in the snapshot used to create subsequent Workspaces. |
| Working-view storage sharing | The worktree command itself does not guarantee COW sharing of checked-out directories. | FUSE views share immutable state with private changes; materialized projections have their own copying costs. |

*Git worktrees support ordinary POSIX operations through their host filesystem. The distinction is what the history retains and restores.* Git also shares content-addressed objects and supports packfile delta compression. See [Git objects](https://git-scm.com/book/en/v2/Git-Internals-Git-Objects), [worktrees](https://git-scm.com/docs/git-worktree), [ignore rules](https://git-scm.com/docs/gitignore), and [packfiles](https://git-scm.com/book/en/v2/Git-Internals-Packfiles).

### 📦 Install once, retain the state, fork for ten Agents

Assume the same execution environment, reusable installed dependencies, and `node_modules` excluded from Git history:

- **Git + worktree:** checking out the same commit into ten directories does not bring along the installed dependencies; they need additional preparation.
- **LayerFS:** install in one Workspace, retain the file state including dependencies, and fork from that state. Each Agent gets an independent view of the prepared files; FUSE shares the underlying immutable content.

**Reusing the prepared state avoids repeating installation just to obtain the same dependencies.** If ten installations are still executed, their download, unpacking, script, and CPU work is not automatically eliminated by deduplication.

LayerFS integrates file-state retention, recovery, branching, content reuse, and execution lifecycles for high-frequency tool calls and multi-agent exploration. End-to-end performance remains a question for equivalent workloads. *Git can continue to own project review and delivery while LayerFS manages the execution states used to reach a result.*

## 5. 🌳 Two applications of a recursive filesystem

### 👥 Multi-agent coding: coordinate independent contributions

Agents work from explicit baselines in independent Branches and Workspaces. Each contribution retains its origin and resulting state, giving reconciliation a base, incoming result, and current state to compare.

The responsibilities stay explicit:

- **Isolation** keeps in-progress edits separate.
- **Reconciliation** organizes independent contributions.
- **Validation** evaluates the state proposed for acceptance.

Overlapping writes and changed dependencies can lead to explicit resolution or another execution. A larger task can recursively delegate subproblems using the same state model.

This is a natural connection to Maka: its execution records could identify which task and tool call produced a file state, while its coordinator manages the work that consumes or integrates that result.

### 🧪 MCTS for multi-lane RSI: evaluate alternative continuations

A search process can fork several candidates from an intermediate state, execute and evaluate them independently, and expand promising results into another generation. File-state recovery makes each retained node a reusable starting point, so a failed continuation can be abandoned while sibling paths remain available.

> **Recursive depth and exploration breadth are separate capabilities.**

A self-improvement loop may run for many iterations along one active trajectory. A multi-lane approach also retains alternatives, enabling different continuations from useful historical states. MCTS provides one way to balance exploration and exploitation across those candidates.

RSI candidates can change an Agent's tools, code, or strategy configuration. The runtime, searcher, and evaluator supply conversation context, execution configuration, and feedback; LayerFS supplies the associated file states and views.

We call this a **Recursive File System** because state, execution, and branching can be composed repeatedly. **RFS before RSI** expresses our belief that self-improvement benefits from a file environment that supports both depth and breadth. Collaboration integrates contributions; search selects and expands candidates. Both depend on isolated execution and reusable state.

## 6. 🗺️ LayerFS roadmap and opportunities with Maka

LayerFS is our **open-source contribution to AgentFS**. We have a working core and are now concentrating on performance, workload coverage, and the next stage of multi-agent state coordination.

### 📅 Current optimization deadline: September 30, 2026

Our target is to **complete the current performance and storage optimization pass by September 30, 2026**, focusing on:

- **Workspace readiness:** creation, projection setup, and the path to tool execution.
- **Capture and Commit:** efficient retention of small edits and larger filesystem changes.
- **Storage efficiency:** CAS / CDC reuse, metadata overhead, and retained-history growth.

### Roadmap

| Stage | Focus |
| --- | --- |
| 0.1.3 — Workspace workloads and optimization | Verify and optimize single-Branch filesystem workloads, including small-file churn, reads, real agent tools, repeated execution, reliability, and deduplication. |
| 0.1.4 — History and branching at scale | Measure and optimize multi-Layer and multi-Branch histories: Fork, Add, Diff, queries, conflict handling, historical reads, and storage reuse. |
| 0.2.0 — Multi-agent Branch coordination and portable projections | Support concurrent Workspaces on one Branch with ordered result acceptance, cumulative reconciliation, changed-read revalidation, and resumable conflicts; establish consistent semantics across projection strategies, including clone/reflink and OverlayFS paths. |

*The September deadline applies to the current optimization pass; the later roadmap stages describe the direction of development.* LayerFS remains a developer preview with one local Store authority, and crash- or power-loss durability is not currently guaranteed.

### 🤝 Where we see opportunities with Maka

Maka's [`SubagentWorktreeExecutor`](https://github.com/apache/maka/blob/411512bd9cb698f668eccbae35adf41e71d7cf68/packages/core/src/subagent-workspace.ts) separates workspace provisioning, patch capture, recovery, and retirement. **We see opportunities to connect this execution boundary with LayerFS's isolated Workspaces, Branch histories, and shared LayerStack checkpoints.**

Maka's scheduling and execution records and LayerFS's file-state model could complement each other. We would welcome a discussion of where these directions align as both projects develop.

---
*Prepared with Codex and published at @yifanxuaaa’s request.*
