import { ApiClient } from "./client";

export interface DirectoryEntry {
    name: string;
    path: string;
    parent?: string;
}

export interface FilesystemApi {
    listDirectories(path?: string): Promise<DirectoryEntry[]>;
}

export function createFilesystemApi(client: ApiClient): FilesystemApi {
    return {
        async listDirectories(path?: string): Promise<DirectoryEntry[]> {
            return client.get<DirectoryEntry[]>("/api/filesystem", {
                params: { path },
            });
        },
    };
}
