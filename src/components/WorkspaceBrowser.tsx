import { invoke } from "@tauri-apps/api/core";
import { ArrowUp, File, Folder, FolderOpen, RefreshCw } from "lucide-react";
import { useCallback, useEffect, useState } from "react";

import type { Workspace, WorkspaceDirectoryListing } from "../lib/types";

interface WorkspaceBrowserProps {
  workspace?: Workspace;
  currentDirectory?: string;
  navigationEnabled: boolean;
  onNavigate: (directory: string) => Promise<string>;
}

function messageFromError(error: unknown) {
  return error instanceof Error ? error.message : String(error);
}

export function WorkspaceBrowser({
  workspace,
  currentDirectory,
  navigationEnabled,
  onNavigate,
}: WorkspaceBrowserProps) {
  const [listing, setListing] = useState<WorkspaceDirectoryListing | null>(
    null,
  );
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const loadDirectory = useCallback(
    async (directory: string) => {
      if (!workspace) {
        setListing(null);
        return;
      }

      setLoading(true);
      setError(null);
      try {
        const result = await invoke<WorkspaceDirectoryListing>(
          "list_workspace_directory",
          { workspace, directory },
        );
        setListing(result);
      } catch (loadError) {
        setError(messageFromError(loadError));
      } finally {
        setLoading(false);
      }
    },
    [workspace],
  );

  useEffect(() => {
    const directory = currentDirectory ?? workspace?.profile.workingDirectory;
    if (directory) {
      void loadDirectory(directory);
    } else {
      setListing(null);
    }
  }, [currentDirectory, loadDirectory, workspace?.profile.workingDirectory]);

  async function openDirectory(directory: string) {
    if (!navigationEnabled || loading) {
      return;
    }

    setLoading(true);
    setError(null);
    try {
      const resolved = await onNavigate(directory);
      await loadDirectory(resolved);
    } catch (navigationError) {
      setError(messageFromError(navigationError));
      setLoading(false);
    }
  }

  if (!workspace) {
    return null;
  }

  const directoryCount =
    listing?.entries.filter((entry) => entry.kind === "directory").length ?? 0;
  const fileCount = (listing?.entries.length ?? 0) - directoryCount;

  return (
    <section className="workspace-browser" aria-label="Workspace browser">
      <div className="workspace-browser-heading">
        <span>
          <FolderOpen size={14} />
          Explorer
        </span>
        <div className="workspace-browser-actions">
          <button
            type="button"
            disabled={
              loading || !listing?.parentDirectory || !navigationEnabled
            }
            onClick={() =>
              listing?.parentDirectory &&
              void openDirectory(listing.parentDirectory)
            }
            aria-label="Open parent folder"
            title={
              listing?.parentDirectory
                ? "Open parent folder"
                : "Workspace root reached"
            }
          >
            <ArrowUp size={14} />
          </button>
          <button
            type="button"
            disabled={loading || !listing}
            onClick={() =>
              listing && void loadDirectory(listing.currentDirectory)
            }
            aria-label="Refresh workspace browser"
            title="Refresh"
          >
            <RefreshCw className={loading ? "spin" : undefined} size={14} />
          </button>
        </div>
      </div>

      <div className="workspace-browser-path" title={listing?.currentDirectory}>
        <strong>{workspace.name}</strong>
        <span>
          {listing?.relativePath === "." ? "/" : `/${listing?.relativePath}`}
        </span>
      </div>

      <div className="workspace-browser-list">
        {error ? <p className="workspace-browser-error">{error}</p> : null}
        {!error && loading && !listing ? (
          <p className="workspace-browser-message">Loading folders...</p>
        ) : null}
        {!error && !loading && listing?.entries.length === 0 ? (
          <p className="workspace-browser-message">This folder is empty.</p>
        ) : null}
        {!error
          ? listing?.entries.map((entry) =>
              entry.kind === "directory" ? (
                <button
                  className="workspace-browser-entry directory"
                  type="button"
                  key={entry.path}
                  disabled={!navigationEnabled || loading}
                  onClick={() => void openDirectory(entry.path)}
                  title={`Open ${entry.name}`}
                >
                  <Folder size={14} />
                  <span>{entry.name}</span>
                </button>
              ) : (
                <div
                  className="workspace-browser-entry file"
                  key={entry.path}
                  title={entry.name}
                >
                  <File size={13} />
                  <span>{entry.name}</span>
                </div>
              ),
            )
          : null}
      </div>

      {listing ? (
        <div className="workspace-browser-summary">
          {directoryCount} folders, {fileCount} files
        </div>
      ) : null}
    </section>
  );
}
