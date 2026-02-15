# 🛡️ Terminal Sanctum

**Terminal Sanctum** is a high-fidelity, Warcraft-inspired productivity TUI (Terminal User Interface). Designed as a **persistent safehouse for developers**—much like the serene Majula in *Dark Souls*—it provides a sanctuary of respite where your coding consistency is rewarded with legendary hero progression.

---

## 🌟 The Product Experience

Terminal Sanctum lives alongside your development workflow. It doesn't interfere; it observes. By passively detecting Git activity in your workspace, it fuels the growth of your summoned Hero.

### Key Features
*   **Legendary Roster:** Summon from 30+ canonical Warcraft 3 heroes (Thrall, Arthas, Sylvanas, Illidan, etc.).
*   **Immersive Wit:** Every hero features unique, personality-driven dialogue that blends iconic lore with development humor.
*   **Persistent Safehouse:** Unlike typical session-based tools, your **Objectives** and Hero progress are permanently stored. They survive switches, restarts, and reboots.
*   **Ergonomic Control:** Optimized for speed. Emotes and selections are mapped to the home row (`ASDFGH`) for zero-friction interaction.
*   **Majula Atmosphere:** A minimalist, aesthetic UI designed to provide a calm "savepoint" within the chaos of complex projects.

---

## 🏗️ Architecture & Safety

The Sanctum is built on a **Principal Seniority** architecture that prioritizes the safety of your workspace.

### 1. The "Ghost" Pattern (Zero-Impact)
The Sanctum is a strictly **passive observer**. 
*   **No Git Commands:** It never executes `git` binaries or shell commands in your project.
*   **No File Modification:** It never opens, reads, or modifies your source code or Git internals.
*   **Filesystem Events:** Detection is handled by matching system-level path events, ensuring 100% isolation from your work.

### 2. Library/Binary Hybrid Model
The application is structured into two distinct layers:
*   **`sanctum_core` (Library):** Contains all Hero logic, XP math, persistence engines, and TUI rendering components.
*   **`sanctum` (Binary):** A thin entry point that links the library to your terminal's I/O.
*   **Total Coverage:** This model allows 100% of the logic and UI rendering to be verified through automated unit, integration, and visual regression tests.

---

## 🚀 Setup & Installation

### Prerequisites
*   **Rust Toolchain:** Ensure you have `cargo` installed (Standard via [rustup.rs](https://rustup.rs)).

### Quick Install
1.  **Clone the Repository:**
    ```bash
    git clone https://github.com/aditya-parab/terminal-sanctum.git
    cd terminal-sanctum
    ```

2.  **Run the Deployment Script:**
    ```bash
    ./install.sh
    ```
    *This script automatically purges caches, validates code standards, executes the 40+ test battery, and installs the `sanctum` command globally.*

---

## 🎮 Basic Controls

Launch your safehouse from any project directory:
```bash
sanctum
```

*   **`a`**: Add a **Persistent Objective**.
*   **`e`**: Open **Emote Menu** (Use `ASDFGH` to select).
*   **`u`**: Trigger **System Overclock** (Spend focus for 3x XP).
*   **`s`**: Return to the **Hall of Heroes** (Switch Hero).
*   **`d`**: **Delete** a character profile (In the Hall of Heroes).
*   **`Space`**: Resolve an objective.
*   **`q`**: Save and exit the sanctuary.

## 🧪 Technical Integrity
Terminal Sanctum is developed with a focus on professional standards and operational stability:
*   **Zero-Warning Builds:** No clippy warnings or linter errors allowed.
*   **Verified Documentation:** Every core API is tested directly via its documentation examples.
*   **Visual Auditing:** Automated tests verify that UI components are rendered correctly to the buffer.

---

## 📜 License
This project is for personal productivity and immersive development. Warcraft 3 is a trademark of Blizzard Entertainment. Dialogue and personalities are inspired by the Warcraft universe.
