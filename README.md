# reStrike VTA

Overlay and automation toolkit for VTA using Tauri and React.

## Project Overview
reStrike VTA is designed to provide a modern overlay and automation solution for VTA, integrating UDP ingestion, OBS control, and license management.

## Directory Structure
```
reStrike_VTA/
├── src/            # Application source code
├── tests/          # Unit and integration tests
├── public/         # Static assets (if applicable)
├── scripts/        # Automation scripts (build, deploy)
├── .github/
│   ├── workflows/  # CI/CD workflows
│   └── ISSUE_TEMPLATE/  # GitHub issue templates
├── docs/           # Design docs and API specs
├── LICENSE
├── package.json    # Dependencies and scripts (Node.js/Tauri)
└── README.md
```

## Installation
1. Clone the repository:
   ```bash
   git clone https://github.com/damjanZGB/reStrike_VTA.git
   cd reStrike_VTA
   ```
2. Install dependencies:
   ```bash
   npm install
   ```
3. Copy `.env.example` to `.env` and fill in required values.

## Usage
1. Start the development server:
   ```bash
   npm run start
   ```
2. Run tests:
   ```bash
   npm test
   ```

## Contributing
1. Fork the repo and create your branch.
2. Submit a pull request with a clear description.
3. Follow the issue templates for bug reports and feature requests.

## License
MIT
# reStrike_VTA