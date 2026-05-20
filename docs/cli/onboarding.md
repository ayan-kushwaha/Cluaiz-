# Onboarding

The Cluaiz setup sequence is a deterministic onboarding ritual that configures local node parameters and establishes the local workspace sandbox.

---

## Workspace Initialization

At startup, the bootstrapper scans the host environment to verify the existence of the Cluaiz directory:

*   **Config Folder:** Seeding the active workspace directory at `~/.cluaiz/workspace/`.
*   **Node Identity Files:** Creating files like `IDENTITY.md` (Node characteristics), `USER.md` (Operator configurations), and `SOUL.md` (Weight biases) to establish persistent memory.

---

## The Sentinel Mechanism

To prevent corrupted configurations if the initial setup is forcefully exited or interrupted:

*   **Ignition Lock:** The setup generates a temporary `.ignition_lock` sentinel file inside the workspace.
*   **Resume Capability:** If the onboarding process is interrupted, next boot detects the `.ignition_lock` file and immediately resumes the interview at the last incomplete phase.
*   **Clean Up:** Once setup finishes and files are written to disk, the sentinel file is permanently deleted.

---

## Privacy Handshake

The setup explicitly prints a privacy and security layout:

*   **Local Boundary:** Verification that all processing runs entirely locally on local silicon.
*   **Owner Authorization:** Verifying that data leaves the node only upon explicit manual configuration by the operator.
