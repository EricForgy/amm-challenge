"use client";

import dynamic from "next/dynamic";

const MonacoEditor = dynamic(() => import("@monaco-editor/react"), { ssr: false });

let solidityConfigured = false;

function configureSolidity(monaco) {
  if (solidityConfigured) return;
  solidityConfigured = true;

  monaco.languages.register({ id: "solidity" });
  monaco.languages.setMonarchTokensProvider("solidity", {
    defaultToken: "",
    tokenizer: {
      root: [
        [/[a-zA-Z_]\w*/, {
          cases: {
            "@keywords": "keyword",
            "@typeKeywords": "type",
            "@default": "identifier"
          }
        }],
        { include: "@whitespace" },
        [/\d+\.\d+([eE][\-+]?\d+)?/, "number.float"],
        [/0[xX][0-9a-fA-F_]+/, "number.hex"],
        [/\d+/, "number"],
        [/[{}()\[\]]/, "@brackets"],
        [/[;,.]/, "delimiter"],
        [/"([^"\\]|\\.)*$/, "string.invalid"],
        [/'([^'\\]|\\.)*$/, "string.invalid"],
        [/"/, "string", "@string_double"],
        [/'/, "string", "@string_single"]
      ],
      whitespace: [
        [/[ \t\r\n]+/, "white"],
        [/\/\*\*(?!\/)/, "comment.doc", "@doccomment"],
        [/\/\*/, "comment", "@comment"],
        [/\/\/.*$/, "comment"]
      ],
      comment: [
        [/[^/*]+/, "comment"],
        [/\*\//, "comment", "@pop"],
        [/[/*]/, "comment"]
      ],
      doccomment: [
        [/[^/*]+/, "comment.doc"],
        [/\*\//, "comment.doc", "@pop"],
        [/[/*]/, "comment.doc"]
      ],
      string_double: [
        [/[^\\"]+/, "string"],
        [/\\./, "string.escape"],
        [/"/, "string", "@pop"]
      ],
      string_single: [
        [/[^\\']+/, "string"],
        [/\\./, "string.escape"],
        [/'/, "string", "@pop"]
      ]
    },
    keywords: [
      "pragma", "solidity", "contract", "interface", "library", "is", "import",
      "function", "returns", "return", "event", "error", "mapping", "struct",
      "enum", "if", "else", "for", "while", "do", "break", "continue", "new",
      "delete", "emit", "require", "revert", "assert", "modifier", "override",
      "virtual", "public", "private", "internal", "external", "pure", "view",
      "payable", "memory", "storage", "calldata", "indexed", "anonymous", "using",
      "as", "from", "constructor", "fallback", "receive", "unchecked", "assembly"
    ],
    typeKeywords: [
      "address", "bool", "string", "bytes", "byte", "uint", "int", "uint8",
      "uint16", "uint24", "uint32", "uint64", "uint128", "uint160", "uint256",
      "int8", "int16", "int24", "int32", "int64", "int128", "int160", "int256",
      "bytes1", "bytes2", "bytes4", "bytes8", "bytes16", "bytes20", "bytes32"
    ]
  });

  monaco.editor.defineTheme("solidity-dark-plus", {
    base: "vs-dark",
    inherit: true,
    rules: [
      { token: "keyword", foreground: "C586C0" },
      { token: "type", foreground: "4EC9B0" },
      { token: "identifier", foreground: "D4D4D4" },
      { token: "number", foreground: "B5CEA8" },
      { token: "number.float", foreground: "B5CEA8" },
      { token: "number.hex", foreground: "B5CEA8" },
      { token: "string", foreground: "CE9178" },
      { token: "string.escape", foreground: "D7BA7D" },
      { token: "comment", foreground: "6A9955" },
      { token: "comment.doc", foreground: "608B4E" },
      { token: "delimiter", foreground: "D4D4D4" }
    ],
    colors: {
      "editor.background": "#0b1220",
      "editor.foreground": "#d4d4d4",
      "editorLineNumber.foreground": "#5c6f91",
      "editorLineNumber.activeForeground": "#9eb4d4",
      "editorCursor.foreground": "#35c5b0",
      "editor.selectionBackground": "#264f78",
      "editor.inactiveSelectionBackground": "#1f3650",
      "editorIndentGuide.background1": "#20304d",
      "editorIndentGuide.activeBackground1": "#35547f"
    }
  });
}

export default function CodeEditor({
  value,
  onChange,
  language = "solidity",
  height = "360px"
}) {
  return (
    <MonacoEditor
      beforeMount={configureSolidity}
      language={language}
      theme="solidity-dark-plus"
      value={value}
      onChange={(next) => onChange(next ?? "")}
      height={height}
      options={{
        minimap: { enabled: false },
        scrollBeyondLastLine: false,
        automaticLayout: true,
        fontSize: 14,
        fontFamily: "JetBrains Mono, Menlo, Consolas, monospace",
        fontLigatures: true,
        lineNumbersMinChars: 3,
        roundedSelection: false,
        renderLineHighlight: "line",
        tabSize: 2
      }}
    />
  );
}
