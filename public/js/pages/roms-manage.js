import { LitElement, css, html } from "lit";
import "../components/navbar.js";
import "../components/rom-edit-form.js";
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
} from "../components/shared-styles.js";
import "../theme-manager.js";

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
                white-space: nowrap;
                font-size: 0;
            }
            .action-btns .popout-btn {
                margin: 4px var(--spacing-xs) 0 0;
                display: inline-block;
                vertical-align: middle;
            }
            .action-btns .popout-btn:last-child {
                margin-right: 0;
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
        /** @type {never[]} we don't use typescript here so why does lit complain anyway?? */
        this._consoles = [];
        this._loading = true;
        this._error = "";
        this._status = "";
        this._statusType = "";
        this._editModalOpen = false;
        this._editingRom = null;
        this._searchTimeout = null;
        this._currentQuery = "";
    }

    connectedCallback() {
        super.connectedCallback();
        this.fetchConsoles();
        this.fetchRoms();
    }

    async fetchConsoles() {
        try {
            const token = (await cookieStore.get("token"))?.value;
            const res = await fetch("/api/v1/consoles", { headers: { Authorization: token } });
            const json = await res.json();
            this._consoles = json.data || [];
        } catch {}
    }

    async fetchRoms() {
        this._loading = true;
        try {
            const token = (await cookieStore.get("token"))?.value;
            const res = await fetch("/api/v1/roms/list", { headers: { Authorization: token } });
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
        this._currentQuery = q;
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

            const token = (await cookieStore.get("token"))?.value;
            const res = await fetch(url, {
                headers: { Authorization: token },
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
            <emunex-navbar></emunex-navbar>
            <div class="auth-container">
                <div class="auth-card">
                    <header class="card-header">
                        <a href="/dev" class="back-link">Back</a>
                        <h1>emuNEX</h1>
                    </header>

                    <div class="content">
                        <div class="header-actions">
                            <div style="display: flex; gap: var(--spacing-sm);">
                                <a href="/roms/upload" class="popout-btn btn-fit" style="text-decoration: none">
                                    <span class="btn-edge"></span>
                                    <span
                                        class="btn-front"
                                        style="padding: 6px 12px; font-size: 0.8rem; min-width: auto"
                                        >+ Upload ROM</span
                                    >
                                </a>
                                <a
                                    href="/roms/bulk_upload"
                                    class="popout-btn btn-fit btn-info"
                                    style="text-decoration: none"
                                >
                                    <span class="btn-edge"></span>
                                    <span
                                        class="btn-front"
                                        style="padding: 6px 12px; font-size: 0.8rem; min-width: auto"
                                        >Bulk Import</span
                                    >
                                </a>
                            </div>
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
                                                    ${rom.realname
                                                        ? html`<br /><span
                                                                  style="font-weight: normal; font-size: 0.8rem; color: var(--color-text-muted)"
                                                                  >${rom.realname}</span
                                                              >`
                                                        : ""}
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
                        <emunex-rom-edit-form
                            .rom=${this._editingRom}
                            .consoles=${this._consoles}
                            @saved=${this._onSaved}
                            @cancel=${this.closeModal}
                        ></emunex-rom-edit-form>
                    </div>
                </div>
            </div>
        `;
    }

    _onSaved(e) {
        this.closeModal();
        if (this._currentQuery) this.searchRoms(this._currentQuery);
        else this.fetchRoms();
        this.showStatus("ROM updated successfully!", "success");
    }

    showStatus(msg, type) {
        this._status = msg;
        this._statusType = type;
        setTimeout(() => (this._status = ""), 3000);
    }

    async openEdit(rom) {
        this._editingRom = { ...rom };
        this._editModalOpen = true;

        try {
            const token = (await cookieStore.get("token"))?.value;
            const res = await fetch(`/api/v1/roms/${encodeURIComponent(rom.id)}`, {
                headers: { Authorization: token },
            });
            if (res.ok) {
                const json = await res.json();
                this._editingRom = { ...rom, ...json.data };
            }
        } catch (e) {
            console.error("Failed to load full rom metadata", e);
        }
    }

    closeModal() {
        this._editModalOpen = false;
    }

    async deleteRom(id) {
        if (!confirm(`Are you sure you want to delete ROM ${id}?`)) return;
        try {
            const token = (await cookieStore.get("token"))?.value;
            const res = await fetch(`/api/v1/roms/${encodeURIComponent(id)}`, {
                method: "DELETE",
                headers: { Authorization: token },
            });
            if (res.ok) {
                this.showStatus("ROM deleted.", "success");
                if (this._currentQuery) this.searchRoms(this._currentQuery);
                else this.fetchRoms();
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
