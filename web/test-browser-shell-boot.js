import {
  click,
  control,
  unloadShellFrame,
  waitFor,
  waitForShellReady,
} from "./test-browser-shell-harness.js";

export async function testBrowserShellBootUrls({ assert, frame, log }) {
  log("\n--- browser shell boot URL harness ---");

  try {
    const manualManifestUrl = createBootManifestUrl({
      version: 1,
      files: [
        {
          path: "go.mod",
          contents: "module example.com/manual\n\ngo 1.21\n",
        },
        {
          path: "cmd/app/main.go",
          contents: `package main

import "fmt"

func main() {
\tfmt.Println("manual-consent")
}
`,
        },
      ],
      entry_path: "main.go",
    });
    let doc = await loadBootFrame(
      frame,
      `boot_manifest_url=${encodeURIComponent(manualManifestUrl)}&boot_entry_path=${encodeURIComponent("cmd/app/main.go")}`,
    );
    await waitForShellReady(doc);
    assert(
      control(doc, "boot-url-status").textContent.includes("awaiting explicit load consent")
        && control(doc, "source").value.includes("worker shell"),
      "browser shell boot URL waits for explicit consent before fetching a remote manifest",
      shellSnapshot(doc),
    );
    click(doc, "boot-url-load-button");
    await waitFor(
      () =>
        control(doc, "status").textContent === "Boot manifest loaded"
          && control(doc, "entry-path-input").value === "cmd/app/main.go"
          && control(doc, "source").value.includes("manual-consent"),
      "manual boot-manifest load completed",
      doc,
    );
    assert(
      control(doc, "output").textContent.includes("Loaded boot manifest")
        && control(doc, "entry-path-input").value === "cmd/app/main.go",
      "browser shell boot URL applies entry-path overrides after explicit consent",
      shellSnapshot(doc),
    );

    await unloadShellFrame(frame);

    const modulePath = "example.com/boot/remote";
    const moduleVersion = "v1.4.2";
    doc = await loadBootFrame(
      frame,
      [
        `boot_manifest_url=${encodeURIComponent(
          createBootManifestUrl({
            version: 1,
            revision: "rev-run",
            files: [
              {
                path: "go.mod",
                contents: "module example.com/bootrun\n\ngo 1.21\n",
              },
              {
                path: "main.go",
                contents: `package main

import (
\t"fmt"
\t"${modulePath}/greeter"
)

func main() {
\tfmt.Println(greeter.Message())
}
`,
              },
            ],
            module_roots: [
              {
                module_path: modulePath,
                version: moduleVersion,
                fetch_url: createModuleBundleFetchUrl({
                  modulePath,
                  version: moduleVersion,
                  message: "boot-remote",
                }),
              },
            ],
          }),
        )}`,
        "boot_consent=1",
        "boot_action=run",
      ].join("&"),
    );
    await waitFor(
      () =>
        control(doc, "status").textContent === "Worker responded"
          && control(doc, "output").textContent.includes("boot-remote")
          && control(doc, "module-status").textContent.includes(`${modulePath}@${moduleVersion}`),
      "auto-consented boot-manifest run completed",
      doc,
    );
    assert(
      control(doc, "output").textContent.includes("boot-remote")
        && control(doc, "module-status").textContent.includes(`${modulePath}@${moduleVersion}`),
      "browser shell boot URL loads module roots and runs the requested action after explicit consent",
      shellSnapshot(doc),
    );

    await unloadShellFrame(frame);

    doc = await loadBootFrame(
      frame,
      [
        `boot_manifest_url=${encodeURIComponent(
          createBootManifestUrl({
            version: 1,
            revision: "rev-tests",
            files: [
              {
                path: "go.mod",
                contents: "module example.com/boottest\n\ngo 1.21\n",
              },
              {
                path: "calc/calc.go",
                contents: `package calc

func Greeting() string {
\treturn "boot-test"
}
`,
              },
              {
                path: "calc/calc_test.go",
                contents: `package calc

func TestBootTarget() {
\tif Greeting() != "boot-test" {
\t\tpanic("unexpected greeting")
\t}
}
`,
              },
            ],
          }),
        )}`,
        "boot_consent=1",
        `boot_package_target=${encodeURIComponent("calc/calc.go")}`,
        "boot_action=test_package",
      ].join("&"),
    );
    await waitFor(
      () =>
        control(doc, "status").textContent.includes("Package tests passed")
          && control(doc, "package-target-input").value === "calc/calc.go"
          && control(doc, "output").textContent.includes("TestBootTarget"),
      "boot-manifest package-test target completed",
      doc,
    );
    assert(
      control(doc, "package-target-input").value === "calc/calc.go"
        && control(doc, "output").textContent.includes("Completed: TestBootTarget"),
      "browser shell boot URL applies package-test target overrides and test actions",
      shellSnapshot(doc),
    );

    await unloadShellFrame(frame);

    doc = await loadBootFrame(
      frame,
      `boot_manifest_url=${encodeURIComponent("data:application/json,%7Bbroken-json")}&boot_consent=1`,
    );
    await waitFor(
      () =>
        control(doc, "output").textContent.includes("response was not valid JSON"),
      "invalid boot-manifest failure surfaced",
      doc,
    );
    assert(
      control(doc, "output").textContent.includes("response was not valid JSON"),
      "browser shell boot URL surfaces malformed-manifest failures",
      shellSnapshot(doc),
    );

    await unloadShellFrame(frame);

    doc = await loadBootFrame(
      frame,
      [
        `boot_manifest_url=${encodeURIComponent(
          createBootManifestUrl({
            version: 1,
            revision: "rev-old",
            files: [
              {
                path: "go.mod",
                contents: "module example.com/stale\n\ngo 1.21\n",
              },
              {
                path: "main.go",
                contents: "package main\n",
              },
            ],
          }),
        )}`,
        "boot_consent=1",
        "boot_manifest_revision=rev-new",
      ].join("&"),
    );
    await waitFor(
      () =>
        control(doc, "output").textContent.includes("Boot manifest is stale"),
      "stale boot-manifest failure surfaced",
      doc,
    );
    assert(
      control(doc, "output").textContent.includes("expected revision rev-new"),
      "browser shell boot URL rejects stale manifest revisions",
      shellSnapshot(doc),
    );
  } finally {
    await unloadShellFrame(frame);
  }
}

async function loadBootFrame(frame, query) {
  frame.src = `./index.html?${query}&browser-shell-boot=${Date.now()}`;
  await waitForFrameLoad(frame);
  return frame.contentDocument;
}

async function waitForFrameLoad(frame) {
  await new Promise((resolve, reject) => {
    const timer = self.setTimeout(() => {
      frame.removeEventListener("load", onLoad);
      reject(new Error("timed out waiting for browser shell boot frame load"));
    }, 20000);

    function onLoad() {
      self.clearTimeout(timer);
      resolve();
    }

    frame.addEventListener("load", onLoad, { once: true });
  });
}

function createBootManifestUrl(manifest) {
  return `data:application/json,${encodeURIComponent(JSON.stringify(manifest))}`;
}

function createModuleBundleFetchUrl({ modulePath, version, message }) {
  return `data:application/json,${encodeURIComponent(
    JSON.stringify({
      module: {
        module_path: modulePath,
        version,
      },
      files: [
        {
          path: "go.mod",
          contents: `module ${modulePath}\n\ngo 1.21\n`,
        },
        {
          path: "greeter/greeter.go",
          contents: `package greeter

func Message() string {
\treturn "${message}"
}
`,
        },
      ],
    }),
  )}`;
}

function shellSnapshot(doc) {
  return JSON.stringify(
    {
      boot_status: control(doc, "boot-url-status").textContent,
      entry_path: control(doc, "entry-path-input").value,
      module_status: control(doc, "module-status").textContent,
      output: control(doc, "output").textContent,
      package_target: control(doc, "package-target-input").value,
      status: control(doc, "status").textContent,
    },
    null,
    2,
  );
}
