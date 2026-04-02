import { LitElement, css, html } from "lit";
import "../components/navbar.js";
import {
    baseTokens,
    buttonStyles,
    cardStyles,
    devTokens,
    formStyles,
    pageHostStyles,
    statusStyles,
    uploadZoneStyles,
} from "../components/shared-styles.js";
import "../theme-manager.js";

class EmunexRomPage extends LitElement {
    static properties = {
        _status: { type: String, state: true },
        _statusType: { type: String, state: true },
        _loading: { type: Boolean, state: true },
        _consoles: { type: Array, state: true },
        _dats: { type: Array, state: true },

        _romFileName: { type: String, state: true },
        _imageFileName: { type: String, state: true },
        _datFileName: { type: String, state: true },

        _noIntroGames: { type: Array, state: true },
        _currentDat: { type: String, state: true },
        _datNotes: { type: String, state: true },

        _noIntroStatus: { type: String, state: true },

        _dragoverRom: { type: Boolean, state: true },
        _dragoverImage: { type: Boolean, state: true },
        _dragoverDat: { type: Boolean, state: true },
        _previewImageUrl: { type: String, state: true },
    };

    static styles = [
        baseTokens,
        devTokens,
        pageHostStyles,
        cardStyles,
        formStyles,
        buttonStyles,
        statusStyles,
        uploadZoneStyles,
        css`
            .max-1200 {
                max-width: 1200px;
            }
            .grid-layout {
                display: grid;
                grid-template-columns: 1fr 1fr;
                gap: var(--spacing-xl);
                align-items: start;
            }
            .upload-form-left {
                background: rgba(255, 255, 255, 0.02);
                padding: var(--spacing-lg);
                border-radius: var(--radius-md);
                border: 1px solid var(--color-border);
            }
            .grid-3 {
                display: grid;
                grid-template-columns: 1fr 1fr 1fr;
                gap: var(--spacing-md);
            }
            .grid-2 {
                display: grid;
                grid-template-columns: 1fr 1fr;
                gap: var(--spacing-md);
            }

            .nointro-panel {
                background: rgba(var(--color-primary-rgb, 107 92 177) / 0.06);
                border: 1px solid rgba(var(--color-primary-rgb, 107 92 177) / 0.2);
                border-radius: var(--radius-md);
                padding: var(--spacing-md);
                display: flex;
                flex-direction: column;
                gap: var(--spacing-sm);
                height: 100%;
                background-color: var(--color-surface);
            }
            .nointro-game-list {
                max-height: 280px;
                overflow-y: auto;
                border: 1px solid var(--color-border);
                border-radius: var(--radius-sm);
                margin-top: var(--spacing-sm);
                flex: 1;
                min-height: 300px;
                background: var(--color-surface-variant);
            }
            .nointro-game-row {
                display: grid;
                grid-template-columns: 1fr auto auto max-content;
                gap: var(--spacing-sm);
                align-items: center;
                padding: var(--spacing-xs) var(--spacing-sm);
                border-bottom: 1px solid var(--color-border);
                font-size: 0.8rem;
            }
            .nointro-game-row:last-child {
                border-bottom: none;
            }
            .nointro-game-row:hover {
                background: var(--color-surface);
            }
            .nointro-game-name {
                font-weight: 700;
                overflow: hidden;
                text-overflow: ellipsis;
                white-space: nowrap;
            }
            .nointro-serial {
                font-family: monospace;
                color: var(--color-text-muted);
                font-size: 0.75rem;
            }
            .nointro-md5 {
                font-family: monospace;
                color: var(--color-text-muted);
                font-size: 0.65rem;
            }
            .nointro-empty {
                padding: var(--spacing-md);
                text-align: center;
                color: var(--color-text-muted);
                font-size: 0.85rem;
            }

            .import-row {
                display: grid;
                grid-template-columns: 1fr auto;
                gap: var(--spacing-sm);
                align-items: end;
            }

            @media (max-width: 900px) {
                .grid-layout {
                    grid-template-columns: 1fr;
                }
            }
        `,
    ];

    constructor() {
        super();
        this._status = "";
        this._statusType = "";
        this._loading = false;
        this._consoles = [];
        this._dats = [];
        this._romFileName = "";
        this._imageFileName = "";
        this._datFileName = "";
        this._noIntroGames = [];
        this._currentDat = "";
        this._datNotes = "";
        this._noIntroStatus = "";
        this._datNotesData = {};

        this._previewImageUrl = "";
        this._searchTimeout = null;
        this._notesTimeout = null;
    }

    connectedCallback() {
        super.connectedCallback();
        this.fetchConsoles();
        this.fetchDats();
    }

    async fetchConsoles() {
        try {
            const token = (await cookieStore.get("token"))?.value;
            const res = await fetch("/api/v1/consoles", { headers: { Authorization: token } });
            const json = await res.json();
            this._consoles = json.data || [];
        } catch (e) {
            console.error("Failed to load consoles", e);
        }
    }

    async fetchDats() {
        try {
            const token = (await cookieStore.get("token"))?.value;
            const res = await fetch("/api/v1/nointro/dats", {
                headers: { Authorization: token },
            });
            const json = await res.json();
            const list = json.data || [];
            this._dats = list.map((dat) => {
                if (typeof dat === "string") return { name: dat, notes: "" };
                return { name: dat.name || "Unknown", notes: dat.notes || "" };
            });
            list.forEach((dat) => {
                const name = typeof dat === "string" ? dat : dat.name;
                const notes = typeof dat === "string" ? "" : dat.notes;
                this._datNotesData[name] = notes || "";
            });
        } catch {}
    }

    render() {
        return html`
            <emunex-navbar></emunex-navbar>
            <div class="auth-container max-1200">
                <div class="auth-card" style="width: 100%">
                    <header class="card-header">
                        <a href="/roms" class="back-link">Back</a>
                        <h1>emuNEX</h1>
                    </header>

                    <div class="content">
                        <div class="grid-layout">
                            <form id="uploadForm" class="upload-form-left" @submit=${this._handleUploadSubmit}>
                                <div class="section-hint">Game Metadata</div>

                                <input type="hidden" id="title_id" name="title_id" />

                                <div class="grid-2">
                                    <div class="form-group">
                                        <label for="title">Title</label>
                                        <input
                                            type="text"
                                            id="title"
                                            name="title"
                                            placeholder="Metroid Fusion"
                                            required
                                        />
                                    </div>
                                    <div class="form-group">
                                        <label for="realname">Real Name (Optional)</label>
                                        <input
                                            type="text"
                                            id="realname"
                                            name="realname"
                                            placeholder="Full Retail Name"
                                        />
                                    </div>
                                </div>

                                <div class="form-group">
                                    <label for="md5_hash">MD5 Hash (Hex)</label>
                                    <input type="text" id="md5_hash" name="md5_hash" placeholder="1a2b3c4d..." />
                                </div>

                                <div class="grid-3">
                                    <div class="form-group">
                                        <label for="console">Console</label>
                                        <select id="console" name="console" required>
                                            <option value="">select console</option>
                                            ${this._consoles.map(
                                                (c) => html`<option value=${c.name}>${c.name.toUpperCase()}</option>`,
                                            )}
                                        </select>
                                    </div>
                                    <div class="form-group">
                                        <label for="category">Category</label>
                                        <input
                                            type="text"
                                            id="category"
                                            name="category"
                                            placeholder="Action"
                                            required
                                        />
                                    </div>
                                    <div class="form-group">
                                        <label for="serial">Serial Number</label>
                                        <input type="text" id="serial" name="serial" placeholder="BJBE" />
                                    </div>
                                </div>

                                <div class="grid-2">
                                    <div class="form-group">
                                        <label for="region">Region</label>
                                        <input type="text" id="region" name="region" placeholder="USA" />
                                    </div>
                                    <div class="form-group">
                                        <label for="release_year">Year</label>
                                        <input type="number" id="release_year" name="release_year" placeholder="2002" />
                                    </div>
                                </div>

                                <div class="section-hint">Assets</div>

                                <div class="form-group">
                                    <label>ROM File</label>
                                    <label
                                        class="upload-zone ${this._dragoverRom ? "dragover" : ""}"
                                        @dragenter=${(e) => this._dragEnter(e, "rom")}
                                        @dragover=${(e) => this._dragEnter(e, "rom")}
                                        @dragleave=${(e) => this._dragLeave(e, "rom")}
                                        @drop=${(e) => this._drop(e, "rom_file", "romFileName")}
                                    >
                                        <div class="upload-icon">↑</div>
                                        <div class="upload-info">
                                            <div class="upload-text">Upload ROM file</div>
                                            <div class="file-name">${this._romFileName}</div>
                                        </div>
                                        <input
                                            type="file"
                                            id="rom_file"
                                            name="rom_file"
                                            required
                                            style="display: none"
                                            @change=${(e) => this._fileChange(e, "romFileName")}
                                        />
                                    </label>
                                </div>

                                <div class="form-group">
                                    <label>Cover Image</label>
                                    <div style="display: flex; gap: var(--spacing-md); align-items: start;">
                                        ${this._previewImageUrl
                                            ? html`
                                                  <img
                                                      src=${this._previewImageUrl}
                                                      style="width: 100px; height: 140px; object-fit: cover; border-radius: var(--radius-sm); border: 1px solid var(--color-border); flex-shrink: 0;"
                                                  />
                                              `
                                            : ""}
                                        <label
                                            class="upload-zone ${this._dragoverImage ? "dragover" : ""}"
                                            style="flex: 1; min-height: 140px; margin: 0;"
                                            @dragenter=${(e) => this._dragEnter(e, "image")}
                                            @dragover=${(e) => this._dragEnter(e, "image")}
                                            @dragleave=${(e) => this._dragLeave(e, "image")}
                                            @drop=${(e) => this._drop(e, "image_file", "imageFileName")}
                                        >
                                            <div class="upload-icon">↑</div>
                                            <div class="upload-info">
                                                <div class="upload-text">Upload cover image</div>
                                                <div class="file-name">${this._imageFileName}</div>
                                            </div>
                                            <input
                                                type="file"
                                                id="image_file"
                                                name="image_file"
                                                required
                                                style="display: none"
                                                @change=${(e) => this._fileChange(e, "imageFileName")}
                                            />
                                        </label>
                                    </div>
                                </div>

                                <button type="submit" class="popout-btn" ?disabled=${this._loading}>
                                    <span class="btn-edge"></span>
                                    <span class="btn-front"
                                        >${this._loading ? "Processing…" : "Upload to Collection"}</span
                                    >
                                </button>
                            </form>

                            <div class="nointro-panel">
                                <div class="section-hint" style="color: var(--color-primary)">No-Intro Import</div>

                                <div class="import-row">
                                    <div class="form-group" style="margin: 0">
                                        <label
                                            class="upload-zone ${this._dragoverDat ? "dragover" : ""}"
                                            style="padding: var(--spacing-sm); min-height: auto"
                                            @dragenter=${(e) => this._dragEnter(e, "dat")}
                                            @dragover=${(e) => this._dragEnter(e, "dat")}
                                            @dragleave=${(e) => this._dragLeave(e, "dat")}
                                            @drop=${(e) => this._drop(e, "noIntroFile", "datFileName")}
                                        >
                                            <div class="upload-info" style="gap: 4px"
                                                ><div class="upload-text" style="font-size: 0.8rem"
                                                    >Drop .dat/.xml here</div
                                                >
                                                <div class="file-name" style="font-size: 0.75rem"
                                                    >${this._datFileName}</div
                                                ></div
                                            >
                                            <input
                                                type="file"
                                                id="noIntroFile"
                                                accept=".dat,.xml"
                                                style="display: none"
                                                @change=${(e) => this._fileChange(e, "datFileName")}
                                            />
                                        </label>
                                    </div>
                                    <button
                                        type="button"
                                        class="popout-btn btn-fit"
                                        style="margin: 0; min-width: 120px"
                                        @click=${this._importDat}
                                    >
                                        <span class="btn-edge"></span
                                        ><span class="btn-front" style="padding: 8px 14px; font-size: 0.8rem;"
                                            >Import Dump</span
                                        >
                                    </button>
                                </div>

                                <div class="form-group">
                                    <label>Platform</label>
                                    <select id="noIntroDatPicker" @change=${this._handleDatChange}>
                                        <option value="">— select a dat —</option>
                                        ${this._dats.map(
                                            (dat) => html`<option value="${dat.name}">${dat.name}</option>`,
                                        )}
                                    </select>
                                </div>

                                <div class="form-group">
                                    <input
                                        type="text"
                                        id="noIntroSearch"
                                        placeholder="Search games by title or serial..."
                                        @input=${this._handleDatSearch}
                                    />
                                </div>

                                <div class="nointro-game-list">
                                    ${!this._currentDat
                                        ? html`<div class="nointro-empty">Select a platform to browse games</div>`
                                        : this._noIntroGames.length === 0
                                          ? html`<div class="nointro-empty">No games found / Searching…</div>`
                                          : this._noIntroGames.map(
                                                (g, idx) => html`
                                                    <div class="nointro-game-row">
                                                        <span class="nointro-game-name" title="${g.name || "Unknown"}"
                                                            >${g.name || "Unknown"}</span
                                                        >
                                                        <span class="nointro-serial">${g.serial || "—"}</span>
                                                        <span class="nointro-md5">${(g.md5 || "").slice(0, 8)}…</span>
                                                        <button
                                                            class="popout-btn btn-fit"
                                                            @click=${() => this._selectDatGame(g)}
                                                        >
                                                            <span class="btn-edge"></span
                                                            ><span
                                                                class="btn-front"
                                                                style="padding: 6px 12px; font-size: 0.75rem;"
                                                                >Select</span
                                                            >
                                                        </button>
                                                    </div>
                                                `,
                                            )}
                                </div>

                                <div class="form-group" style="margin-top: var(--spacing-md)">
                                    <label>Admin Notes</label>
                                    <textarea
                                        id="datNotes"
                                        placeholder="Serial formats for this platform, known issues..."
                                        style="min-height: 120px; font-size: 0.85rem;"
                                        .value=${this._datNotes}
                                        @input=${this._handleNotesChange}
                                    ></textarea>
                                </div>

                                ${this._noIntroStatus
                                    ? html`
                                          <div class="status-box status-success" style="display: block"
                                              >${this._noIntroStatus}</div
                                          >
                                      `
                                    : ""}
                            </div>
                        </div>

                        ${this._status
                            ? html`
                                  <div
                                      class="status-box ${this._statusType === "error"
                                          ? "status-error"
                                          : "status-success"}"
                                      >${this._status}</div
                                  >
                              `
                            : ""}
                    </div>
                </div>
            </div>
        `;
    }

    _dragEnter(e, zone) {
        e.preventDefault();
        if (zone === "rom") this._dragoverRom = true;
        else if (zone === "image") this._dragoverImage = true;
        else this._dragoverDat = true;
    }
    _dragLeave(e, zone) {
        e.preventDefault();
        if (zone === "rom") this._dragoverRom = false;
        else if (zone === "image") this._dragoverImage = false;
        else this._dragoverDat = false;
    }
    _drop(e, inputId, propName) {
        e.preventDefault();
        this._dragoverRom = false;
        this._dragoverImage = false;
        this._dragoverDat = false;
        const input = this.renderRoot.querySelector("#" + inputId);
        input.files = e.dataTransfer.files;
        if (input.files.length) {
            this[`_${propName}`] = input.files[0].name;
            if (propName === "imageFileName") {
                if (this._previewImageUrl && this._previewImageUrl.startsWith("blob:"))
                    URL.revokeObjectURL(this._previewImageUrl);
                this._previewImageUrl = URL.createObjectURL(input.files[0]);
            }
        }
    }
    _fileChange(e, propName) {
        if (e.target.files.length) {
            this[`_${propName}`] = e.target.files[0].name;
            if (propName === "imageFileName") {
                if (this._previewImageUrl && this._previewImageUrl.startsWith("blob:"))
                    URL.revokeObjectURL(this._previewImageUrl);
                this._previewImageUrl = URL.createObjectURL(e.target.files[0]);
            }
        }
    }

    async _handleUploadSubmit(e) {
        e.preventDefault();
        this._loading = true;
        this._status = "Processing upload…";
        this._statusType = "success";

        const form = this.renderRoot.querySelector("#uploadForm");
        const formData = new FormData(form);

        if (!formData.get("title_id")) {
            const titleRaw = formData.get("title") || "";
            formData.set(
                "title_id",
                titleRaw
                    .replace(/\\s+/g, "_")
                    .replace(/[^a-zA-Z0-9_\\-]/g, "")
                    .slice(0, 64) || `ROM_${Date.now()}`,
            );
        }

        try {
            const token = (await cookieStore.get("token"))?.value;
            const response = await fetch("/api/v1/roms/upload", {
                method: "POST",
                body: formData,
                headers: { Authorization: token },
            });

            const result = await response.json();
            if (response.ok) {
                this._status = `Success! ROM ID: ${result.data}`;
                this._statusType = "success";
                form.reset();
                this._romFileName = "";
                this._imageFileName = "";
            } else {
                this._status = `Error: ${result.error || "Upload failed"}`;
                this._statusType = "error";
            }
        } catch (err) {
            this._status = "A network error occurred.";
            this._statusType = "error";
        } finally {
            this._loading = false;
        }
    }

    async _importDat() {
        const file = this.renderRoot.querySelector("#noIntroFile").files[0];
        if (!file) {
            this._noIntroStatus = "Pick a .dat file first";
            return;
        }

        const fd = new FormData();
        fd.append("dat_file", file);
        this._noIntroStatus = "Importing…";

        try {
            const token = (await cookieStore.get("token"))?.value;
            const res = await fetch("/api/v1/nointro/import", {
                method: "POST",
                headers: { Authorization: token },
                body: fd,
            });
            const json = await res.json();
            if (res.ok) {
                this._noIntroStatus = `✓ Imported ${json.data.imported} games`;
                await this.fetchDats();
            } else {
                this._noIntroStatus = `Error: ${json.error || "import failed"}`;
            }
        } catch {
            this._noIntroStatus = "Network error during import";
        }
    }

    _handleDatChange(e) {
        this._currentDat = e.target.value;
        const searchInput = this.renderRoot.querySelector("#noIntroSearch");
        if (searchInput) searchInput.value = "";
        this._datNotes = this._datNotesData[this._currentDat] || "";

        if (this._currentDat) {
            this._searchNoIntro("");
        } else {
            this._noIntroGames = [];
        }
    }

    _handleDatSearch(e) {
        const query = e.target.value;
        clearTimeout(this._searchTimeout);
        this._searchTimeout = setTimeout(() => {
            this._searchNoIntro(query);
        }, 300);
    }

    async _searchNoIntro(query) {
        if (!this._currentDat) return;
        this._noIntroGames = [];
        const params = new URLSearchParams({ dat: this._currentDat, limit: "100" });
        if (query) params.set("query", query);

        try {
            const token = (await cookieStore.get("token"))?.value;
            const res = await fetch(`/api/v1/nointro/search?${params}`, {
                headers: { Authorization: token },
            });
            const json = await res.json();
            this._noIntroGames = json.data || [];
        } catch {
            this._noIntroGames = [];
        }
    }

    _selectDatGame(game) {
        if (!game) return;
        const rr = this.renderRoot;
        rr.querySelector("#title").value = game.name || "";
        rr.querySelector("#realname").value = game.name || "";
        rr.querySelector("#title_id").value = game.id || "";
        rr.querySelector("#serial").value = game.serial || "";
        rr.querySelector("#md5_hash").value = game.md5 || "";

        if (this._currentDat) {
            rr.querySelector("#console").value = this._currentDat;
        }

        this._status = `Matched with No-Intro: ${game.name || "Unknown"}`;
        this._statusType = "success";
    }

    _handleNotesChange(e) {
        const notes = e.target.value;
        this._datNotes = notes;
        if (!this._currentDat) return;

        clearTimeout(this._notesTimeout);
        this._notesTimeout = setTimeout(async () => {
            this._datNotesData[this._currentDat] = notes;
            try {
                const token = (await cookieStore.get("token"))?.value;
                await fetch(`/api/v1/nointro/dats/${encodeURIComponent(this._currentDat)}/notes`, {
                    method: "POST",
                    headers: { Authorization: token, "Content-Type": "application/json" },
                    body: JSON.stringify({ notes }),
                });
            } catch (e) {
                console.error("Failed to save notes", e);
            }
        }, 1000);
    }
}

customElements.define("emunex-rom-page", EmunexRomPage);
