#!/usr/bin/env node

const { URL } = require("node:url");

const URL_ARGUMENT = process.argv[2];
const TIMEOUT_MS = Number(process.env.GOWASM_SMOKE_TIMEOUT_MS || 20000);

if (!URL_ARGUMENT) {
  console.error("usage: smoke-run.js <url>");
  process.exit(2);
}

let puppeteer;
for (const pkg of ["puppeteer", "puppeteer-core"]) {
  try {
    // eslint-disable-next-line global-require
    const candidate = require(pkg);
    puppeteer = candidate.default || candidate;
    break;
  } catch (err) {
    if (err.code !== "MODULE_NOT_FOUND") {
      throw err;
    }
  }
}

if (!puppeteer) {
  console.error("Puppeteer is required for this smoke check but was not found.");
  console.error("Install it with: npm i --save-dev puppeteer");
  process.exit(2);
}

const launchOptions = {
  headless: true,
  args: ["--no-sandbox", "--disable-dev-shm-usage", "--disable-gpu"],
};

if (process.env.GOWASM_CHROME_BIN) {
  launchOptions.executablePath = process.env.GOWASM_CHROME_BIN;
}

(async () => {
  let browser;
  const consoleErrors = [];
  const pageErrors = [];

  try {
    browser = await puppeteer.launch(launchOptions);
    const page = await browser.newPage();
    page.on("console", (msg) => {
      if (msg.type() === "error") {
        consoleErrors.push(msg.text());
      }
    });
    page.on("pageerror", (err) => pageErrors.push(String(err.message || err)));

    const response = await page.goto(URL_ARGUMENT, {
      waitUntil: "domcontentloaded",
      timeout: TIMEOUT_MS,
    });

    if (!response) {
      throw new Error("No response from navigation");
    }

    if (!response.ok()) {
      throw new Error(`Unexpected response ${response.status()} for ${URL_ARGUMENT}`);
    }

    await page.waitForFunction("document.readyState === 'complete'", { timeout: TIMEOUT_MS });

    const title = await page.title();
    const bodyHasText = await page.evaluate(() => {
      const body = document.body;
      return Boolean(body && body.textContent && body.textContent.trim().length > 0);
    });

    const resolvedUrl = new URL(response.url()).toString();

    if (!bodyHasText) {
      throw new Error("Page loaded without visible body text");
    }

    if (pageErrors.length > 0 || consoleErrors.length > 0) {
      if (pageErrors.length > 0) {
        console.error(`[smoke] page JS errors: ${pageErrors.join(", ")}`);
      }
      if (consoleErrors.length > 0) {
        console.error(`[smoke] console errors: ${consoleErrors.join(", ")}`);
      }
      throw new Error("Browser reported JavaScript errors");
    }

    console.log(`[smoke] URL: ${URL_ARGUMENT}`);
    console.log(`[smoke] resolved URL: ${resolvedUrl}`);
    console.log(`[smoke] status: ${response.status()}`);
    console.log(`[smoke] title: ${title || "<empty>"}`);
    console.log("[smoke] page check passed");
    process.exit(0);
  } catch (err) {
    console.error(`[smoke] failed: ${err.message}`);
    process.exit(1);
  } finally {
    if (browser) {
      await browser.close().catch(() => {});
    }
  }
})(); 
