
## DockBar and StatusbarDock Layout Update (2024-06)

- DockBar and StatusbarDock are now visually connected with no gap.
- DockBar uses `flex-1` and `min-h-0` on the row containing SidebarSmall and SidebarBig.
- SidebarSmall no longer has a fixed height.
- This ensures the DockBar content stretches fully to the bottom, and the status bar is always flush with the DockBar.
- All future UI and layout work should reference this structure for vertical alignment and sidebar stretching. 