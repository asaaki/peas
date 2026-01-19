+++
key = "tui-mouse-support"
tags = [
    "tui",
    "feature",
    "mouse",
]
created = "2026-01-19T23:53:07.243287100Z"
updated = "2026-01-19T23:53:07.243287100Z"
+++

Added comprehensive mouse support to the peas TUI:

- Click-to-select for tickets and memories in list view
- Mouse wheel scrolling for navigation (up/down)
- Mouse wheel scrolling in detail view for content
- Proper row calculation accounting for borders (row >= 2)
- Works seamlessly in both Tickets and Memory views
- Integrated into existing event loop with Event::Mouse handling

Implementation uses crossterm's MouseEvent with MouseEventKind::Down for clicks and ScrollUp/ScrollDown for wheel events. The handle_mouse_click method calculates the clicked row and updates selection state accordingly.
