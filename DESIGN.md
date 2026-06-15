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

### A. Window Strategy: Compact Keystroke Overlay
- The current keystroke visualizer uses a compact, borderless, transparent click-through window instead of a maximized/fullscreen overlay.
- The window size is computed from runtime layout values and keystroke limits such as max history, max active text length, font sizes, text line-height, spacing, and padding.
- The default placement is bottom-left with a screen margin. Once the monitor size is known, the window is clamped to the monitor's available area inside that margin. Placement is not configurable yet, but the runtime layout values are structured so this can be added later.
- Future cursor-following mouse visualization may need a separate window or a different overlay strategy.

### B. Keystroke Grouping Algorithm
- Normal keys (alphanumeric and symbols) are appended to the *current active bubble*.
- Delimiters like `Space`, `Enter`, or `Tab` are appended to the same event row and then finalize the active typing bubble.
- **Inactivity Timeout**: If no keystroke occurs for `1` second, the current bubble is finalized. Subsequent typing starts a new bubble.
- **History Expiration**: Finalized bubbles disappear after `5` seconds. Duplicate key-only bubbles refresh their expiration when their repeat count increases.
- **Active Text Limit**: Active text is capped at 24 characters before it is split into a new history row. A delimiter may appear after that text as an extra bubble in the same row.
- **Backspace Handling**:
  - If a Backspace key is pressed and the current bubble is active and has characters, the last character is deleted from the bubble.
  - If the current bubble is empty, or the previous bubble was already finalized (by timeout/delimiters), a backspace symbol `⌫` is appended.
- **Modifiers & Shortcut commands**:
  - Keys combined with command-style modifiers (Control, Alt/Option, or Super/Command) immediately finalize the active typing bubble and display as one shortcut row.
  - Shortcut rows render each modifier/key as a separate bubble in the same row, e.g. `Super+Shift+S` appears as three adjacent bubbles.
  - A subtle modifier row is always visible at the bottom and highlights held modifiers.
- **Duplicate Compression**: Adjacent finalized key-only bubbles with the same content and kind are collapsed into one history entry with a small inline repeat count such as `×2` or `×3`. Text bubbles are not compressed.
- **Expiration Order**: History expiration times are monotonic because new bubbles append to the back and only the latest duplicate bubble can be refreshed. Expiration pruning only removes from the front of the queue.

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

---

## 6. Implementation Status

Current stage: **first vertical slice / keystroke visualizer**.

Completed:
- Compact transparent `iced` overlay window sized from layout and keystroke limits.
- Monitor-aware overlay clamping so the compact window does not extend beyond the available screen area.
- Always-on-top borderless window configuration.
- Mouse passthrough enablement after window creation.
- Global input hook subscription through `rdev`.
- Keystroke grouping with active text editing, delimiter rows, shortcut rows, and held modifier indicators.
- Adjacent duplicate bubble compression for repeated keys and shortcuts.
- Five-second expiration for finalized bubbles.
- Bottom-left bubble rendering using the dedicated icon font for keyboard glyphs and monospace text for typed text.

In progress / next:
- Manual verification of keystroke grouping behavior on the target OS.
- Mouse follower rendering and click/scroll feedback.
- Default hotkeys for visibility, clearing, and exit.
- Configuration persistence is intentionally deferred.
