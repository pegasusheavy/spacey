# Spacey Documentation Website

This is the documentation website for the Spacey JavaScript engine, built with Angular 21 and Tailwind CSS.

## Live Site

Visit the documentation at: **https://pegasusheavy.github.io/spacey/**

## Development

### Prerequisites

- Node.js 20+
- pnpm 9+

### Setup

```bash
cd docs
pnpm install
```

### Development Server

```bash
pnpm start
```

Navigate to `http://localhost:4200/`. The app will automatically reload when you change any of the source files.

### Build

```bash
# Production build
pnpm build

# GitHub Pages build (with correct base href)
pnpm build:ghpages
```

The build artifacts will be stored in the `dist/spacey/browser/` directory.

## Deployment

The website is automatically deployed to GitHub Pages when changes are pushed to the `main` or `develop` branch. The deployment is handled by the `.github/workflows/docs.yml` workflow.

### Manual Deployment

If you need to deploy manually:

1. Build the project with the GitHub Pages configuration:
   ```bash
   pnpm build:ghpages
   ```

2. The contents of `dist/spacey/browser/` can be deployed to any static hosting service.

## Technology Stack

- **Angular 21** - Modern web framework
- **Tailwind CSS 4** - Utility-first CSS framework
- **TypeScript** - Type-safe JavaScript

## License

MPL-2.0 - See the [LICENSE](../LICENSE) file in the root of the repository.
