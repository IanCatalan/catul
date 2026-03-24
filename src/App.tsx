import "./App.css";
import { useState, useEffect, useRef, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { EditorView, keymap, placeholder } from "@codemirror/view";
import { EditorState } from "@codemirror/state";
import { defaultKeymap, history, historyKeymap } from "@codemirror/commands";
import Database from '@tauri-apps/plugin-sql'


interface Segment {
  id: number;
  source: string;
  target: string;
  matchPercent: number;
  status: "empty" | "translated" | "reviewed";
}

interface CMEditorProps {
  value: string;
  onChange: (value: string) => void;
  onConfirm: () => void;
}

const db = await Database.load('sqlite:catul.db') // Load the SQLite database located at 'catul.db'

function CMEditor({ value, onChange, onConfirm }: CMEditorProps) {
  const containerRef = useRef<HTMLDivElement>(null);
  const viewRef = useRef<EditorView | null>(null);
  const externalUpdate = useRef(false);

  const handleConfirm = useCallback(() => {
    onConfirm();
  }, [onConfirm]);

  useEffect(() => {
    if (!containerRef.current) return;

    const startState = EditorState.create({
      doc: value,
      extensions: [
        history(),
        keymap.of([
          ...defaultKeymap,
          ...historyKeymap,
          {
            key: "Mod-Enter",
            run: () => {
              handleConfirm();
              return true;
            },
          },
        ]),
        placeholder("Escribe tu traducción aquí…"),
        EditorView.updateListener.of((update) => {
          if (update.docChanged && !externalUpdate.current) {
            onChange(update.state.doc.toString());
          }
        }),
        EditorView.theme({
          "&": {
            height: "100%",
            fontSize: "16px",
            backgroundColor: "transparent",
            color: "#e5e5e5",
          },
          ".cm-content": {
            padding: "12px 16px",
            fontFamily: "'IBM Plex Mono', 'Fira Code', monospace",
            caretColor: "#34d399",
            lineHeight: "1.7",
          },
          ".cm-line": { padding: "0" },
          ".cm-cursor": {
            borderLeftColor: "#34d399",
            borderLeftWidth: "2px",
          },
          ".cm-focused": { outline: "none" },
          "&.cm-focused .cm-selectionBackground, .cm-selectionBackground": {
            backgroundColor: "#064e3b88",
          },
          ".cm-placeholder": { color: "#525252" },
          // Scrollbar
          ".cm-scroller": { overflow: "auto" },
        }),
        EditorView.lineWrapping,
      ],
    });

    const view = new EditorView({
      state: startState,
      parent: containerRef.current,
    });

    viewRef.current = view;
    view.focus();

    return () => {
      view.destroy();
      viewRef.current = null;
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  useEffect(() => {
    const view = viewRef.current;
    if (!view) return;
    const current = view.state.doc.toString();
    if (current === value) return;

    externalUpdate.current = true;
    view.dispatch({
      changes: { from: 0, to: current.length, insert: value },
    });
    externalUpdate.current = false;
    view.focus();
  }, [value]);

  useEffect(() => {
  }, [handleConfirm]);

  return (
    <div
      ref={containerRef}
      className="h-full w-full"
      style={{ minHeight: 0 }}
    />
  );
}

// ── Main view ─────
export default function MainView() {
  const [segments, setSegments] = useState<Segment[]>([
    {
      id: 1,
      source: "Welcome to our new translation tool.",
      target: "",
      matchPercent: 0,
      status: "empty",
    },
    {
      id: 2,
      source: "This is a simple CAT prototype.",
      target: "Este es un prototipo simple de CAT.",
      matchPercent: 85,
      status: "translated",
    },
    {
      id: 3,
      source: "It runs completely offline.",
      target: "",
      matchPercent: 0,
      status: "empty",
    },
    {
      id: 4,
      source: "Privacy first.",
      target: "Privacidad primero.",
      matchPercent: 100,
      status: "reviewed",
    },
  ]);

  const [activeSegmentId, setActiveSegmentId] = useState<number | null>(1);

  const handleTargetChange = (id: number, value: string) => {
    setSegments((prev) =>
      prev.map((seg) =>
        seg.id === id
          ? {
              ...seg,
              target: value,
              status: value.trim() ? "translated" : "empty",
            }
          : seg
      )
    );
  };

  const handleConfirm = () => {
    if (activeSegmentId === null) return;
    setSegments((prev) =>
      prev.map((seg) =>
        seg.id === activeSegmentId && seg.target.trim()
          ? { ...seg, status: "reviewed" }
          : seg
      )
    );
    const nextEmpty = segments.find(
      (s) => s.id > activeSegmentId && s.status !== "reviewed"
    );
    if (nextEmpty) setActiveSegmentId(nextEmpty.id);
  };

  const activeSegment = segments.find((s) => s.id === activeSegmentId);

  return (
    <div className="flex h-screen flex-col bg-neutral-950 text-neutral-100">
      <header className="flex h-14 shrink-0 items-center justify-between border-b border-neutral-800 bg-neutral-900/80 px-4 backdrop-blur-sm">
        <div className="flex items-center gap-4">
          <h1 className="text-lg font-semibold tracking-tight">Mi CAT Simple</h1>
          <span className="rounded bg-emerald-950 px-2.5 py-0.5 text-xs font-medium text-emerald-400">
            Offline • v0.1
          </span>
        </div>
        <div className="flex items-center gap-3">
          <button
            className="rounded-md px-3 py-1.5 text-sm hover:bg-neutral-800"
            onClick={() => invoke("saludo")}
          >
            Abrir proyecto…
          </button>
          <button className="rounded-md bg-emerald-600 px-4 py-1.5 text-sm font-medium hover:bg-emerald-700">
            Guardar TM
          </button>
          <button className="rounded-md px-3 py-1.5 text-sm hover:bg-neutral-800">
            Configuración
          </button>
        </div>
      </header>

      <div className="flex flex-1 overflow-hidden">
        <div className="flex w-1/2 flex-col border-r border-neutral-800">
          <div className="flex h-10 shrink-0 items-center border-b border-neutral-800 bg-neutral-900/50 px-4 text-sm font-medium text-neutral-400">
            <div className="w-12">N°</div>
            <div className="flex-1">Original (Source)</div>
            <div className="w-24 text-right">Match</div>
          </div>
          <div className="flex-1 overflow-y-auto">
            {segments.map((seg) => {
              const isActive = seg.id === activeSegmentId;
              return (
                <div
                  key={seg.id}
                  onClick={() => setActiveSegmentId(seg.id)}
                  className={`
                    flex cursor-pointer border-b border-neutral-800 px-4 py-3 transition-colors
                    ${isActive
                      ? "bg-emerald-950/40 hover:bg-emerald-950/60"
                      : "hover:bg-neutral-900/60"}
                  `}
                >
                  <div className="w-12 font-mono text-neutral-500">{seg.id}</div>
                  <div className="flex-1 pr-4 text-neutral-200">{seg.source}</div>
                  <div className="w-24 text-right">
                    {seg.matchPercent > 0 && (
                      <span
                        className={`
                          rounded px-2 py-0.5 text-xs font-medium
                          ${seg.matchPercent === 100
                            ? "bg-emerald-950 text-emerald-300"
                            : seg.matchPercent >= 75
                            ? "bg-amber-950 text-amber-300"
                            : "bg-neutral-800 text-neutral-400"}
                        `}
                      >
                        {seg.matchPercent}%
                      </span>
                    )}
                  </div>
                </div>
              );
            })}
          </div>
        </div>

        <div className="flex w-1/2 flex-col">
          <div className="flex h-10 shrink-0 items-center border-b border-neutral-800 bg-neutral-900/50 px-4 text-sm font-medium text-neutral-400">
            Traducción (Target)
            <span className="ml-auto text-xs text-neutral-600">
              Ctrl+Enter para confirmar
            </span>
          </div>

          {activeSegment ? (
            <div className="flex flex-1 flex-col p-6" style={{ minHeight: 0 }}>
              <div className="mb-4 rounded-lg border border-neutral-800 bg-neutral-900/60 p-4">
                <h3 className="mb-2 text-sm font-medium text-neutral-400">
                  Texto original:
                </h3>
                <p className="text-lg leading-relaxed text-neutral-100 whitespace-pre-wrap">
                  {activeSegment.source}
                </p>
              </div>

              <div className="flex-1 min-h-0 rounded-lg border border-neutral-700 overflow-hidden bg-neutral-900">
                <CMEditor
                  key={activeSegment.id}
                  value={activeSegment.target}
                  onChange={(v) => handleTargetChange(activeSegment.id, v)}
                  onConfirm={handleConfirm}
                />
              </div>

              <div className="mt-4 flex justify-end gap-3">
                <button className="rounded-md px-4 py-2 text-sm hover:bg-neutral-800">
                  Saltar
                </button>
                <button
                  className="rounded-md bg-emerald-600 px-5 py-2 font-medium hover:bg-emerald-700"
                  onClick={handleConfirm}
                >
                  Confirmar → siguiente
                </button>
              </div>
            </div>
          ) : (
            <div className="flex flex-1 items-center justify-center text-neutral-500">
              Selecciona un segmento para traducir
            </div>
          )}
        </div>

        <div className="w-80 shrink-0 border-l border-neutral-800 bg-neutral-900/40 p-4">
          <h2 className="mb-4 text-sm font-semibold uppercase tracking-wide text-neutral-400">
            Sugerencias
          </h2>
          <div className="space-y-3">
            <div className="rounded border border-neutral-800 bg-neutral-950 p-3">
              <div className="mb-1 flex items-center justify-between text-xs text-neutral-500">
                <span>Coincidencia 92%</span>
                <span className="text-emerald-400">TM</span>
              </div>
              <p className="text-sm text-neutral-300">
                Esta es una sugerencia de memoria de traducción.
              </p>
            </div>
          </div>
        </div>
      </div>

      {/* Footer */}
      <footer className="flex h-10 shrink-0 items-center justify-between border-t border-neutral-800 bg-neutral-900/80 px-4 text-sm text-neutral-400">
        <div>
          Segmento {activeSegmentId} de {segments.length} • 75 palabras nuevas •
          2 repetidas
        </div>
        <div>Progreso: 50% • Modo oscuro</div>
      </footer>
    </div>
  );
}