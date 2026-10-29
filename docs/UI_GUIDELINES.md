# UI guidelines

Compact mode is 210 × 105 logical pixels. Expanded mode is 350 × 560. Use logical
dimensions for sizing and native physical coordinates for screen position.

Tokens live in `src/lib/styles/tokens.css`. The dark lavender surface, pink timer,
lilac text, locally bundled Inter Variable, and restrained borders define the identity.
The timer is the dominant element. Tasks scroll inside the widget; settings scroll
independently with Save always visible. Long task names truncate, with full-title tooltips.

Ten keyed dots show elapsed fractions. At zero, none is complete; at 100%, all are.
Only the current running dot pulses, once when it becomes current. CSS honors both
`prefers-reduced-motion` and the explicit app preference. Pausing removes the pulse.
Task progress is a separate bar with a count and accessible percentage.

Every icon button needs an accessible name, tooltip, and visible focus state. Task
editing and reordering must work by keyboard. List deletion requires an inline confirmation.
Native drag regions must exclude controls. Never add continuous blur or a full-screen
settings route. Test 100%, 125%, 150%, and 200% scale after layout changes.
