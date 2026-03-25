import { LitElement, css, html } from "lit";
import {
    baseTokens,
    buttonStyles,
    cardStyles,
    devTokens,
    formStyles,
    modalStyles,
    pageHostStyles,
    statusStyles,
    tableStyles,
    uploadZoneStyles,
} from "./components/shared-styles.js";

class EmunexRomsManagePage extends LitElement {
    static properties = {
        _roms: { type: Array, state: true },
        _filtered: { type: Array, state: true },
        _consoles: { type: Array, state: true },
        _loading: { type: Boolean, state: true },
        _error: { type: String, state: true },
        _status: { type: String, state: true },
        _statusType: { type: String, state: true },
        _editModalOpen: { type: Boolean, state: true },
        _editingRom: { type: Object, state: true },
        _editRomFileName: { type: String, state: true },
        _editImageFileName: { type: String, state: true },
        _dragoverRom: { type: Boolean, state: true },
        _dragoverImage: { type: Boolean, state: true },
        _previewImageUrl: { type: String, state: true },
    };

    static styles = [
        baseTokens,
        devTokens,
        pageHostStyles,
        cardStyles,
        formStyles,
        buttonStyles,
        tableStyles,
        statusStyles,
        uploadZoneStyles,
        modalStyles,
        css`
            .auth-container {
                max-width: 1000px;
            }
            .header-actions {
                display: flex;
                justify-content: space-between;
                align-items: center;
                margin-bottom: var(--spacing-md);
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
            .action-btns {
                display: flex;
                gap: var(--spacing-xs);
            }
            .action-btns .popout-btn {
                margin: 0;
            }
            .action-btns .btn-front {
                padding: 4px 10px;
                font-size: 0.75rem;
                min-width: 60px;
            }
            .modal-card {
                max-width: 800px;
            }
        `,
    ];

    constructor() {
        super();
        this._roms = [];
        this._filtered = [];
        this._consoles = [];
        this._loading = true;
        this._error = "";
        this._status = "";
        this._statusType = "";
        this._editModalOpen = false;
        this._editingRom = null;
        this._editRomFileName = "";
        this._editImageFileName = "";
        this._dragoverRom = false;
        this._dragoverImage = false;
        this._previewImageUrl = "";
        this._searchTimeout = null;
    }

    connectedCallback() {
        super.connectedCallback();
        this.fetchConsoles();
        this.fetchRoms();
    }

    async fetchConsoles() {
        try {
            const res = await fetch("/api/v1/consoles", { headers: { Authorization: localStorage.getItem("token") } });
            const json = await res.json();
            this._consoles = json.data || [];
        } catch { }
    }

    async fetchRoms() {
        this._loading = true;
        try {
            const res = await fetch("/api/v1/roms/list", { headers: { Authorization: localStorage.getItem("token") } });
            const json = await res.json();
            if (res.ok) {
                this._roms = json.data || [];
                this._filtered = this._roms;
            } else {
                this._error = json.error || "Failed to load";
            }
        } catch (err) {
            this._error = "Network error";
        } finally {
            this._loading = false;
        }
    }

    _handleSearch(e) {
        const q = e.target.value.toLowerCase();
        clearTimeout(this._searchTimeout);

        this._searchTimeout = setTimeout(() => {
            this.searchRoms(q);
        }, 300);
    }

    async searchRoms(query = "") {
        if (query === "") {
            await this.fetchRoms();
            return;
        }

        this._loading = true;
        this._error = "";
        try {
            const url = new URL("/api/v1/roms/search", window.location.origin);
            if (query) url.searchParams.set("query", query);

            const res = await fetch(url, {
                headers: { Authorization: localStorage.getItem("token") },
            });

            const json = await res.json();

            if (res.ok) {
                this._roms = json.data || [];
                this._filtered = [...this._roms];
            } else {
                this._error = json.error || "Failed to load";
                this._filtered = [];
            }
        } catch (err) {
            this._error = "Network error";
            this._filtered = [];
        } finally {
            this._loading = false;
        }
    }

    render() {
        return html`
            <div class="auth-container">
                <div class="auth-card">
                    <header class="card-header">
                        <a href="/dev" class="back-link">Back</a>
                        <h1>emuNEX</h1>
                    </header>

                    <div class="content">
                        <div class="header-actions">
                            <a href="/roms/upload" class="popout-btn btn-fit" style="text-decoration: none">
                                <span class="btn-edge"></span>
                                <span class="btn-front" style="padding: 6px 12px; font-size: 0.8rem; min-width: auto"
                                    >+ Upload ROM</span
                                >
                            </a>
                        </div>

                        <div class="form-group" style="margin-bottom: var(--spacing-md)">
                            <input
                                type="text"
                                placeholder="Search ROMs by title, ID, or console..."
                                style="font-weight: 700"
                                @input=${this._handleSearch}
                            />
                        </div>

                        <div style="overflow-x: auto">
                            <table style="margin-top: 0">
                                <thead>
                                    <tr><th>ID</th><th>Title</th><th>Console</th><th>Actions</th></tr>
                                </thead>
                                <tbody>
                                    ${this._loading
                ? html`<tr><td colspan="5" style="text-align: center">Loading...</td></tr>`
                : ""}
                                    ${this._error
                ? html`<tr
                                              ><td colspan="5" style="text-align: center; color: var(--color-error)"
                                                  >${this._error}</td
                                              ></tr
                                          >`
                : ""}
                                    ${!this._loading && !this._error && this._filtered.length === 0
                ? html`<tr><td colspan="5" style="text-align: center">No ROMs found.</td></tr>`
                : ""}
                                    ${this._filtered.map(
                    (rom) => html`
                                            <tr>
                                                <td style="font-family: monospace; font-size: 0.8rem">${rom.id}</td>
                                                <td style="font-weight: 700;">
                                                    ${rom.title}
                                                    ${rom.realname ? html`<br><span style="font-weight: normal; font-size: 0.8rem; color: var(--color-text-muted)">${rom.realname}</span>` : ""}
                                                </td>
                                                <td style="font-weight: 800;">${rom.console}</td>
                                                <td class="action-btns">
                                                    <button
                                                        class="popout-btn btn-fit btn-info"
                                                        @click=${() => this.openEdit(rom)}
                                                    >
                                                        <span class="btn-edge"></span
                                                        ><span class="btn-front">Edit</span>
                                                    </button>
                                                    <button
                                                        class="popout-btn btn-fit btn-error"
                                                        @click=${() => this.deleteRom(rom.id)}
                                                    >
                                                        <span class="btn-edge"></span
                                                        ><span class="btn-front">Delete</span>
                                                    </button>
                                                </td>
                                            </tr>
                                        `,
                )}
                                </tbody>
                            </table>
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

            ${this._editModalOpen ? this.renderModal() : ""}
        `;
    }

    renderModal() {
        return html`
            <div class="modal-backdrop open">
                <div class="modal-card">
                    <div class="content">
                        <div class="section-hint">Edit ROM Metadata</div>
                        <form id="editForm" @submit=${this.submitEdit}>
                            <div class="form-group">
                                <label>ID (Read-only)</label>
                                <input
                                    type="text"
                                    .value=${this._editingRom.id}
                                    readonly
                                    style="opacity: 0.7; cursor: not-allowed"
                                />
                            </div>
                            <div class="grid-2">
                                <div class="form-group">
                                    <label>Title</label>
                                    <input type="text" id="edit-title" .value=${this._editingRom.title} required />
                                </div>
                                <div class="form-group">
                                    <label>Real Name</label>
                                    <input type="text" id="edit-realname" .value=${this._editingRom.realname || ""} />
                                </div>
                            </div>

                            <div class="form-group">
                                <label>MD5 Hash (Hex)</label>
                                <input
                                    type="text"
                                    id="edit-md5"
                                    .value=${this._editingRom.md5_hash || ""}
                                    placeholder="1a2b3c4d..."
                                />
                            </div>

                            <div class="grid-3">
                                <div class="form-group">
                                    <label>Console</label>
                                    <select id="edit-console" .value=${this._editingRom.console} required>
                                        <option value="">select console</option>
                                        ${this._consoles.map(
            (c) => html`<option value=${c.name}>${c.name.toUpperCase()}</option>`,
        )}
                                    </select>
                                </div>
                                <div class="form-group">
                                    <label>Category</label>
                                    <input
                                        type="text"
                                        id="edit-category"
                                        .value=${this._editingRom.category || ""}
                                        required
                                    />
                                </div>
                                <div class="form-group">
                                    <label>Serial Number</label>
                                    <input type="text" id="edit-serial" .value=${this._editingRom.serial || ""} />
                                </div>
                            </div>

                            <div class="grid-2">
                                <div class="form-group">
                                    <label>Region</label>
                                    <input type="text" id="edit-region" .value=${this._editingRom.region || ""} />
                                </div>
                                <div class="form-group">
                                    <label>Release Year</label>
                                    <input type="number" id="edit-year" .value=${this._editingRom.release_year || ""} />
                                </div>
                            </div>

                            <div class="section-hint">Update Assets (Leave blank to keep current)</div>

                            <div class="form-group">
                                <label>Replace ROM File</label>
                                <label
                                    class="upload-zone ${this._dragoverRom ? "dragover" : ""}"
                                    @dragenter=${(e) => this._dragEnter(e, "rom")}
                                    @dragover=${(e) => this._dragEnter(e, "rom")}
                                    @dragleave=${(e) => this._dragLeave(e, "rom")}
                                    @drop=${(e) => this._drop(e, "edit-rom", "editRomFileName")}
                                >
                                    <div class="upload-icon">↑</div>
                                    <div class="upload-info">
                                        <div class="upload-text">Upload new ROM</div>
                                        <div class="file-name">${this._editRomFileName}</div>
                                    </div>
                                    <input
                                        type="file"
                                        id="edit-rom"
                                        style="display: none"
                                        @change=${(e) => this._fileChange(e, "editRomFileName")}
                                    />
                                </label>
                            </div>

                            <div class="form-group">
                                <label>Replace Cover Image</label>
                                <div style="display: flex; gap: var(--spacing-md); align-items: start;">
                                    ${this._previewImageUrl ? html`
                                        <img src=${this._previewImageUrl} style="width: 100px; height: 140px; object-fit: cover; border-radius: var(--radius-sm); border: 1px solid var(--color-border); flex-shrink: 0;" />
                                    ` : ""}
                                    <label
                                        class="upload-zone ${this._dragoverImage ? "dragover" : ""}"
                                        style="flex: 1; min-height: 140px; margin: 0;"
                                        @dragenter=${(e) => this._dragEnter(e, "image")}
                                        @dragover=${(e) => this._dragEnter(e, "image")}
                                        @dragleave=${(e) => this._dragLeave(e, "image")}
                                        @drop=${(e) => this._drop(e, "edit-image", "editImageFileName")}
                                    >
                                        <div class="upload-icon">↑</div>
                                        <div class="upload-info">
                                            <div class="upload-text">Upload new cover</div>
                                            <div class="file-name">${this._editImageFileName}</div>
                                        </div>
                                        <input
                                            type="file"
                                            id="edit-image"
                                            style="display: none"
                                            @change=${(e) => this._fileChange(e, "editImageFileName")}
                                        />
                                    </label>
                                </div>
                            </div>

                            <div style="display: flex; gap: var(--spacing-md); margin-top: var(--spacing-lg)">
                                <button type="submit" class="popout-btn btn-fit btn-info" style="flex: 1">
                                    <span class="btn-edge"></span
                                    ><span class="btn-front" style="padding: 10px">Save Changes</span>
                                </button>
                                <button
                                    type="button"
                                    class="popout-btn btn-fit btn-cancel"
                                    style="flex: 1"
                                    @click=${this.closeModal}
                                >
                                    <span class="btn-edge"></span
                                    ><span class="btn-front" style="padding: 10px">Cancel</span>
                                </button>
                            </div>
                        </form>
                    </div>
                </div>
            </div>
        `;
    }

    _dragEnter(e, z) {
        e.preventDefault();
        if (z === "rom") this._dragoverRom = true;
        else this._dragoverImage = true;
    }
    _dragLeave(e, z) {
        e.preventDefault();
        if (z === "rom") this._dragoverRom = false;
        else this._dragoverImage = false;
    }
    _drop(e, id, prop) {
        e.preventDefault();
        this._dragoverRom = false;
        this._dragoverImage = false;
        const input = this.renderRoot.querySelector("#" + id);
        input.files = e.dataTransfer.files;
        if (input.files.length) {
            this[`_${prop}`] = input.files[0].name;
            if (prop === "editImageFileName") {
                if (this._previewImageUrl && this._previewImageUrl.startsWith("blob:")) URL.revokeObjectURL(this._previewImageUrl);
                this._previewImageUrl = URL.createObjectURL(input.files[0]);
            }
        }
    }
    _fileChange(e, prop) {
        if (e.target.files.length) {
            this[`_${prop}`] = e.target.files[0].name;
            if (prop === "editImageFileName") {
                if (this._previewImageUrl && this._previewImageUrl.startsWith("blob:")) URL.revokeObjectURL(this._previewImageUrl);
                this._previewImageUrl = URL.createObjectURL(e.target.files[0]);
            }
        }
    }

    showStatus(msg, type) {
        this._status = msg;
        this._statusType = type;
        setTimeout(() => (this._status = ""), 3000);
    }

    async openEdit(rom) {
        this._editingRom = { ...rom };
        this._editRomFileName = "";
        this._editImageFileName = "";
        this._previewImageUrl = rom.image_path ? (rom.image_path.startsWith("blob:") ? rom.image_path : `/storage${rom.image_path}`) : "";
        this._editModalOpen = true;

        try {
            const res = await fetch(`/api/v1/roms/${encodeURIComponent(rom.id)}`, {
                headers: { Authorization: localStorage.getItem("token") }
            });
            if (res.ok) {
                const json = await res.json();
                this._editingRom = { ...rom, ...json.data };
                if (this._editingRom.image_path) {
                    this._previewImageUrl = this._editingRom.image_path.startsWith("blob:") ? this._editingRom.image_path : `/storage${this._editingRom.image_path}`;
                }
            }
        } catch (e) {
            console.error("Failed to load full rom metadata", e);
        }

        setTimeout(() => {
            const pConsole = this.renderRoot.querySelector("#edit-console");
            if (pConsole) pConsole.value = this._editingRom.console;
        }, 0);
    }

    closeModal() {
        this._editModalOpen = false;
    }

    async submitEdit(e) {
        e.preventDefault();
        const id = this._editingRom.id;
        const root = this.renderRoot;

        const reqData = {
            title: root.querySelector("#edit-title").value,
            realname: root.querySelector("#edit-realname").value || null,
            console: root.querySelector("#edit-console").value,
            category: root.querySelector("#edit-category").value,
            release_year: parseInt(root.querySelector("#edit-year").value) || null,
            region: root.querySelector("#edit-region").value || null,
            serial: root.querySelector("#edit-serial").value || null,
            md5_hash: root.querySelector("#edit-md5").value || null,
        };

        try {
            const res = await fetch(`/api/v1/roms/${encodeURIComponent(id)}`, {
                method: "PUT",
                headers: { Authorization: localStorage.getItem("token"), "Content-Type": "application/json" },
                body: JSON.stringify(reqData),
            });

            if (!res.ok) throw new Error((await res.json()).error || "Metadata update failed");

            const romInput = root.querySelector("#edit-rom");
            if (romInput.files.length) {
                const d = new FormData();
                d.append("rom", romInput.files[0]);
                const r2 = await fetch(`/api/v1/roms/${encodeURIComponent(id)}/file`, {
                    method: "POST",
                    headers: { Authorization: localStorage.getItem("token") },
                    body: d,
                });
                if (!r2.ok) throw new Error("ROM file update failed");
            }

            const imgInput = root.querySelector("#edit-image");
            if (imgInput.files.length) {
                const d = new FormData();
                d.append("image", imgInput.files[0]);
                const r3 = await fetch(`/api/v1/roms/${encodeURIComponent(id)}/image`, {
                    method: "POST",
                    headers: { Authorization: localStorage.getItem("token") },
                    body: d,
                });
                if (!r3.ok) throw new Error("Cover image update failed");
            }

            this.showStatus("ROM updated!", "success");
            this.closeModal();
            this.fetchRoms();
        } catch (err) {
            this.showStatus(err.message || "Update failed.", "error");
        }
    }

    async deleteRom(id) {
        if (!confirm(`Are you sure you want to delete ROM ${id}?`)) return;
        try {
            const res = await fetch(`/api/v1/roms/${encodeURIComponent(id)}`, {
                method: "DELETE",
                headers: { Authorization: localStorage.getItem("token") },
            });
            if (res.ok) {
                this.showStatus("ROM deleted.", "success");
                this.fetchRoms();
            } else {
                const err = await res.json();
                this.showStatus(`Error: ${err.error}`, "error");
            }
        } catch {
            this.showStatus("Deletion failed.", "error");
        }
    }
}

customElements.define("emunex-roms-manage-page", EmunexRomsManagePage);
