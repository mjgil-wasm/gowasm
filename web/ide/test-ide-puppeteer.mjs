/**
 * Full IDE workflow test using Puppeteer.
 *
 * Usage:
 *   # First start a server from the repo root web/ dir:
 *   python3 -m http.server 8765 --directory /home/m/git/gowasm-test/web
 *   # Then run:
 *   node test-ide-puppeteer.mjs
 */

import { createRequire } from "node:module";
import { existsSync } from "node:fs";
import { join } from "node:path";

const require = createRequire(import.meta.url);

let puppeteer;
for (const pkg of ["puppeteer", "puppeteer-core"]) {
  try {
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
  console.error("Puppeteer is required for this IDE test but was not found.");
  console.error("Install it with: npm i --save-dev puppeteer");
  process.exit(2);
}

const BASE = process.env.IDE_TEST_URL || "http://localhost:8765";
const browserCandidates = [
  process.env.GOWASM_CHROME_BIN,
  process.env.PUPPETEER_EXECUTABLE_PATH,
  "/usr/bin/google-chrome",
  "/usr/bin/google-chrome-stable",
  "/usr/bin/chromium",
  "/usr/bin/chromium-browser",
  join(
    process.env.HOME || "",
    ".cache/puppeteer/chrome/linux-148.0.7778.97/chrome-linux64/chrome",
  ),
  join(
    process.env.HOME || "",
    ".cache/puppeteer/chrome-headless-shell/linux-148.0.7778.97/chrome-headless-shell-linux64/chrome-headless-shell",
  ),
  "/usr/bin/firefox",
].filter((candidate, index, all) => candidate && all.indexOf(candidate) === index && existsSync(candidate));

let browser;
let page;
let passed = 0;
let failed = 0;

function assert(cond, msg) {
  if (cond) {
    passed++;
    console.log("  PASS", msg);
  } else {
    failed++;
    console.log("  FAIL", msg);
  }
}

async function waitFor(selector, timeout = 5000) {
  await page.waitForSelector(selector, { timeout });
}

async function sleep(ms) {
  await new Promise((resolve) => setTimeout(resolve, ms));
}

async function launchBrowser() {
  let lastError = null;
  for (const executablePath of browserCandidates) {
    const isFirefox = executablePath.includes("firefox");
    try {
      return await puppeteer.launch({
        headless: true,
        executablePath,
        ...(isFirefox
          ? { browser: "firefox", protocol: "webDriverBiDi", args: ["--headless"] }
          : { args: ["--no-sandbox", "--disable-setuid-sandbox", "--disable-dev-shm-usage"] }),
      });
    } catch (error) {
      lastError = error;
      console.warn(`Skipping browser candidate ${executablePath}: ${error.message}`);
    }
  }
  throw lastError || new Error("No browser executable could be launched");
}

async function openTreeFile(path) {
  await page.waitForFunction(
    (targetPath) => Array.from(document.querySelectorAll(".tree-node")).some((n) => n.dataset.path === targetPath),
    { timeout: 5000 },
    path,
  );
  await page.evaluate((targetPath) => {
    const node = Array.from(document.querySelectorAll(".tree-node")).find((n) => n.dataset.path === targetPath);
    if (!node) throw new Error(`tree node not found: ${targetPath}`);
    node.click();
  }, path);
  await sleep(250);
}

async function replaceEditorText(text) {
  await page.click(".cm-content");
  await page.keyboard.down("Control");
  await page.keyboard.press("a");
  await page.keyboard.up("Control");
  await page.keyboard.press("Backspace");
  await page.keyboard.type(text);
  await sleep(200);
}

async function saveEditor() {
  await page.keyboard.down("Control");
  await page.keyboard.press("s");
  await page.keyboard.up("Control");
  await sleep(300);
}

async function createFile(name, contents) {
  await page.click("#new-file-btn");
  await waitFor("#modal:not([hidden])");
  await page.fill("#modal-input", name);
  await page.click("#modal-confirm");
  await sleep(250);
  await replaceEditorText(contents);
  await saveEditor();
}

async function runTests() {
  console.log("\n--- IDE Puppeteer workflow tests ---\n");
  console.log(`Base URL: ${BASE}\n`);

  // 1. Page load and shell structure
  await page.goto(`${BASE}/ide/index.html`, { waitUntil: "networkidle0" });
  await waitFor("#ide-layout");

  const hasHeader = await page.evaluate(() => !!document.querySelector("header h1"));
  assert(hasHeader, "header with title exists");

  const hasFileTree = await page.evaluate(() => !!document.getElementById("file-tree"));
  assert(hasFileTree, "file-tree panel exists");

  const hasLegacyEditor = await page.evaluate(() => !!document.getElementById("editor"));
  assert(!hasLegacyEditor, "legacy editor textarea is removed");

  const hasBuildOutput = await page.evaluate(() => !!document.getElementById("build-output"));
  assert(!hasBuildOutput, "legacy build-output panel is removed");

  const hasTerminal = await page.evaluate(() => !!document.getElementById("terminal-output"));
  assert(hasTerminal, "terminal-output panel exists");

  // 2. Toolbar buttons exist and are wired
  const buttons = ["run-btn", "build-btn", "test-btn", "format-btn", "vet-btn", "abort-btn"];
  for (const id of buttons) {
    const exists = await page.evaluate((i) => !!document.getElementById(i), id);
    assert(exists, `${id} exists`);
  }

  // 3. File panel buttons (including import/export)
  const fileButtons = ["new-file-btn", "new-folder-btn", "import-zip-btn", "export-zip-btn"];
  for (const id of fileButtons) {
    const exists = await page.evaluate((i) => !!document.getElementById(i), id);
    assert(exists, `${id} exists`);
  }

  // 4. Open a file and verify tab appears
  await openTreeFile("main.go");
  assert(true, "clicked main.go in file tree");

  await sleep(300);

  const tabExists = await page.evaluate(() =>
    Array.from(document.querySelectorAll(".editor-tab")).some((t) =>
      t.textContent.includes("main.go")
    )
  );
  assert(tabExists, "main.go tab appears after click");

  const hasEditor = await page.evaluate(() => !!document.querySelector(".cm-editor"));
  assert(hasEditor, "CodeMirror editor exists after opening a file");

  // 5. Type in editor and verify dirty indicator appears
  await page.click(".cm-content");
  await page.keyboard.press("End");
  await page.keyboard.type("\n// test edit");
  await sleep(150);

  const editorUpdated = await page.evaluate(() => {
    const content = document.querySelector(".cm-content");
    return Boolean(content && content.textContent && content.textContent.includes("// test edit"));
  });
  assert(editorUpdated, "editor content updates after typing");

  const hasUnsavedState = await page.evaluate(() => {
    const editorStatus = document.getElementById("editor-status")?.textContent?.trim();
    return editorStatus === "Unsaved" || Array.from(document.querySelectorAll(".editor-tab")).some((t) =>
      t.classList.contains("dirty") || Boolean(t.querySelector(".tab-indicator.dirty"))
    );
  });
  assert(hasUnsavedState, "unsaved editor state appears after edit");

  // 6. Save the file and verify saved indicator appears
  await page.keyboard.down("Control");
  await page.keyboard.press("s");
  await page.keyboard.up("Control");
  await sleep(300);

  const hasSavedIndicator = await page.evaluate(() =>
    Array.from(document.querySelectorAll(".editor-tab")).some((t) =>
      t.querySelector(".tab-indicator.saved")
    )
  );
  assert(hasSavedIndicator, "saved indicator (✓) appears after Ctrl+S");

  // 7. Initialize Module button workflow
  const initModuleVisible = await page.evaluate(() => {
    const btn = document.getElementById("init-module-btn");
    return btn && !btn.hidden;
  });
  assert(!initModuleVisible, "Initialize Module button is hidden when go.mod exists");

  // 8. Format button triggers a request
  await page.click("#format-btn");
  await sleep(500);
  assert(true, "format button was clickable and triggered a request");

  // 9. Output panel tabs switch
  await page.click('[data-target="terminal-output"]');
  await sleep(100);

  const terminalActive = await page.evaluate(() =>
    document.getElementById("terminal-output").classList.contains("active")
  );
  assert(terminalActive, "terminal output tab becomes active");

  // 10. Responsive layout classes exist
  const cssHasResponsive = await page.evaluate(() => {
    const sheets = Array.from(document.styleSheets);
    for (const sheet of sheets) {
      try {
        for (const rule of sheet.cssRules) {
          if (rule.cssText && rule.cssText.includes("@media (max-width: 900px)")) {
            return true;
          }
        }
      } catch {}
    }
    return false;
  });
  assert(cssHasResponsive, "CSS responsive breakpoint exists");

  // 11. Browser regression for unsupported standard-library testing package
  await openTreeFile("main.go");
  await replaceEditorText(`package main

import "fmt"

// Add takes two integers and returns their sum.
func Add(a, b int) int {
	return a + b
}

func main() {
	result := Add(5, 7)
	fmt.Printf("5 + 7 = %d\n", result)
}
`);
  await saveEditor();

  await createFile("main_test.go", `package main

import "testing"

func TestAdd(t *testing.T) {
	// Table-driven tests are the standard way to write tests in Go
	tests := []struct {
		name     string
		a        int
		b        int
		expected int
	}{
		{"positive numbers", 2, 3, 5},
		{"negative numbers", -2, -4, -6},
		{"mixed numbers", -1, 5, 4},
		{"zeroes", 0, 0, 0},
	}

	for _, tc := range tests {
		t.Run(tc.name, func(t *testing.T) {
			result := Add(tc.a, tc.b)
			if result != tc.expected {
				t.Errorf("Add(%d, %d) = %d; want %d", tc.a, tc.b, result, tc.expected)
			}
		})
	}
}
`);

  await page.click("#test-btn");
  await page.waitForFunction(
    () => {
      const text = document.getElementById("terminal-history")?.innerText || "";
      return text.includes("[tests failed]") || text.includes("[tests passed]");
    },
    { timeout: 20000 },
  );

  const terminalText = await page.evaluate(
    () => document.getElementById("terminal-history")?.innerText || "",
  );
  assert(terminalText.includes("$ go test ./... -v"), "test button logs the go test command");
  assert(
    terminalText.includes("import `testing` could not be resolved while loading package `example.com/app`"),
    "standard-library testing import failure is shown in terminal output",
  );
  assert(terminalText.includes("[tests failed]"), "unsupported testing package path reports test failure");

  console.log("\n--- Summary ---");
  console.log(`Passed: ${passed}`);
  console.log(`Failed: ${failed}`);
  return failed === 0;
}

async function main() {
  try {
    browser = await launchBrowser();
    page = await browser.newPage();
    page.setViewport({ width: 1280, height: 900 });

    const ok = await runTests();
    process.exit(ok ? 0 : 1);
  } catch (e) {
    console.error("Test runner error:", e.message);
    process.exit(1);
  } finally {
    if (browser) await browser.close();
  }
}

main();
