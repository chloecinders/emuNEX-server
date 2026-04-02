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
    tableStyles,
} from "../components/shared-styles.js";
import "../theme-manager.js";

class EmunexSearchSectionsPage extends LitElement {
    static properties = {
        _sections: { type: Array, state: true },
        _categories: { type: Array, state: true },
        _loading: { type: Boolean, state: true },
        _error: { type: String, state: true },
        _status: { type: String, state: true },
        _statusType: { type: String, state: true },
        _editingSectionId: { type: String, state: true },
        _isCreating: { type: Boolean, state: true },

        _editTitle: { type: String, state: true },
        _editType: { type: String, state: true },
        _editSmartFilter: { type: String, state: true },
        _editFilterValue: { type: String, state: true },
        _editRoms: { type: Array, state: true },

        _romSearchQuery: { type: String, state: true },
        _romSearchResults: { type: Array, state: true },
        _isSearchingRoms: { type: Boolean, state: true },
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
            .content-section {
                margin-bottom: var(--spacing-xxl);
            }
            .editor-panel {
                background: var(--color-surface-variant);
                border: 1px solid var(--color-border);
                border-radius: var(--radius-md);
                padding: var(--spacing-lg);
                margin-bottom: var(--spacing-lg);
            }
            .form-grid {
                display: grid;
                grid-template-columns: 1fr 1fr;
                gap: var(--spacing-md);
                margin-bottom: var(--spacing-md);
            }
            .form-group {
                display: flex;
                flex-direction: column;
                gap: var(--spacing-xs);
            }
            .rom-list-container {
                border: 1px solid var(--color-border);
                border-radius: var(--radius-sm);
                max-height: 200px;
                overflow-y: auto;
                background: var(--color-background);
            }
            .rom-list-item {
                display: flex;
                justify-content: space-between;
                align-items: center;
                padding: var(--spacing-sm);
                border-bottom: 1px solid var(--color-border);
            }
            .rom-list-item:last-child {
                border-bottom: none;
            }
            .search-results {
                position: absolute;
                background: var(--color-surface-variant);
                border: 1px solid var(--color-border);
                border-radius: var(--radius-md);
                width: 100%;
                max-height: 250px;
                overflow-y: auto;
                z-index: 10;
                box-shadow: var(--shadow-lg);
            }
            .search-result-item {
                padding: var(--spacing-sm);
                cursor: pointer;
                border-bottom: 1px solid var(--color-border);
            }
            .search-result-item:hover {
                background: var(--color-primary-light);
            }
            .order-controls {
                display: flex;
                gap: 4px;
            }
        `,
    ];

    constructor() {
        super();
        this._sections = [];
        this._categories = [];
        this._loading = true;
        this._error = "";
        this._status = "";
        this._statusType = "";
        this._editingSectionId = null;
        this._isCreating = false;

        this._editTitle = "";
        this._editType = "smart";
        this._editSmartFilter = "most_played";
        this._editFilterValue = "";
        this._editRoms = [];

        this._romSearchQuery = "";
        this._romSearchResults = [];
        this._isSearchingRoms = false;
        this._searchTimeout = null;
    }

    connectedCallback() {
        super.connectedCallback();
        this.fetchData();
        this.fetchCategories();
    }

    async fetchData() {
        this._loading = true;
        try {
            const token = (await cookieStore.get("token"))?.value;
            const hdrs = { Authorization: token };
            const res = await fetch("/api/v1/search_sections", { headers: hdrs });
            const json = await res.json();

            if (res.ok) {
                this._sections = (json.data || []).sort((a, b) => a.order_index - b.order_index);
            } else {
                this._error = json.error || "Failed to load sections";
            }
        } catch {
            this._error = "Network error loading data";
        } finally {
            this._loading = false;
        }
    }

    async fetchCategories() {
        try {
            const token = (await cookieStore.get("token"))?.value;
            const res = await fetch("/api/v1/roms/categories", {
                headers: { Authorization: token },
            });
            const json = await res.json();
            if (res.ok) {
                this._categories = json.data || [];
                if (this._categories.length > 0 && !this._editFilterValue) {
                    this._editFilterValue = this._categories[0].name;
                }
            }
        } catch (e) {
            console.error("Failed to fetch categories", e);
        }
    }

    showStatus(msg, type) {
        this._status = msg;
        this._statusType = type;
        setTimeout(() => (this._status = ""), 3000);
    }

    _startCreate() {
        this._isCreating = true;
        this._editingSectionId = null;
        this._editTitle = "New Section";
        this._editType = "smart";
        this._editSmartFilter = "most_played";
        this._editFilterValue = this._categories.length > 0 ? this._categories[0].name : "";
        this._editRoms = [];
    }

    _startEdit(s) {
        this._isCreating = false;
        this._editingSectionId = s.id;
        this._editTitle = s.title;
        this._editType = s.section_type;
        this._editSmartFilter = s.smart_filter || "most_played";
        this._editFilterValue = s.filter_value || (this._categories.length > 0 ? this._categories[0].name : "");
        this._editRoms = s.roms ? [...s.roms] : [];

        if (this._editRoms.length > 0) {
            this._loadFullEditRoms();
        }
    }

    async _loadFullEditRoms() {
        const fullRoms = [];
        const token = (await cookieStore.get("token"))?.value;
        for (const id of this._editRoms) {
            try {
                const res = await fetch(`/api/v1/roms/${id}`, {
                    headers: { Authorization: token },
                });
                if (res.ok) {
                    const json = await res.json();
                    fullRoms.push(json.data);
                }
            } catch (e) {
                console.error("Failed fetching rom details for ID", id);
            }
        }
        this._editRoms = fullRoms;
    }

    _cancelEdit() {
        this._isCreating = false;
        this._editingSectionId = null;
    }

    async _saveSection() {
        const isCustom = this._editType === "custom";
        const isSmart = this._editType === "smart";
        const isCat = this._editType === "category";

        const data = {
            title: this._editTitle,
            section_type: this._editType,
            smart_filter: isSmart ? this._editSmartFilter : null,
            filter_value: isCat ? this._editFilterValue : null,
            order_index: this._isCreating
                ? this._sections.length
                : this._sections.find((s) => s.id === this._editingSectionId).order_index,
            roms: isCustom ? this._editRoms.map((r) => (typeof r === "string" ? r : r.id)) : null,
        };

        const url = this._isCreating ? "/api/v1/search_sections" : `/api/v1/search_sections/${this._editingSectionId}`;
        const method = this._isCreating ? "POST" : "PUT";

        try {
            const token = (await cookieStore.get("token"))?.value;
            const res = await fetch(url, {
                method,
                headers: { Authorization: token, "Content-Type": "application/json" },
                body: JSON.stringify(data),
            });

            if (res.ok) {
                this.showStatus("Section saved.", "success");
                this._cancelEdit();
                this.fetchData();
            } else {
                const err = await res.json();
                this.showStatus(`Error: ${err.error}`, "error");
            }
        } catch {
            this.showStatus("Failed to save section", "error");
        }
    }

    async _deleteSection(id) {
        if (!confirm("Are you sure you want to delete this section?")) return;
        try {
            const token = (await cookieStore.get("token"))?.value;
            const res = await fetch(`/api/v1/search_sections/${id}`, {
                method: "DELETE",
                headers: { Authorization: token },
            });
            if (res.ok) {
                this.showStatus("Section deleted.", "success");
                this.fetchData();
            } else {
                const err = await res.json();
                this.showStatus(`Error: ${err.error}`, "error");
            }
        } catch {
            this.showStatus("Deletion failed.", "error");
        }
    }

    async _moveSection(index, direction) {
        if (index + direction < 0 || index + direction >= this._sections.length) return;

        const newSections = [...this._sections];
        const temp = newSections[index];
        newSections[index] = newSections[index + direction];
        newSections[index + direction] = temp;

        const updates = newSections.map((s, i) => [s.id, i]);

        try {
            const token = (await cookieStore.get("token"))?.value;
            const res = await fetch("/api/v1/search_sections/order", {
                method: "PUT",
                headers: { Authorization: token, "Content-Type": "application/json" },
                body: JSON.stringify({ updates }),
            });
            if (res.ok) {
                this.fetchData();
            } else {
                this.showStatus("Failed to update order", "error");
            }
        } catch {
            this.showStatus("Network error updating order", "error");
        }
    }

    _onSearchInput(e) {
        this._romSearchQuery = e.target.value;
        if (this._searchTimeout) clearTimeout(this._searchTimeout);

        if (!this._romSearchQuery.trim()) {
            this._romSearchResults = [];
            return;
        }

        this._searchTimeout = setTimeout(() => this._performSearch(), 400);
    }

    async _performSearch() {
        this._isSearchingRoms = true;
        try {
            const token = (await cookieStore.get("token"))?.value;
            const res = await fetch(`/api/v1/roms/search?query=${encodeURIComponent(this._romSearchQuery)}&limit=10`, {
                headers: { Authorization: token },
            });
            if (res.ok) {
                const json = await res.json();
                this._romSearchResults = json.data || [];
            }
        } catch (e) {
            console.error("Search failed:", e);
        } finally {
            this._isSearchingRoms = false;
        }
    }

    _addRomToCustom(rom) {
        if (!this._editRoms.find((r) => r.id === rom.id)) {
            this._editRoms = [...this._editRoms, rom];
        }
        this._romSearchQuery = "";
        this._romSearchResults = [];
    }

    _removeRomFromCustom(id) {
        this._editRoms = this._editRoms.filter((r) => r.id !== id);
    }

    _moveRom(index, direction) {
        if (index + direction < 0 || index + direction >= this._editRoms.length) return;
        const newRoms = [...this._editRoms];
        const temp = newRoms[index];
        newRoms[index] = newRoms[index + direction];
        newRoms[index + direction] = temp;
        this._editRoms = newRoms;
    }

    renderEditor() {
        if (!this._editingSectionId && !this._isCreating) return "";

        return html`
            <div class="editor-panel">
                <h2 style="margin-bottom: var(--spacing-md)"
                    >${this._isCreating ? "Create Section" : "Edit Section"}</h2
                >

                <div class="form-grid">
                    <div class="form-group">
                        <label>Title</label>
                        <input
                            type="text"
                            .value=${this._editTitle}
                            @input=${(e) => (this._editTitle = e.target.value)}
                        />
                    </div>
                    <div class="form-group">
                        <label>Section Type</label>
                        <select .value=${this._editType} @change=${(e) => (this._editType = e.target.value)}>
                            <option value="smart">Smart Group</option>
                            <option value="category">Category Filter</option>
                            <option value="custom">Custom ROM List</option>
                        </select>
                    </div>
                </div>

                ${this._editType === "smart"
                    ? html`
                          <div class="form-group" style="margin-bottom: var(--spacing-md)">
                              <label>Filter Type</label>
                              <select
                                  .value=${this._editSmartFilter}
                                  @change=${(e) => (this._editSmartFilter = e.target.value)}
                              >
                                  <option value="most_played">Most Played</option>
                                  <option value="recently_added">Recently Added</option>
                              </select>
                          </div>
                      `
                    : ""}
                ${this._editType === "category"
                    ? html`
                          <div class="form-group" style="margin-bottom: var(--spacing-md)">
                              <label>Category</label>
                              <select
                                  .value=${this._editFilterValue}
                                  @change=${(e) => (this._editFilterValue = e.target.value)}
                              >
                                  ${this._categories.map((c) => html`<option value="${c.name}">${c.name}</option>`)}
                              </select>
                          </div>
                      `
                    : ""}
                ${this._editType === "custom"
                    ? html`
                          <div class="form-group" style="margin-bottom: var(--spacing-md)">
                              <label>Add Specific Games</label>
                              <div style="position: relative;">
                                  <input
                                      type="text"
                                      placeholder="Search by title..."
                                      .value=${this._romSearchQuery}
                                      @input=${this._onSearchInput}
                                  />
                                  ${this._romSearchResults.length > 0
                                      ? html`
                                            <div class="search-results">
                                                ${this._romSearchResults.map(
                                                    (r) => html`
                                                        <div
                                                            class="search-result-item"
                                                            @click=${() => this._addRomToCustom(r)}
                                                        >
                                                            <strong>${r.title}</strong> (${r.console})
                                                        </div>
                                                    `,
                                                )}
                                            </div>
                                        `
                                      : ""}
                              </div>
                          </div>

                          <div class="form-group" style="margin-bottom: var(--spacing-md)">
                              <label>Configured Games</label>
                              <div class="rom-list-container">
                                  ${this._editRoms.length === 0
                                      ? html`<div style="padding: var(--spacing-sm); color: var(--color-text-muted)"
                                            >No games strictly specified.</div
                                        >`
                                      : ""}
                                  ${this._editRoms.map(
                                      (r, idx) => html`
                                          <div class="rom-list-item">
                                              <span><strong>${r.title}</strong> (${r.console})</span>
                                              <div class="order-controls">
                                                  <button
                                                      class="popout-btn btn-fit btn-info"
                                                      @click=${() => this._moveRom(idx, -1)}
                                                      ?disabled=${idx === 0}
                                                      >↑</button
                                                  >
                                                  <button
                                                      class="popout-btn btn-fit btn-info"
                                                      @click=${() => this._moveRom(idx, 1)}
                                                      ?disabled=${idx === this._editRoms.length - 1}
                                                      >↓</button
                                                  >
                                                  <button
                                                      class="popout-btn btn-fit btn-error"
                                                      @click=${() => this._removeRomFromCustom(r.id)}
                                                      >X</button
                                                  >
                                              </div>
                                          </div>
                                      `,
                                  )}
                              </div>
                          </div>
                      `
                    : ""}

                <div class="action-btns" style="margin-top: var(--spacing-lg)">
                    <button class="popout-btn btn-success" @click=${this._saveSection}>
                        <span class="btn-edge"></span><span class="btn-front">Save Section</span>
                    </button>
                    <button class="popout-btn btn-cancel" @click=${this._cancelEdit}>
                        <span class="btn-edge"></span><span class="btn-front">Cancel</span>
                    </button>
                </div>
            </div>
        `;
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
                        ${this._status
                            ? html`
                                  <div
                                      class="status-box ${this._statusType === "error"
                                          ? "status-error"
                                          : "status-success"}"
                                  >
                                      ${this._status}
                                  </div>
                              `
                            : ""}
                        ${this.renderEditor()}

                        <div class="content-section">
                            <div class="header-actions">
                                <div class="section-hint" style="margin: 0">Active Sections</div>
                                <button
                                    class="popout-btn btn-fit btn-success"
                                    @click=${this._startCreate}
                                    ?disabled=${this._isCreating || this._editingSectionId}
                                >
                                    <span class="btn-edge"></span
                                    ><span class="btn-front" style="padding: 6px 12px; font-size: 0.8rem;"
                                        >+ New Section</span
                                    >
                                </button>
                            </div>

                            <div style="overflow-x: auto">
                                <table style="margin-top: 0">
                                    <thead>
                                        <tr>
                                            <th>Order</th>
                                            <th>Title</th>
                                            <th>Type</th>
                                            <th>Value</th>
                                            <th>Actions</th>
                                        </tr>
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
                                        ${!this._loading && !this._error && this._sections.length === 0
                                            ? html`
                                                  <tr
                                                      ><td colspan="5" style="text-align: center"
                                                          >No sections configured.</td
                                                      ></tr
                                                  >
                                              `
                                            : ""}
                                        ${this._sections.map(
                                            (s, idx) => html`
                                                <tr>
                                                    <td>
                                                        <div class="order-controls">
                                                            <button
                                                                class="popout-btn btn-fit btn-info"
                                                                @click=${() => this._moveSection(idx, -1)}
                                                                ?disabled=${idx === 0}
                                                                >↑</button
                                                            >
                                                            <button
                                                                class="popout-btn btn-fit btn-info"
                                                                @click=${() => this._moveSection(idx, 1)}
                                                                ?disabled=${idx === this._sections.length - 1}
                                                                >↓</button
                                                            >
                                                        </div>
                                                    </td>
                                                    <td><strong>${s.title}</strong></td>
                                                    <td>
                                                        ${s.section_type === "smart"
                                                            ? "Smart Group"
                                                            : s.section_type === "category"
                                                              ? "Category Filter"
                                                              : "Custom List"}
                                                    </td>
                                                    <td style="color: var(--color-text-muted); font-size: 0.85rem">
                                                        ${s.section_type === "smart"
                                                            ? s.smart_filter
                                                            : s.section_type === "category"
                                                              ? s.filter_value
                                                              : `${s.roms ? s.roms.length : 0} item(s)`}
                                                    </td>
                                                    <td class="action-btns">
                                                        <button
                                                            class="popout-btn btn-fit btn-info"
                                                            @click=${() => this._startEdit(s)}
                                                            ?disabled=${this._editingSectionId === s.id}
                                                        >
                                                            <span class="btn-edge"></span
                                                            ><span class="btn-front">Edit</span>
                                                        </button>
                                                        <button
                                                            class="popout-btn btn-fit btn-error"
                                                            @click=${() => this._deleteSection(s.id)}
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
                        </div>
                    </div>
                </div>
            </div>
        `;
    }
}

customElements.define("emunex-search-sections-page", EmunexSearchSectionsPage);
