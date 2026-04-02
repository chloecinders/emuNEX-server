import { css, html, LitElement } from "lit";
import { ChevronDown, ChevronRight, createIcons, Download, File, Folder, Gamepad2 } from "lucide";
import "../components/navbar.js";
import {
    baseTokens,
    buttonStyles,
    cardStyles,
    formStyles,
    pageHostStyles,
    statusStyles,
    tableStyles,
    tagStyles,
} from "../components/shared-styles.js";
import "../theme-manager.js";

export class EmunexSavesManagePage extends LitElement {
    static properties = {
        saves: { type: Array },
        loading: { type: Boolean },
        error: { type: String },
        selectedRomId: { type: String },
        selectedVersions: { type: Object },
    };

    static styles = [
        baseTokens,
        pageHostStyles,
        cardStyles,
        formStyles,
        buttonStyles,
        tableStyles,
        statusStyles,
        tagStyles,
        css`
            .auth-container {
                max-width: 900px;
            }
            .game-node {
                margin-bottom: var(--spacing-sm);
                background: var(--color-surface);
                border: 1px solid var(--color-border);
                border-radius: var(--radius-md);
            }
            .game-header {
                display: flex;
                align-items: center;
                padding: var(--spacing-sm) var(--spacing-md);
                cursor: pointer;
                transition: background-color 0.2s ease;
                border-radius: var(--radius-md);
            }
            .game-header:hover {
                background: var(--color-surface-variant);
            }
            .icon-wrapper {
                display: flex;
                align-items: center;
                justify-content: center;
                margin-right: var(--spacing-md);
                color: var(--color-primary);
            }
            .icon-wrapper .lucide {
                width: 20px;
                height: 20px;
            }
            .game-title {
                font-weight: 700;
                font-size: 1.1rem;
                flex: 1;
            }
            .game-console {
                margin-left: var(--spacing-md);
                font-size: 0.85rem;
                font-weight: 800;
                color: var(--color-text-muted);
                background: var(--color-surface-variant);
                padding: 4px 10px;
                border-radius: var(--radius-full);
                text-transform: uppercase;
            }
            .versions-list {
                background: transparent;
            }
            .version-item {
                display: flex;
                align-items: center;
                justify-content: space-between;
                padding: var(--spacing-sm) var(--spacing-md);
                border: 1px solid var(--color-border);
                border-radius: var(--radius-md);
                margin-bottom: var(--spacing-sm);
                gap: var(--spacing-md);
            }
            .version-info {
                display: flex;
                align-items: center;
                gap: var(--spacing-md);
                flex: 1;
            }
            .version-name {
                font-family: monospace;
                font-weight: 600;
                font-size: 0.95rem;
                display: flex;
                align-items: center;
                flex: 1;
            }
            .version-name .lucide {
                width: 16px;
                height: 16px;
                margin-right: 8px;
                color: var(--color-text-muted);
            }
            .version-select {
                width: auto;
                padding: var(--spacing-xs) var(--spacing-sm);
                border: 2px solid var(--color-border);
                border-radius: var(--radius-sm);
                background: var(--color-surface);
                color: var(--color-text);
                font-family: inherit;
                font-weight: 600;
                cursor: pointer;
                outline: none;
                transition: border-color 0.2s;
            }
            .version-select:focus {
                border-color: var(--color-primary);
            }
            .btn-sm {
                margin-top: 0;
            }
            .btn-sm .btn-front {
                padding: 6px 12px;
                font-size: 0.85rem;
                min-width: auto;
            }

            .empty-state {
                text-align: center;
                padding: 4rem 2rem;
                color: var(--color-text-muted);
            }
            .empty-icon {
                color: var(--color-border);
                margin-bottom: 1rem;
            }
            .loading,
            .error {
                text-align: center;
                padding: 2rem;
                font-weight: 700;
            }
            .error {
                color: var(--color-error);
            }
        `,
    ];

    constructor() {
        super();
        this.saves = [];
        this.loading = true;
        this.error = "";
        this.selectedRomId = null;
        this.selectedVersions = {};
    }

    async connectedCallback() {
        super.connectedCallback();
        await this.fetchSaves();
    }

    async fetchSaves() {
        try {
            this.loading = true;
            const res = await fetch("/api/v1/saves", {
                headers: { Authorization: await cookieStore.get("token").then((t) => t?.value) },
            });

            if (!res.ok) throw new Error("Failed to fetch saves data");

            const json = await res.json();

            const gamesMap = new Map();
            json.data.forEach((save) => {
                if (!gamesMap.has(save.rom_id)) {
                    gamesMap.set(save.rom_id, {
                        rom_id: save.rom_id,
                        title: save.realname || save.title,
                        console: save.console,
                        filesMap: new Map(),
                    });
                }

                let game = gamesMap.get(save.rom_id);
                if (!game.filesMap.has(save.file_name)) {
                    game.filesMap.set(save.file_name, {
                        file_name: save.file_name,
                        versions: [],
                    });
                }
                game.filesMap.get(save.file_name).versions.push(save);
            });

            this.saves = Array.from(gamesMap.values()).map((game) => {
                const filesArray = Array.from(game.filesMap.values());
                filesArray.sort((a, b) => a.file_name.localeCompare(b.file_name));
                filesArray.forEach((f) => f.versions.sort((a, b) => b.version_id - a.version_id));
                return {
                    ...game,
                    files: filesArray,
                };
            });
        } catch (e) {
            this.error = e.message;
        } finally {
            this.loading = false;
        }
    }

    selectGame(romId) {
        this.selectedRomId = romId;
    }

    clearSelection() {
        this.selectedRomId = null;
    }

    selectVersion(romId, fileName, versionId) {
        this.selectedVersions = {
            ...this.selectedVersions,
            [`${romId}_${fileName}`]: versionId,
        };
    }

    async downloadSave(romId, versionId, fileName) {
        try {
            const res = await fetch(`/api/v1/roms/${romId}/save/${versionId}/download`, {
                method: "POST",
                headers: {
                    "Content-Type": "application/json",
                    Authorization: await cookieStore.get("token").then((t) => t?.value),
                },
                body: JSON.stringify({ path: fileName }),
            });

            if (!res.ok) throw new Error("Download failed");

            const blob = await res.blob();
            const url = window.URL.createObjectURL(blob);
            const a = document.createElement("a");
            a.href = url;
            a.download = fileName;
            document.body.appendChild(a);
            a.click();
            window.URL.revokeObjectURL(url);
            document.body.removeChild(a);
        } catch (e) {
            alert("Failed to download save: " + e.message);
        }
    }

    firstUpdated() {
        this.updateIcons();
    }

    updated(changedProperties) {
        if (changedProperties.has("saves") || changedProperties.has("selectedRomId")) {
            this.updateIcons();
        }
    }

    updateIcons() {
        createIcons({
            icons: { Folder, File, Download, ChevronRight, ChevronDown, Gamepad2 },
            nameAttr: "data-lucide",
            root: this.shadowRoot,
        });
    }

    formatDate(dateString) {
        if (!dateString) return "Unknown date";
        const date = new Date(dateString);
        return date.toLocaleDateString(undefined, {
            year: "numeric",
            month: "short",
            day: "numeric",
            hour: "2-digit",
            minute: "2-digit",
        });
    }

    renderSelectedGame() {
        const game = this.saves.find((g) => g.rom_id === this.selectedRomId);
        if (!game) return html`<div class="error">Game not found</div>`;

        return html`
            <div style="margin-bottom: var(--spacing-sm);">
                <h2
                    style="margin: 0 0 var(--spacing-md) 0; display: flex; align-items: center; gap: var(--spacing-sm);"
                >
                    <i data-lucide="folder" style="color: var(--color-primary);"></i>
                    ${game.title}
                    <span class="game-console">${game.console}</span>
                </h2>
            </div>

            <div class="versions-list">
                ${game.files.map((fileGroup) => {
                    const defaultVersion = fileGroup.versions[0].version_id;
                    const selectedVersionId =
                        this.selectedVersions[`${game.rom_id}_${fileGroup.file_name}`] || defaultVersion;

                    return html`
                        <div class="version-item">
                            <div class="version-info">
                                <span class="version-name">
                                    <i data-lucide="file"></i>
                                    ${fileGroup.file_name}
                                </span>

                                <select
                                    class="version-select"
                                    @change=${(e) =>
                                        this.selectVersion(game.rom_id, fileGroup.file_name, e.target.value)}
                                >
                                    ${fileGroup.versions.map(
                                        (v) => html`
                                            <option
                                                value="${v.version_id}"
                                                ?selected=${v.version_id == selectedVersionId}
                                            >
                                                v${v.version_id} (${this.formatDate(v.created_at)})
                                            </option>
                                        `,
                                    )}
                                </select>
                            </div>

                            <button
                                class="popout-btn btn-fit btn-sm"
                                @click=${() => this.downloadSave(game.rom_id, selectedVersionId, fileGroup.file_name)}
                            >
                                <span class="btn-edge"></span>
                                <span class="btn-front" style="display: flex; align-items: center; gap: 6px;">
                                    <i data-lucide="download" style="width: 14px; height: 14px;"></i> Download
                                </span>
                            </button>
                        </div>
                    `;
                })}
            </div>
        `;
    }

    render() {
        return html`
            <emunex-navbar></emunex-navbar>
            <div class="auth-container">
                <div class="auth-card">
                    <header class="card-header">
                        ${this.selectedRomId
                            ? html`
                                  <a
                                      href="#"
                                      class="back-link"
                                      @click=${(e) => {
                                          e.preventDefault();
                                          this.clearSelection();
                                      }}
                                      >Back</a
                                  >
                              `
                            : ""}
                        <h1>Manage Saves</h1>
                    </header>

                    <div class="content">
                        ${this.loading ? html`<div class="loading">Loading your saves...</div>` : ""}
                        ${this.error ? html`<div class="error">${this.error}</div>` : ""}
                        ${!this.loading && !this.error
                            ? html`
                                  ${this.selectedRomId
                                      ? this.renderSelectedGame()
                                      : html`
                                            <div class="tree-container">
                                                ${this.saves.length === 0
                                                    ? html`
                                                          <div class="empty-state">
                                                              <i
                                                                  data-lucide="folder"
                                                                  class="empty-icon"
                                                                  style="width: 48px; height: 48px;"
                                                              ></i>
                                                              <h3>No Saves Found</h3>
                                                              <p
                                                                  >Play games to start synchronizing your saves to the
                                                                  cloud.</p
                                                              >
                                                          </div>
                                                      `
                                                    : this.saves.map(
                                                          (game) => html`
                                                              <div
                                                                  class="game-node"
                                                                  @click=${() => this.selectGame(game.rom_id)}
                                                              >
                                                                  <div class="game-header">
                                                                      <div class="icon-wrapper">
                                                                          <i data-lucide="folder"></i>
                                                                      </div>
                                                                      <span class="game-title">${game.title}</span>
                                                                      <span class="game-console">${game.console}</span>
                                                                      <i
                                                                          data-lucide="chevron-right"
                                                                          style="color: var(--color-text-muted); width: 20px; height: 20px;"
                                                                      ></i>
                                                                  </div>
                                                              </div>
                                                          `,
                                                      )}
                                            </div>
                                        `}
                              `
                            : ""}
                    </div>
                </div>
            </div>
        `;
    }
}

customElements.define("emunex-saves-manage-page", EmunexSavesManagePage);
