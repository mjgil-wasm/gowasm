const source = document.querySelector("#source");
const output = document.querySelector("#output");
const status = document.querySelector("#status");
const runButton = document.querySelector("#run-button");

source.value = `package main
import "fmt"

func helper() {
    fmt.Println("worker shell")
}

func main() {
    helper()
    fmt.Println("Rust engine next")
}
`;

const worker = new Worker("./engine-worker.js", { type: "module" });

worker.addEventListener("message", ({ data }) => {
  switch (data.kind) {
    case "ready":
      status.textContent = `Worker ready: ${data.info.engine_name}`;
      break;
    case "run_result":
      status.textContent = "Worker responded";
      output.textContent = data.stdout;
      break;
    case "diagnostics":
      status.textContent = "Diagnostics received";
      output.textContent = data.diagnostics.map((item) => item.message).join("\n");
      break;
    case "fatal":
      status.textContent = "Worker failed";
      output.textContent = data.message;
      break;
    case "cancelled":
      status.textContent = "Worker cancelled";
      output.textContent = "Execution cancelled.";
      break;
    default:
      status.textContent = "Unknown worker message";
      output.textContent = JSON.stringify(data, null, 2);
      break;
  }
});

worker.postMessage({ kind: "boot" });

runButton.addEventListener("click", () => {
  status.textContent = "Sending run request…";
  output.textContent = "";
  worker.postMessage({
    kind: "run",
    entry_path: "main.go",
    files: [{ path: "main.go", contents: source.value }],
  });
});
