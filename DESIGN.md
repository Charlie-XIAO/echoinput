# EchoInput Design Document

`echoinput` is a keystroke and mouse event visualizer written in Rust using the `iced` GUI library. It aims to provide real-time, aesthetically pleasing visual feedback of user inputs (keys and clicks) for screencasts, presentations, and tutorials.

---

## 1. Overview & Core Features

- **Keystroke Visualizer**: Displays a history of the most recent keystrokes. Grouping logic ensures typing is aggregated into readable words/sentences rather than flooding the screen key-by-key.
- **Mouse Event Visualizer**: A visual representation of a mouse following the cursor. Left/right clicks glow/light up, and scrolls display direction arrows.
- **Always-on-Top & Transparent Overlay**: The app runs borderless, transparent, and sits on top of all other windows, ignoring clicks (mouse passthrough enabled) so it does not interfere with the user's work.
- **Global Input Hook**: Since it sits in the background, it captures input events globally (even when not focused) using a dedicated worker thread.
- **Hotkeys**: Configurable hotkeys to toggle display states or exit since there is no tray icon support in basic `iced` yet.

---

## 2. Architecture & Tech Stack

```mermaid
graph TD
    subgraph Host OS
        A[Global Input Hook: rdev] -- Event Stream --> B[Background Thread]
    end
    subgraph Iced Application
        B -- Channel --> C[Iced Subscription / Stream]
        C -- Messages --> D[Iced Update Loop]
        D -- State Update --> E[State Management]
        E -- Render --> F[Single Fullscreen Overlay (Keystrokes + Mouse Follower)]
    end
```

### Key Dependencies
1. **`iced` (v0.14)**: For rendering the overlay windows with transparency, borderless style, and mouse passthrough.
2. **`rdev`**: For establishing a global input hook across mouse and keyboard events (Linux X11, macOS, Windows).
3. **`tokio` / `futures`**: For async channel communication between the hook thread and the `iced` event loop.

---

## 3. Detailed Component Designs

### A. Window Strategy: Single Fullscreen Overlay
- A single, borderless, transparent click-through window covering the entire screen is spawned.
- Both the keystroke visualizer list (positioned at a fixed area on the screen, e.g., bottom-left) and the mouse event follower (drawn at the cursor's current logical `(x, y)` coordinates relative to the fullscreen canvas) are rendered within this single window.
- This design eliminates window-moving latency/flicker and provides a unified rendering canvas.

### B. Keystroke Grouping Algorithm
- Normal keys (alphanumeric and symbols) are appended to the *current active bubble*.
- Delimiters like `Space`, `Enter`, or `Tab` terminate the current active bubble, starting a new one.
- **Inactivity Timeout**: If no keystroke occurs for `0.5` seconds, the current bubble is finalized. Subsequent typing starts a new bubble.
- **Backspace Handling**:
  - If a Backspace key is pressed and the current bubble is active and has characters, the last character is deleted from the bubble.
  - If the current bubble is empty, or the previous bubble was already finalized (by timeout/delimiters), a backspace symbol `⌫` is appended.
- **Modifiers & Shortcut commands**:
  - Keys combined with modifiers (e.g., `Ctrl+C`, `Alt+Tab`, `Super+Shift+A`) are instantly displayed in a separate, dedicated bubble and immediately finalize the active typing bubble.

### C. Mouse Event Follower
- Tracks global mouse pointer coordinates via `rdev`.
- A modern, semi-transparent mouse silhouette is drawn on the overlay at the cursor position.
- **Visual Feedback**:
  - Left click button area glows bright neon blue (e.g., `#00d2ff`) with a pulse effect.
  - Right click button area glows bright neon pink (e.g., `#ff007f`).
  - Scroll wheel area lights up showing a directional arrow (↑ or ↓) depending on the scroll direction, which fades out after 300ms.

---

## 4. Default Hotkeys

Since there is no system tray support in our iced setup:
- `Super + Shift + K`: Toggle Keystroke Visualizer visibility.
- `Super + Shift + M`: Toggle Mouse Follower visibility.
- `Super + Shift + C`: Clear current keystroke history.
- `Super + Shift + Q`: Exit EchoInput.

---

## 5. Target OS Support

- **Linux**: Targets X11 desktop environments (requires Xlib development headers for building).
- **Windows / macOS**: Cross-platform support is designed out of the box through `rdev`'s native platform event hooks.

