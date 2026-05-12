const DB_NAME = "gowasm-ide-fs";
const DB_VERSION = 1;
const STORE_NAME = "handles";

let dbPromise = null;

function openDB() {
  if (dbPromise) return dbPromise;
  dbPromise = new Promise((resolve, reject) => {
    const req = indexedDB.open(DB_NAME, DB_VERSION);
    req.onerror = () => reject(req.error);
    req.onsuccess = () => resolve(req.result);
    req.onupgradeneeded = () => {
      req.result.createObjectStore(STORE_NAME);
    };
  });
  return dbPromise;
}

export async function saveHandle(handle) {
  const db = await openDB();
  return new Promise((resolve, reject) => {
    const tx = db.transaction(STORE_NAME, "readwrite");
    const store = tx.objectStore(STORE_NAME);
    const req = store.put(handle, "workspace");
    req.onsuccess = () => resolve();
    req.onerror = () => reject(req.error);
  });
}

export async function loadHandle() {
  const db = await openDB();
  return new Promise((resolve, reject) => {
    const tx = db.transaction(STORE_NAME, "readonly");
    const store = tx.objectStore(STORE_NAME);
    const req = store.get("workspace");
    req.onsuccess = () => resolve(req.result ?? null);
    req.onerror = () => reject(req.error);
  });
}

export async function verifyPermission(handle, readWrite = false) {
  const options = { mode: readWrite ? "readwrite" : "read" };
  if ((await handle.queryPermission(options)) === "granted") {
    return true;
  }
  if ((await handle.requestPermission(options)) === "granted") {
    return true;
  }
  return false;
}

export async function pickDirectory() {
  if (typeof showDirectoryPicker !== "function") {
    return null;
  }
  const handle = await showDirectoryPicker();
  await saveHandle(handle);
  return handle;
}

export async function restoreDirectory() {
  const handle = await loadHandle();
  if (!handle) return null;
  const ok = await verifyPermission(handle, true);
  return ok ? handle : null;
}

export async function listDir(handle, path = "") {
  const entries = [];
  for await (const [name, child] of handle.entries()) {
    const kind = child.kind;
    entries.push({ name, kind, handle: child });
  }
  entries.sort((a, b) => {
    if (a.kind === b.kind) return a.name.localeCompare(b.name);
    return a.kind === "directory" ? -1 : 1;
  });
  return entries;
}

export async function readFile(dirHandle, path) {
  const parts = path.split("/").filter(Boolean);
  let current = dirHandle;
  for (let i = 0; i < parts.length - 1; i++) {
    current = await current.getDirectoryHandle(parts[i]);
  }
  const fileHandle = await current.getFileHandle(parts[parts.length - 1]);
  const file = await fileHandle.getFile();
  return await file.text();
}

export async function writeFile(dirHandle, path, text) {
  const parts = path.split("/").filter(Boolean);
  let current = dirHandle;
  for (let i = 0; i < parts.length - 1; i++) {
    current = await current.getDirectoryHandle(parts[i], { create: true });
  }
  const fileHandle = await current.getFileHandle(parts[parts.length - 1], { create: true });
  const writable = await fileHandle.createWritable();
  await writable.write(text);
  await writable.close();
}

export async function deleteEntry(dirHandle, path) {
  const parts = path.split("/").filter(Boolean);
  let current = dirHandle;
  for (let i = 0; i < parts.length - 1; i++) {
    current = await current.getDirectoryHandle(parts[i]);
  }
  await current.removeEntry(parts[parts.length - 1], { recursive: true });
}

export async function createDirectory(dirHandle, path) {
  const parts = path.split("/").filter(Boolean);
  let current = dirHandle;
  for (const part of parts) {
    current = await current.getDirectoryHandle(part, { create: true });
  }
}

export async function renameEntry(dirHandle, oldPath, newPath) {
  const contents = await readFile(dirHandle, oldPath);
  await writeFile(dirHandle, newPath, contents);
  await deleteEntry(dirHandle, oldPath);
}

/* In-memory fallback file system */
class MemoryFS {
  constructor() {
    this.files = new Map();
    this.files.set("go.mod", "module example.com/app\n\ngo 1.22\n");
    this.files.set("main.go", `package main\n\nimport "fmt"\n\nfunc main() {\n\tfmt.Println("Hello, gowasm!")\n}\n`);
  }

  listDir(path = "") {
    const dirs = new Set();
    const files = [];
    const prefix = path ? path + "/" : "";
    for (const p of this.files.keys()) {
      if (!p.startsWith(prefix)) continue;
      const rest = p.slice(prefix.length);
      const slash = rest.indexOf("/");
      if (slash >= 0) {
        dirs.add(rest.slice(0, slash));
      } else {
        files.push(rest);
      }
    }
    const entries = [];
    for (const d of Array.from(dirs).sort()) entries.push({ name: d, kind: "directory" });
    for (const f of files.sort()) entries.push({ name: f, kind: "file" });
    return entries;
  }

  readFile(path) {
    if (!this.files.has(path)) throw new Error("File not found: " + path);
    return this.files.get(path);
  }

  writeFile(path, text) {
    this.files.set(path, text);
  }

  deleteEntry(path) {
    this.files.delete(path);
    const prefix = path + "/";
    for (const p of Array.from(this.files.keys())) {
      if (p.startsWith(prefix)) this.files.delete(p);
    }
  }

  createDirectory(path) {
    // No-op for flat Map; directories are implicit
  }

  renameEntry(oldPath, newPath) {
    const contents = this.readFile(oldPath);
    this.writeFile(newPath, contents);
    this.deleteEntry(oldPath);
  }

  exportFiles() {
    return Array.from(this.files.entries()).map(([path, contents]) => ({ path, contents }));
  }

  importFiles(files) {
    for (const { path, contents } of files) {
      this.files.set(path, contents);
    }
  }
}

export function createMemoryFS() {
  return new MemoryFS();
}

export function isFileSystemAccessSupported() {
  return typeof showDirectoryPicker === "function";
}
