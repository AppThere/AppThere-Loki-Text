/**
 * Low-level session file I/O helpers.
 *
 * Wraps Tauri filesystem + path APIs so the rest of the session module
 * doesn't have to deal with the async import dance.
 */

import { appDataDir, join } from '@tauri-apps/api/path';
import {
    exists,
    mkdir,
    readFile,
    writeFile,
    remove,
    readDir,
} from '@tauri-apps/plugin-fs';

export interface SessionMeta {
    sessionId: string;
    originalPath: string;
    createdAt: string;
    lastModified: string;
    autoSaveCount: number;
    snapshotCount: number;
}

/** Absolute path of the `.loki/sessions` root directory. */
export async function sessionsRoot(): Promise<string> {
    const base = await appDataDir();
    return join(base, 'sessions');
}

/** Absolute path of a specific session directory. */
export async function sessionDir(sessionId: string): Promise<string> {
    return join(await sessionsRoot(), sessionId);
}

/** Create a session directory (and snapshots sub-dir). */
export async function createSessionDir(sessionId: string): Promise<string> {
    const dir = await sessionDir(sessionId);
    await mkdir(dir, { recursive: true });
    await mkdir(await join(dir, 'snapshots'), { recursive: true });
    return dir;
}

/** Write raw bytes to `<sessionDir>/current.odt`. */
export async function writeCurrentOdt(
    sessionId: string,
    bytes: Uint8Array,
): Promise<void> {
    const dir = await sessionDir(sessionId);
    await writeFile(await join(dir, 'current.odt'), bytes);
}

/** Read `<sessionDir>/current.odt`; returns null if absent. */
export async function readCurrentOdt(
    sessionId: string,
): Promise<Uint8Array | null> {
    const path = await join(await sessionDir(sessionId), 'current.odt');
    if (!(await exists(path))) return null;
    return readFile(path);
}

/** Write a numbered snapshot. */
export async function writeSnapshot(
    sessionId: string,
    num: number,
    bytes: Uint8Array,
): Promise<void> {
    const name = num.toString().padStart(3, '0') + '.odt';
    const path = await join(await sessionDir(sessionId), 'snapshots', name);
    await writeFile(path, bytes);
}

/** Write `metadata.json` for a session. */
export async function writeMeta(meta: SessionMeta): Promise<void> {
    const path = await join(await sessionDir(meta.sessionId), 'metadata.json');
    const json = JSON.stringify(meta, null, 2);
    await writeFile(path, new TextEncoder().encode(json));
}

/** Read `metadata.json`; throws if absent. */
export async function readMeta(sessionId: string): Promise<SessionMeta> {
    const path = await join(await sessionDir(sessionId), 'metadata.json');
    const bytes = await readFile(path);
    return JSON.parse(new TextDecoder().decode(bytes)) as SessionMeta;
}

/** Remove a session directory entirely. */
export async function deleteSessionDir(sessionId: string): Promise<void> {
    const dir = await sessionDir(sessionId);
    if (await exists(dir)) {
        await remove(dir, { recursive: true });
    }
}

/** List session IDs that have a `current.odt` (crash-recoverable). */
export async function findRecoverableSessions(): Promise<SessionMeta[]> {
    const root = await sessionsRoot();
    if (!(await exists(root))) return [];

    const entries = await readDir(root);
    const metas: SessionMeta[] = [];

    for (const entry of entries) {
        if (!entry.isDirectory) continue;
        try {
            const meta = await readMeta(entry.name);
            const currentPath = await join(
                await sessionDir(entry.name),
                'current.odt',
            );
            if (await exists(currentPath)) {
                metas.push(meta);
            }
        } catch {
            // Corrupted session directory — skip
        }
    }

    return metas;
}
