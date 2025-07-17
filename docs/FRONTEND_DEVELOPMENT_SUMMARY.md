## Diagnostics & Logs Manager

- Logging and Download Logs sections are now side by side (horizontal layout) above the Live Data section.
- Download Logs uses a table for log files; double-clicking a row downloads the file. Dropdown filters the table.
- Live Data is below, with a scrollable, auto-scrolling rich text area for live stream data.
- All sections follow atomic design and are ready for backend integration. 
- All logs are now saved in a dedicated 'log' folder in the installation directory. The folder is created automatically if missing. Log file is 'log/backend.log'. 