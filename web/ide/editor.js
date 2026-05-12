import { EditorView, keymap } from "https://esm.sh/@codemirror/view";
import { EditorState, Compartment } from "https://esm.sh/@codemirror/state";
import { basicSetup } from "https://esm.sh/codemirror";
import { go } from "https://esm.sh/@codemirror/lang-go";
import { indentWithTab } from "https://esm.sh/@codemirror/commands";
import { HighlightStyle, syntaxHighlighting } from "https://esm.sh/@codemirror/language";
import { tags } from "https://esm.sh/@lezer/highlight";

const themeCompartment = new Compartment();
const syntaxCompartment = new Compartment();

const THEMES = {
  dark: EditorView.theme({
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
      borderLeft: "1.2px solid #e0e0e0",
    },
    ".cm-content": {
      caretColor: "#e0e0e0",
    },
    ".cm-scroller": {
      fontFamily: 'ui-monospace, "Cascadia Code", "Fira Code", monospace',
    },
  }, { dark: true }),
  light: EditorView.theme({
    "&": {
      backgroundColor: "#fafafa",
      color: "#2e2e2e",
      fontSize: "13px",
      lineHeight: "1.6",
    },
    ".cm-gutters": {
      backgroundColor: "#f0f0f0",
      color: "#666",
      borderRight: "1px solid #ddd",
    },
    ".cm-activeLineGutter": {
      backgroundColor: "rgba(0, 119, 170, 0.12)",
    },
    ".cm-activeLine": {
      backgroundColor: "rgba(0, 119, 170, 0.06)",
    },
    ".cm-selectionBackground": {
      backgroundColor: "rgba(0, 119, 170, 0.25)",
    },
    ".cm-cursor": {
      borderLeft: "1.2px solid #2e2e2e",
    },
    ".cm-content": {
      caretColor: "#2e2e2e",
    },
    ".cm-scroller": {
      fontFamily: 'ui-monospace, "Cascadia Code", "Fira Code", monospace',
    },
  }),
  "high-contrast": EditorView.theme({
    "&": {
      backgroundColor: "#000000",
      color: "#ffffff",
      fontSize: "13px",
      lineHeight: "1.6",
    },
    ".cm-gutters": {
      backgroundColor: "#000000",
      color: "#cccccc",
      borderRight: "1px solid #555",
    },
    ".cm-activeLineGutter": {
      backgroundColor: "rgba(255, 255, 255, 0.2)",
    },
    ".cm-activeLine": {
      backgroundColor: "rgba(255, 255, 255, 0.1)",
    },
    ".cm-selectionBackground": {
      backgroundColor: "rgba(255, 255, 255, 0.35)",
    },
    ".cm-cursor": {
      borderLeft: "2px solid #ffff00",
    },
    ".cm-content": {
      caretColor: "#ffff00",
    },
    ".cm-scroller": {
      fontFamily: 'ui-monospace, "Cascadia Code", "Fira Code", monospace',
    },
  }),
};

const DARK_SYNTAX = HighlightStyle.define([
  { tag: tags.keyword, color: "#ff79c6" },
  { tag: tags.string, color: "#f1fa8c" },
  { tag: tags.comment, color: "#6272a4" },
  { tag: tags.variableName, color: "#f8f8f2" },
  { tag: tags.definition(tags.variableName), color: "#50fa7b" },
  { tag: tags.function(tags.variableName), color: "#50fa7b" },
  { tag: tags.propertyName, color: "#50fa7b" },
  { tag: tags.definition(tags.propertyName), color: "#50fa7b" },
  { tag: tags.local(tags.variableName), color: "#f8f8f2" },
  { tag: tags.special(tags.variableName), color: "#ff79c6" },
  { tag: tags.typeName, color: "#8be9fd" },
  { tag: tags.className, color: "#8be9fd" },
  { tag: tags.namespace, color: "#8be9fd" },
  { tag: tags.number, color: "#bd93f9" },
  { tag: tags.operator, color: "#ff79c6" },
  { tag: tags.punctuation, color: "#d0d0e0" },
  { tag: tags.bool, color: "#bd93f9" },
  { tag: tags.atom, color: "#bd93f9" },
  { tag: tags.meta, color: "#6272a4" },
  { tag: tags.invalid, color: "#ff5555" },
]);

const LIGHT_SYNTAX = HighlightStyle.define([
  { tag: tags.keyword, color: "#d73a49" },
  { tag: tags.string, color: "#032f62" },
  { tag: tags.comment, color: "#6a737d" },
  { tag: tags.variableName, color: "#24292e" },
  { tag: tags.definition(tags.variableName), color: "#005cc5" },
  { tag: tags.function(tags.variableName), color: "#005cc5" },
  { tag: tags.propertyName, color: "#005cc5" },
  { tag: tags.definition(tags.propertyName), color: "#005cc5" },
  { tag: tags.local(tags.variableName), color: "#24292e" },
  { tag: tags.special(tags.variableName), color: "#d73a49" },
  { tag: tags.typeName, color: "#6f42c1" },
  { tag: tags.className, color: "#6f42c1" },
  { tag: tags.namespace, color: "#6f42c1" },
  { tag: tags.number, color: "#005cc5" },
  { tag: tags.operator, color: "#d73a49" },
  { tag: tags.punctuation, color: "#2e2e2e" },
  { tag: tags.bool, color: "#005cc5" },
  { tag: tags.atom, color: "#005cc5" },
  { tag: tags.meta, color: "#6a737d" },
  { tag: tags.invalid, color: "#cb2431" },
]);

const HC_SYNTAX = HighlightStyle.define([
  { tag: tags.keyword, color: "#ffff00" },
  { tag: tags.string, color: "#00ff00" },
  { tag: tags.comment, color: "#aaaaaa" },
  { tag: tags.variableName, color: "#ffffff" },
  { tag: tags.definition(tags.variableName), color: "#ffaa00" },
  { tag: tags.function(tags.variableName), color: "#ffaa00" },
  { tag: tags.propertyName, color: "#ffaa00" },
  { tag: tags.definition(tags.propertyName), color: "#ffaa00" },
  { tag: tags.local(tags.variableName), color: "#ffffff" },
  { tag: tags.special(tags.variableName), color: "#ffff00" },
  { tag: tags.typeName, color: "#00ffff" },
  { tag: tags.className, color: "#00ffff" },
  { tag: tags.namespace, color: "#00ffff" },
  { tag: tags.number, color: "#ff66ff" },
  { tag: tags.operator, color: "#ffff00" },
  { tag: tags.punctuation, color: "#ffffff" },
  { tag: tags.bool, color: "#ff66ff" },
  { tag: tags.atom, color: "#ff66ff" },
  { tag: tags.meta, color: "#aaaaaa" },
  { tag: tags.invalid, color: "#ff4444" },
]);

const SYNTAX_BY_THEME = {
  dark: DARK_SYNTAX,
  light: LIGHT_SYNTAX,
  "high-contrast": HC_SYNTAX,
};
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
    this.saved = new Set();
    this.editorView = null;
    this.snippetsEl = document.getElementById("snippets");
    this.snippetIndex = -1;
    this._programmaticChange = false;
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
        go(),
        themeCompartment.of(THEMES.dark),
        syntaxCompartment.of(syntaxHighlighting(DARK_SYNTAX)),
        keymap.of([indentWithTab]),
        EditorView.updateListener.of((update) => {
          if (update.docChanged && !this._programmaticChange) {
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
    if (this._programmaticChange) return;
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
      const isDirty = this.dirty.has(tab.path);
      const isSaved = this.saved.has(tab.path);
      el.className = "editor-tab" + (tab.path === this.activePath ? " active" : "") + (isDirty ? " dirty" : "");
      const name = tab.path.split("/").pop();
      const indicator = isDirty ? '<span class="tab-indicator dirty" title="Modified">●</span>' : isSaved ? '<span class="tab-indicator saved" title="Saved">✓</span>' : '';
      el.innerHTML = `${indicator}<span class="tab-label">${name}</span><span class="tab-close">×</span>`;
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
  }

  _setContent(text) {
    if (!this.editorView) return;
    const current = this.editorView.state.doc.toString();
    if (current === text) return;
    this._programmaticChange = true;
    this.editorView.dispatch({
      changes: { from: 0, to: this.editorView.state.doc.length, insert: text },
    });
    this._programmaticChange = false;
  }

  getContent() {
    return this.editorView ? this.editorView.state.doc.toString() : "";
  }

  markClean(path) {
    this.dirty.delete(path);
    this.saved.add(path);
    this._renderTabs();
    setTimeout(() => {
      this.saved.delete(path);
      this._renderTabs();
    }, 1500);
  }

  isDirty(path) {
    return this.dirty.has(path);
  }

  setTheme(themeName) {
    const theme = THEMES[themeName] ?? THEMES.dark;
    const syntax = SYNTAX_BY_THEME[themeName] ?? DARK_SYNTAX;
    if (this.editorView) {
      this.editorView.dispatch({
        effects: [
          themeCompartment.reconfigure(theme),
          syntaxCompartment.reconfigure(syntaxHighlighting(syntax)),
        ],
      });
    }
  }
}
