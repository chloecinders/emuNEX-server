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
    tagStyles,
    uploadZoneStyles,
} from "./components/shared-styles.js";

class EmunexEmulatorsManagePage extends LitElement {
    static properties = {
        _emulators: { type: Array, state: true },
        _filtered: { type: Array, state: true },
        _consoles: { type: Array, state: true },
        _loading: { type: Boolean, state: true },
        _error: { type: String, state: true },
        _status: { type: String, state: true },
        _statusType: { type: String, state: true },
        _editModalOpen: { type: Boolean, state: true },
        _editingEmulator: { type: Object, state: true },
        _editTags: { type: Array, state: true },
        _editConsoles: { type: Array, state: true },
        _editSaveExtensions: { type: Array, state: true },
        _editFileName: { type: String, state: true },
        _dragover: { type: Boolean, state: true },
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
        tagStyles,
        modalStyles,
        css`
            .auth-container {
                max-width: 900px;
            }
            .header-actions {
                display: flex;
                justify-content: space-between;
                align-items: center;
                margin-bottom: var(--spacing-md);
            }
            .grid-2 {
                display: grid;
                grid-template-columns: 1fr 1fr;
                gap: var(--spacing-md);
            }
            .checkbox-row {
                display: flex;
                align-items: center;
                gap: var(--spacing-sm);
                margin-bottom: var(--spacing-md);
            }
            .checkbox-row label {
                margin: 0;
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
                max-width: 600px;
            }
        `,
    ];

    constructor() {
        super();
        this._emulators = [];
        this._filtered = [];
        this._consoles = [];
        this._loading = true;
        this._error = "";
        this._status = "";
        this._statusType = "";
        this._editModalOpen = false;
        this._editingEmulator = null;
        this._editTags = [];
        this._editConsoles = [];
        this._editSaveExtensions = [];
        this._editFileName = "";
        this._dragover = false;
        this._searchTimeout = null;
    }

    connectedCallback() {
        super.connectedCallback();
        this.fetchConsoles();
        this.fetchEmulators();
    }

    async fetchConsoles() {
        try {
            const res = await fetch("/api/v1/consoles", { headers: { Authorization: localStorage.getItem("token") } });
            const json = await res.json();
            this._consoles = json.data || [];
        } catch (e) {
            console.error("Failed to load consoles", e);
        }
    }

    async fetchEmulators() {
        this._loading = true;
        try {
            const res = await fetch("/api/v1/emulators/all", {
                headers: { Authorization: localStorage.getItem("token") },
            });
            const json = await res.json();
            if (res.ok) {
                this._emulators = json.data || [];
                this._filtered = this._emulators;
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
            this._filtered = this._emulators.filter(
                (e) => e.name.toLowerCase().includes(q) || (e.consoles && e.consoles.some(c => c.toLowerCase().includes(q))),
            );
        }, 300);
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
                            <a href="/emulators/upload" class="popout-btn btn-fit" style="text-decoration: none">
                                <span class="btn-edge"></span>
                                <span class="btn-front" style="padding: 6px 12px; font-size: 0.8rem; min-width: auto"
                                    >+ Upload Emulator</span
                                >
                            </a>
                        </div>

                        <div class="form-group" style="margin-bottom: var(--spacing-md)">
                            <input
                                type="text"
                                placeholder="Search emulators by name..."
                                style="font-weight: 700"
                                @input=${this._handleSearch}
                            />
                        </div>

                        <div style="overflow-x: auto">
                            <table style="margin-top: 0">
                                <thead>
                                    <tr><th>ID</th><th>Name</th><th>Console</th><th>Platform</th><th>Actions</th></tr>
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
                                        ? html`<tr
                                              ><td colspan="5" style="text-align: center">No emulators found.</td></tr
                                          >`
                                        : ""}
                                    ${this._filtered.map(
                                        (emu) => html`
                                            <tr>
                                                <td
                                                    style="font-family: monospace; font-weight: 700; color: var(--color-text-muted);"
                                                    >#${emu.id}</td
                                                >
                                                <td style="font-weight: 700;">${emu.name}</td>
                                                <td style="font-weight: 800;">${(emu.consoles || []).map(c => c.toUpperCase()).join(", ")}</td>
                                                <td
                                                    style="text-transform: uppercase; font-size: 0.8rem; font-weight: 700;"
                                                    >${emu.platform}</td
                                                >
                                                <td class="action-btns">
                                                    <button
                                                        class="popout-btn btn-fit btn-info"
                                                        @click=${() => this.openEdit(emu)}
                                                    >
                                                        <span class="btn-edge"></span
                                                        ><span class="btn-front">Edit</span>
                                                    </button>
                                                    <button
                                                        class="popout-btn btn-fit btn-error"
                                                        @click=${() => this.deleteEmulator(emu.id)}
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
                        <div class="section-hint">Edit Emulator</div>
                        <form id="editForm" @submit=${this.submitEdit}>
                            <div class="form-group">
                                <label>Display Name</label>
                                <input type="text" id="edit-name" .value=${this._editingEmulator.name} required />
                            </div>

                            <div class="grid-2">
                                <div class="form-group">
                                    <label>Consoles (select at least one)</label>
                                    <div class="checkbox-grid" style="display: grid; grid-template-columns: repeat(2, 1fr); gap: 4px; margin-top: 4px;">
                                        ${this._consoles.map(
                                            (c) => html`
                                                <label class="checkbox-row" style="margin: 0; align-items: center; gap: 4px;">
                                                    <input type="checkbox" .checked=${this._editConsoles.includes(c.name)} @change=${(e) => this._toggleConsole(c.name, e.target.checked)}>
                                                    ${c.name.toUpperCase()}
                                                </label>
                                            `
                                        )}
                                    </div>
                                </div>
                                <div class="form-group">
                                    <label>Target Platform</label>
                                    <select id="edit-platform" .value=${this._editingEmulator.platform} required>
                                        <option value="windows">Windows</option>
                                        <option value="linux">Linux</option>
                                        <option value="android">Android</option>
                                    </select>
                                </div>
                            </div>

                            <div class="form-group">
                                <label>Run Command (with args)</label>
                                <input
                                    type="text"
                                    id="edit-command"
                                    .value=${this._editingEmulator.run_command || ""}
                                    placeholder="emu.exe {rom}"
                                />
                            </div>

                            <div class="form-group">
                                <label>Binary Name</label>
                                <input
                                    type="text"
                                    id="edit-binary-name"
                                    .value=${this._editingEmulator.binary_name || ""}
                                    placeholder="emulator.exe"
                                />
                            </div>

                            <div class="form-group">
                                <label>Save Path (optional)</label>
                                <input
                                    type="text"
                                    id="edit-save-path"
                                    .value=${this._editingEmulator.save_path || ""}
                                    placeholder="/saves/$rom_name.sav"
                                />
                            </div>

                            <div class="form-group">
                                <label>Configuration Files (press Enter to add)</label>
                                <div class="tag-system">
                                    ${this._editTags.map(
                                        (t) =>
                                            html`<div class="tag"
                                                >${t}
                                                <span class="tag-remove" @click=${() => this._removeTag(t)}
                                                    >×</span
                                                ></div
                                            >`,
                                    )}
                                    <div class="tag-input-wrapper">
                                        <input
                                            type="text"
                                            placeholder="config.ini"
                                            @keydown=${this._handleTagKeydown}
                                        />
                                    </div>
                                </div>
                            </div>

                            <div class="form-group">
                                <label>Save File Extensions (press Enter to add, e.g. <code>.sra</code>, <code>.srm</code>)</label>
                                <div class="tag-system">
                                    ${this._editSaveExtensions.map(
                                        (t) =>
                                            html`<div class="tag"
                                                >${t}
                                                <span class="tag-remove" @click=${() => this._removeSaveExt(t)}
                                                    >×</span
                                                ></div
                                            >`,
                                    )}
                                    <div class="tag-input-wrapper">
                                        <input
                                            type="text"
                                            placeholder=".sra"
                                            @keydown=${(e) => this._handleSaveExtKeydown(e)}
                                        />
                                    </div>
                                </div>
                            </div>

                            <div class="checkbox-row">
                                <input type="checkbox" id="edit-zipped" .checked=${this._editingEmulator.zipped} />
                                <label>Executable is zipped (contains DLLs/assets)</label>
                            </div>

                            <div class="form-group">
                                <label>Emulator Binary</label>
                                <label
                                    class="upload-zone ${this._dragover ? "dragover" : ""}"
                                    @dragenter=${this._dragEnter}
                                    @dragover=${this._dragOver}
                                    @dragleave=${this._dragLeave}
                                    @drop=${this._drop}
                                >
                                    <div class="upload-icon">↑</div>
                                    <div class="upload-info">
                                        <div class="upload-text">Upload emulator bundle</div>
                                        <div class="file-name">${this._editFileName}</div>
                                    </div>
                                    <input
                                        type="file"
                                        id="edit-binary"
                                        style="display: none"
                                        @change=${this._fileChange}
                                    />
                                </label>
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

    _handleTagKeydown(e) {
        if (e.key === "Enter" && e.target.value.trim()) {
            e.preventDefault();
            const val = e.target.value.trim();
            if (!this._editTags.includes(val)) this._editTags = [...this._editTags, val];
            e.target.value = "";
        }
    }

    _removeTag(t) {
        this._editTags = this._editTags.filter((tag) => tag !== t);
    }

    _handleSaveExtKeydown(e) {
        if (e.key === "Enter" && e.target.value.trim()) {
            e.preventDefault();
            const val = e.target.value.trim();
            if (!this._editSaveExtensions.includes(val)) this._editSaveExtensions = [...this._editSaveExtensions, val];
            e.target.value = "";
        }
    }

    _removeSaveExt(t) {
        this._editSaveExtensions = this._editSaveExtensions.filter((ext) => ext !== t);
    }

    _toggleConsole(name, checked) {
        if (checked) {
            if (!this._editConsoles.includes(name)) this._editConsoles = [...this._editConsoles, name];
        } else {
            this._editConsoles = this._editConsoles.filter(c => c !== name);
        }
    }

    _dragEnter(e) {
        e.preventDefault();
        this._dragover = true;
    }
    _dragOver(e) {
        e.preventDefault();
        this._dragover = true;
    }
    _dragLeave(e) {
        e.preventDefault();
        this._dragover = false;
    }
    _drop(e) {
        e.preventDefault();
        this._dragover = false;
        const input = this.renderRoot.querySelector("#edit-binary");
        input.files = e.dataTransfer.files;
        if (input.files.length) this._editFileName = input.files[0].name;
    }
    _fileChange(e) {
        if (e.target.files.length) this._editFileName = e.target.files[0].name;
    }

    showStatus(msg, type) {
        this._status = msg;
        this._statusType = type;
        setTimeout(() => (this._status = ""), 3000);
    }

    openEdit(emu) {
        this._editingEmulator = { ...emu };
        this._editTags = [...(emu.config_files || [])];
        this._editConsoles = [...(emu.consoles || [])];
        this._editSaveExtensions = [...(emu.save_extensions || [])];
        this._editFileName = "";
        this._editModalOpen = true;

        // Slight hack for select sync timing since Lit updates are async
        setTimeout(() => {
            const pPlatform = this.renderRoot.querySelector("#edit-platform");
            if (pPlatform) pPlatform.value = emu.platform;
        }, 0);
    }

    closeModal() {
        this._editModalOpen = false;
    }

    async submitEdit(e) {
        e.preventDefault();
        const id = this._editingEmulator.id;
        const root = this.renderRoot;

        const updateData = {
            name: root.querySelector("#edit-name").value,
            consoles: this._editConsoles,
            platform: root.querySelector("#edit-platform").value,
            run_command: root.querySelector("#edit-command").value,
            binary_name: root.querySelector("#edit-binary-name").value,
            save_path: root.querySelector("#edit-save-path").value,
            save_extensions: this._editSaveExtensions,
            config_files: this._editTags,
            zipped: root.querySelector("#edit-zipped").checked,
        };

        try {
            const response = await fetch(`/api/v1/emulators/${id}`, {
                method: "PUT",
                headers: { Authorization: localStorage.getItem("token"), "Content-Type": "application/json" },
                body: JSON.stringify(updateData),
            });

            if (!response.ok) {
                const err = await response.json();
                throw new Error(err.error || "Metadata update failed");
            }

            const binaryInput = root.querySelector("#edit-binary");
            if (binaryInput.files.length > 0) {
                const formData = new FormData();
                formData.append("binary", binaryInput.files[0]);
                const bRes = await fetch(`/api/v1/emulators/${id}/binary`, {
                    method: "POST",
                    headers: { Authorization: localStorage.getItem("token") },
                    body: formData,
                });
                if (!bRes.ok) throw new Error("Binary upload failed");
            }

            this.showStatus("Emulator updated!", "success");
            this.closeModal();
            this.fetchEmulators();
        } catch (err) {
            this.showStatus(err.message || "Update failed.", "error");
        }
    }

    async deleteEmulator(id) {
        if (!confirm(`Are you sure you want to delete emulator #${id}?`)) return;
        try {
            const res = await fetch(`/api/v1/emulators/${id}`, {
                method: "DELETE",
                headers: { Authorization: localStorage.getItem("token") },
            });
            if (res.ok) {
                this.showStatus("Emulator deleted.", "success");
                this.fetchEmulators();
            } else {
                const err = await res.json();
                this.showStatus(`Error: ${err.error}`, "error");
            }
        } catch {
            this.showStatus("Deletion failed.", "error");
        }
    }
}

customElements.define("emunex-emulators-manage-page", EmunexEmulatorsManagePage);
