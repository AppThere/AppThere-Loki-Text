/**
 * Session-based autosave manager.
 *
 * Creates an isolated editing session per document. Autosave writes to the
 * session directory inside the app data directory; the user's original file
 * is never touched until they explicitly save (Ctrl+S / Save menu item).
 *
 * ## Session lifecycle
 *
 * 1. User opens a file  → `SessionManager.create(originalPath)`
 * 2. Editor changes     → `session.autoSave(content, styles, metadata)` every 30 s
 * 3. 5-minute interval  → `session.createSnapshot(...)` for version history
 * 4. User presses Ctrl+S → `session.saveToOriginal(...)` (writes original file)
 * 5. User closes file   → `session.cleanup()`
 */

import { invoke } from '@tauri-apps/api/core';
import type { LexicalDocumentData, StyleDefinition, Metadata } from '../types/odt';
import {
    SessionMeta,
    createSessionDir,
    writeCurrentOdt,
    readCurrentOdt,
    writeSnapshot,
    writeMeta,
    readMeta,
    deleteSessionDir,
    findRecoverableSessions,
} from './sessionStorage';

export type { SessionMeta };

interface DocState {
    content: LexicalDocumentData;
    styles: Record<string, StyleDefinition>;
    metadata: Metadata;
}

// ─── Serialisation helpers ────────────────────────────────────────────────────

async function serializeToBytes(state: DocState): Promise<Uint8Array> {
    const result: number[] = await invoke('serialize_document', {
        lexicalJson: JSON.stringify(state.content),
        styles: state.styles,
        metadata: state.metadata,
    });
    return new Uint8Array(result);
}

async function deserializeFromBytes(bytes: Uint8Array): Promise<DocState> {
    return invoke('deserialize_document', {
        fileContent: Array.from(bytes),
    });
}

// ─── SessionManager ───────────────────────────────────────────────────────────

export class SessionManager {
    private meta: SessionMeta;

    private constructor(meta: SessionMeta) {
        this.meta = meta;
    }

    // ── Factory ──────────────────────────────────────────────────────────────

    /** Create a brand-new session for `originalPath`. */
    static async create(originalPath: string): Promise<SessionManager> {
        const sessionId = crypto.randomUUID();
        await createSessionDir(sessionId);
        const meta: SessionMeta = {
            sessionId,
            originalPath,
            createdAt: new Date().toISOString(),
            lastModified: new Date().toISOString(),
            autoSaveCount: 0,
            snapshotCount: 0,
        };
        await writeMeta(meta);
        return new SessionManager(meta);
    }

    /** Resume an existing session by ID (e.g. after crash recovery). */
    static async load(sessionId: string): Promise<SessionManager> {
        const meta = await readMeta(sessionId);
        return new SessionManager(meta);
    }

    /** Find all sessions that can be recovered (have a `current.odt`). */
    static findRecoverable(): Promise<SessionMeta[]> {
        return findRecoverableSessions();
    }

    // ── Autosave (safe — never writes original file) ─────────────────────────

    /**
     * Autosave current editor state to the session directory.
     *
     * Called on a 30-second timer. The user's original file is untouched.
     */
    async autoSave(state: DocState): Promise<void> {
        const bytes = await serializeToBytes(state);
        await writeCurrentOdt(this.meta.sessionId, bytes);
        this.meta.autoSaveCount++;
        this.meta.lastModified = new Date().toISOString();
        await writeMeta(this.meta);
    }

    /**
     * Create a named snapshot (called every 5 minutes).
     *
     * Snapshots are kept even after the session is cleaned up, providing
     * a short version history the user can recover from.
     */
    async createSnapshot(state: DocState): Promise<void> {
        const bytes = await serializeToBytes(state);
        const num = this.meta.snapshotCount + 1;
        await writeSnapshot(this.meta.sessionId, num, bytes);
        this.meta.snapshotCount = num;
        this.meta.lastModified = new Date().toISOString();
        await writeMeta(this.meta);
    }

    // ── Explicit user-initiated save ─────────────────────────────────────────

    /**
     * Save editor state to the original file (user-initiated only).
     *
     * Called when the user presses Ctrl+S or chooses File → Save.
     * Also updates the session's `current.odt` so they stay in sync.
     */
    async saveToOriginal(state: DocState): Promise<void> {
        const bytes = await serializeToBytes(state);
        // Write to original file. On Android, plugin-fs uses Rust's std::fs which
        // cannot write to content:// URIs; use the native ContentResolver command
        // for those paths instead.
        // plugin-fs writeFile uses Android's ContentResolver for content:// URIs,
        // so it handles both regular paths and SAF content:// URIs correctly.
        const { writeFile } = await import('@tauri-apps/plugin-fs');
        await writeFile(this.meta.originalPath, bytes);
        // Keep session current in sync
        await writeCurrentOdt(this.meta.sessionId, bytes);
        this.meta.lastModified = new Date().toISOString();
        await writeMeta(this.meta);
    }

    // ── Recovery ─────────────────────────────────────────────────────────────

    /** Load the last autosaved state, or null if there is none. */
    async loadCurrent(): Promise<DocState | null> {
        const bytes = await readCurrentOdt(this.meta.sessionId);
        if (!bytes) return null;
        return deserializeFromBytes(bytes);
    }

    // ── Cleanup ───────────────────────────────────────────────────────────────

    /**
     * Remove the session directory.
     *
     * Called when the user closes the document without a pending crash.
     */
    async cleanup(): Promise<void> {
        await deleteSessionDir(this.meta.sessionId);
    }

    // ── Accessors ─────────────────────────────────────────────────────────────

    get sessionId(): string { return this.meta.sessionId; }
    get originalPath(): string { return this.meta.originalPath; }
    get sessionMeta(): Readonly<SessionMeta> { return { ...this.meta }; }
}
