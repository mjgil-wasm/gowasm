import { EditorView, keymap, lineNumbers } from "https://esm.sh/@codemirror/view";
import { EditorState, Compartment } from "https://esm.sh/@codemirror/state";
import { basicSetup } from "https://esm.sh/codemirror";
import { go } from "https://esm.sh/@codemirror/lang-go";
import { indentWithTab } from "https://esm.sh/@codemirror/commands";

const themeCompartment = new Compartment();

function makeTheme() {
  return EditorView.theme({
    "&": {
      backgroundColor: "#0f0f1e",
      color: "#d0d0e0",
      fontSize: "13px",
      lineHeight: "1.6",
    },
    ".cm-gutters": {
      backgroundColor: "#0f0f1e",
      color: "#888",
      borderRight: "1px solid #333",
    },
    ".cm-activeLineGutter": {
      backgroundColor: "rgba(0, 119, 170, 0.15)",
    },
    ".cm-activeLine": {
      backgroundColor: "rgba(0, 119, 170, 0.08)",
    },
    ".cm-selectionBackground": {
      backgroundColor: "rgba(0, 119, 170, 0.3)",
    },
    ".cm-cursor": {
      borderLeftColor: "#e0e0e0",
    },
    ".cm-scroller": {
      fontFamily: 'ui-monospace, "Cascadia Code", "Fira Code", monospace',
    },
  }, { dark: true });
}

const SNIPPETS = [
  { label: "func main", desc: "main function", text: 'func main() {\n\t$0\n}' },
  { label: "func Test", desc: "test function", text: 'func Test$1(t *testing.T) {\n\t$0\n}' },
  { label: "package main", desc: "package declaration", text: 'package main\n\n$0' },
  { label: "fmt.Println", desc: "print line", text: 'fmt.Println($0)' },
  { label: "for range", desc: "range loop", text: 'for $1 := range $2 {\n\t$0\n}' },
  { label: "if err", desc: "error check", text: 'if err != nil {\n\treturn err\n}' },
  { label: "struct", desc: "struct type", text: 'type $1 struct {\n\t$0\n}' },
];

export class TabbedEditor {
  constructor(parent, options = {}) {
    this.parent = parent;
    this.onChange = options.onChange ?? (() => {});
    this.onSave = options.onSave ?? (() => {});
    this.tabs = [];
    this.activePath = "";
    this.dirty = new Set();
    this.editorView = null;
    this.snippetsEl = document.getElementById("snippets");
    this.snippetIndex = -1;
    this._buildDOM();
    this._initEditor();
    this._bindKeys();
  }

  _buildDOM() {
    this.tabBar = this.parent.querySelector(".editor-tabs");
    this.editorContainer = this.parent.querySelector(".editor-wrapper");
  }

  _initEditor() {
    const state = EditorState.create({
      doc: "",
      extensions: [
        basicSetup,
        lineNumbers(),
        go(),
        makeTheme(),
        themeCompartment.of(makeTheme()),
        keymap.of([indentWithTab]),
        EditorView.updateListener.of((update) => {
          if (update.docChanged) {
            this._handleChange(update.state.doc.toString());
          }
        }),
      ],
    });

    this.editorView = new EditorView({
      state,
      parent: this.editorContainer,
    });
  }

  _bindKeys() {
    document.addEventListener("keydown", (e) => {
      if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "s") {
        e.preventDefault();
        this.onSave(this.activePath);
      }
      if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "w") {
        e.preventDefault();
        this.closeTab(this.activePath);
      }
      if (e.ctrlKey && e.key === " ") {
        e.preventDefault();
        this._showSnippets();
      }
    });

    document.addEventListener("click", (e) => {
      if (!this.snippetsEl.contains(e.target)) {
        this._hideSnippets();
      }
    });
  }

  _handleChange(text) {
    if (this.activePath) {
      this.dirty.add(this.activePath);
      this._renderTabs();
      this.onChange(this.activePath, text);
    }
  }

  _showSnippets() {
    if (!this.editorView) return;
    const coords = this.editorView.coordsAtPos(this.editorView.state.selection.main.head);
    if (!coords) return;
    this.snippetsEl.style.left = coords.left + "px";
    this.snippetsEl.style.top = coords.bottom + "px";
    this.snippetsEl.hidden = false;
    this.snippetsEl.innerHTML = "";
    this.snippetIndex = -1;
    for (const s of SNIPPETS) {
      const div = document.createElement("div");
      div.className = "snippet-item";
      div.innerHTML = `<span class="snippet-label">${s.label}</span><span class="snippet-desc"> — ${s.desc}</span>`;
      div.addEventListener("mousedown", (e) => {
        e.preventDefault();
        this._insertSnippet(s.text);
      });
      this.snippetsEl.appendChild(div);
    }
  }

  _hideSnippets() {
    this.snippetsEl.hidden = true;
    this.snippetIndex = -1;
  }

  _insertSnippet(template) {
    if (!this.editorView) return;
    const cursor = this.editorView.state.selection.main.head;
    this.editorView.dispatch({
      changes: { from: cursor, insert: template.replace(/\$\d+/g, "") },
    });
    this._hideSnippets();
  }

  openTab(path, content = "") {
    if (!this.tabs.some((t) => t.path === path)) {
      this.tabs.push({ path });
    }
    this._activePath = path;
    this._renderTabs();
    this._setContent(content);
  }

  get activePath() {
    return this._activePath;
  }

  set activePath(value) {
    this._activePath = value;
  }

  closeTab(path) {
    this.tabs = this.tabs.filter((t) => t.path !== path);
    this.dirty.delete(path);
    if (this.activePath === path) {
      this.activePath = this.tabs.length > 0 ? this.tabs[this.tabs.length - 1].path : "";
      if (this.activePath) {
        // Content will be set by caller from file system
        this._renderTabs();
        this._setContent("");
        return this.activePath;
      }
    }
    this._renderTabs();
    if (!this.activePath) {
      this._setContent("");
    }
    return this.activePath;
  }

  _renderTabs() {
    this.tabBar.innerHTML = "";
    for (const tab of this.tabs) {
      const el = document.createElement("div");
      el.className = "editor-tab" + (tab.path === this.activePath ? " active" : "") + (this.dirty.has(tab.path) ? " dirty" : "");
      const name = tab.path.split("/").pop();
      el.innerHTML = `<span class="tab-label">${name}</span><span class="tab-close">×</span>`;
      el.addEventListener("click", (e) => {
        if (e.target.classList.contains("tab-close")) {
          this.closeTab(tab.path);
        } else {
          this._activateTab(tab.path);
        }
      });
      this.tabBar.appendChild(el);
    }
  }

  _activateTab(path) {
    this._activePath = path;
    this._renderTabs();
    // Content must be provided by caller
  }

  _setContent(text) {
    if (!this.editorView) return;
    const state = EditorState.create({
      doc: text,
      extensions: this.editorView.state.facet(EditorView.editable) ? this.editorView.state.reconfiguration : [
        basicSetup,
        lineNumbers(),
        go(),
        makeTheme(),
        keymap.of([indentWithTab]),
        EditorView.updateListener.of((update) => {
          if (update.docChanged) {
            this._handleChange(update.state.doc.toString());
          }
        }),
      ],
    });
    this.editorView.setState(state);
  }

  getContent() {
    return this.editorView ? this.editorView.state.doc.toString() : "";
  }

  markClean(path) {
    this.dirty.delete(path);
    this._renderTabs();
  }

  isDirty(path) {
    return this.dirty.has(path);
  }

  setReadonly(readonly) {
    if (!this.editorView) return;
    this.editorView.dispatch({
      effects: this.editorView.state.reconfiguration,
    });
  }
}
