# Browser Smoke Check

This repository includes a simple browser smoke flow to validate that the app is
reachable at the URL printed by `run.sh`.

- `scripts/smoke-run.sh` starts `run.sh`, captures the displayed URL, and runs a
  browser smoke check against it.
- `scripts/smoke-run.js` performs Puppeteer validation of the loaded page.

Expected success checks:
- The target URL responds with a successful HTTP status.
- The page reaches `document.readyState === "complete"`.
- The page has body text content.
- No JavaScript or browser console errors are emitted.

When Puppeteer is unavailable, the Node smoke script exits with code `2` and prints
an install hint.
