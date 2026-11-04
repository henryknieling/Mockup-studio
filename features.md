# Mockup Studio — Feature Plan

## Product scope

- [x] Build a single-user website that runs locally on the user's device.
- [x] Use a simple, light visual style.
- [x] Organize work as projects containing multiple screens.
- [x] Let each screen use its own fixed viewport size.
- [x] Provide presets for mobile app and web screen sizes.

## Projects and screens

- [x] Create, rename, and delete projects.
- [x] Create, rename, reorder, and delete screens within a project.
- [x] Configure a screen's viewport dimensions and device-frame type.

## Visual editor

- [x] Provide a canvas for composing app and website mockups.
- [x] Provide mobile and web UI component libraries.
- [x] Add and select components on the canvas.
- [x] Move and resize components directly on the canvas.
- [x] Edit selected component properties in an inspector panel.
- [x] Show the screen's elements in a layers panel.
- [x] Support undo and redo for editor changes.
- [x] Include the basic elements needed to compose screens, such as text, buttons, images, inputs, cards, and navigation.
- [x] Keep JavaScript focused on interactive editor behavior; use server-rendered pages for the rest of the site.

## Saving data

- [x] Run PostgreSQL locally using Docker Compose.
- [x] Persist projects, screens, viewport settings, and screen elements in PostgreSQL.
- [x] Use database migrations and keep the stored screen document versioned.

## PDF export

- [x] Export a screen as a PDF with one screen per page.
- [x] Fit each PDF page to the screen's aspect ratio.
- [x] Include the selected device frame in the PDF.
- [x] Support exporting either one screen or all screens in a project.
- [x] Keep exported text and graphics crisp at print resolution.
- [x] Prototype and select a PDF-generation approach early to verify output quality.

## Local development workflow

- [x] Add a `just run` command that starts Docker Compose in the background, starts the web server, and displays the local URL.
- [x] Add a command to stop the Docker Compose services.
- [x] Add commands to apply and inspect database migrations.
- [x] Add convenient commands for formatting, tests, and code checks.

## Quality checks

- [ ] Test core project and screen behavior.
- [ ] Test database persistence and migrations.
- [x] Add a PDF export smoke test.
- [ ] Ensure the documented local setup works from a fresh checkout.

## Explicitly out of scope for the first version

- AI-generated layouts.
- Importing project files.
- User accounts, collaboration, and hosted multi-user service.
