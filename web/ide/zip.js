import JSZip from "https://esm.sh/jszip@3.10.1";

/**
 * Export an array of { path, contents } files to a ZIP Blob.
 */
export async function exportZip(files) {
  const zip = new JSZip();
  for (const { path, contents } of files) {
    zip.file(path, contents);
  }
  return await zip.generateAsync({ type: "blob" });
}

/**
 * Import a ZIP Blob and return an array of { path, contents } files.
 */
export async function importZip(blob) {
  const zip = await JSZip.loadAsync(blob);
  const files = [];
  for (const [path, entry] of Object.entries(zip.files)) {
    if (entry.dir) continue;
    const contents = await entry.async("string");
    files.push({ path, contents });
  }
  return files;
}
