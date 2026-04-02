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

class EmunexUsersPage extends LitElement {
    static properties = {
        _users: { type: Array, state: true },
        _invites: { type: Array, state: true },
        _loading: { type: Boolean, state: true },
        _error: { type: String, state: true },
        _status: { type: String, state: true },
        _statusType: { type: String, state: true },
        _editingUserId: { type: String, state: true },
        _editUsername: { type: String, state: true },
        _editRole: { type: String, state: true },
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
                max-width: 900px;
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
            .action-btns .btn-front {
                padding: 4px 10px;
                font-size: 0.75rem;
                min-width: 60px;
            }
            .content-section {
                margin-bottom: var(--spacing-xxl);
            }
            .invite-table td.code {
                font-family: monospace;
                font-weight: 700;
                color: var(--color-primary);
            }
        `,
    ];

    constructor() {
        super();
        this._users = [];
        this._invites = [];
        this._loading = true;
        this._error = "";
        this._status = "";
        this._statusType = "";
        this._editingUserId = null;
        this._editUsername = "";
        this._editRole = "";
    }

    connectedCallback() {
        super.connectedCallback();
        this.fetchData();
    }

    async fetchData() {
        this._loading = true;
        try {
            const token = (await cookieStore.get("token"))?.value;
            const hdrs = { Authorization: token };
            const [uRes, iRes] = await Promise.all([
                fetch("/api/v1/users", { headers: hdrs }),
                fetch("/api/v1/invites", { headers: hdrs }),
            ]);
            const [uJson, iJson] = await Promise.all([uRes.json(), iRes.json()]);

            if (uRes.ok && iRes.ok) {
                this._users = uJson.data || [];
                this._invites = iJson.data || [];
            } else {
                this._error = uJson.error || iJson.error || "Failed to load data";
            }
        } catch {
            this._error = "Network error loading data";
        } finally {
            this._loading = false;
        }
    }

    showStatus(msg, type) {
        this._status = msg;
        this._statusType = type;
        setTimeout(() => (this._status = ""), 3000);
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
                                      >${this._status}</div
                                  >
                              `
                            : ""}

                        <div class="content-section">
                            <div class="section-hint">Registered Users</div>
                            <div style="overflow-x: auto">
                                <table style="margin-top: 0">
                                    <thead
                                        ><tr><th>ID</th><th>Username</th><th>Role</th><th>Actions</th></tr></thead
                                    >
                                    <tbody>
                                        ${this._loading
                                            ? html`<tr><td colspan="4" style="text-align: center">Loading...</td></tr>`
                                            : ""}
                                        ${this._error
                                            ? html`<tr
                                                  ><td colspan="4" style="text-align: center; color: var(--color-error)"
                                                      >${this._error}</td
                                                  ></tr
                                              >`
                                            : ""}
                                        ${!this._loading && !this._error && this._users.length === 0
                                            ? html`<tr
                                                  ><td colspan="4" style="text-align: center">No users found.</td></tr
                                              >`
                                            : ""}
                                        ${this._users.map((u) => {
                                            const isEditing = this._editingUserId === u.id;
                                            return html`
                                                <tr>
                                                    <td style="font-family: monospace; font-size: 0.8rem">${u.id}</td>
                                                    <td>
                                                        ${isEditing
                                                            ? html`<input
                                                                  type="text"
                                                                  .value=${this._editUsername}
                                                                  @input=${(e) => (this._editUsername = e.target.value)}
                                                                  style="padding: 4px; font-size: 0.8rem;"
                                                              />`
                                                            : html`<strong style="font-weight: 700"
                                                                  >${u.username}</strong
                                                              >`}
                                                    </td>
                                                    <td>
                                                        ${isEditing
                                                            ? html` <select
                                                                  .value=${this._editRole.toLowerCase()}
                                                                  @change=${(e) => (this._editRole = e.target.value)}
                                                                  style="padding: 4px; font-size: 0.8rem;"
                                                              >
                                                                  <option value="user">User</option>
                                                                  <option value="moderator">Moderator</option>
                                                                  <option value="admin">Admin</option>
                                                              </select>`
                                                            : html`<span>${u.role}</span>`}
                                                    </td>
                                                    <td class="action-btns">
                                                        ${isEditing
                                                            ? html`
                                                                  <button
                                                                      class="popout-btn btn-fit btn-success"
                                                                      style="--btn-color-primary: #38a169; --btn-color-dark: #276749"
                                                                      @click=${() => this._saveUser(u.id)}
                                                                  >
                                                                      <span class="btn-edge"></span
                                                                      ><span class="btn-front">Save</span>
                                                                  </button>
                                                                  <button
                                                                      class="popout-btn btn-fit btn-cancel"
                                                                      @click=${this._cancelEdit}
                                                                  >
                                                                      <span class="btn-edge"></span
                                                                      ><span class="btn-front">Cancel</span>
                                                                  </button>
                                                              `
                                                            : html`
                                                                  <button
                                                                      class="popout-btn btn-fit btn-info"
                                                                      @click=${() => this._startEdit(u)}
                                                                  >
                                                                      <span class="btn-edge"></span
                                                                      ><span class="btn-front">Edit</span>
                                                                  </button>
                                                              `}
                                                    </td>
                                                </tr>
                                            `;
                                        })}
                                    </tbody>
                                </table>
                            </div>
                        </div>

                        <div class="content-section" style="margin-bottom: 0">
                            <div class="header-actions">
                                <div class="section-hint" style="margin: 0">Invite Codes</div>
                                <button class="popout-btn btn-fit" @click=${this._generateInvite}>
                                    <span class="btn-edge"></span
                                    ><span class="btn-front" style="padding: 6px 12px; font-size: 0.8rem;"
                                        >+ Generate Code</span
                                    >
                                </button>
                            </div>

                            <div style="overflow-x: auto">
                                <table class="invite-table" style="margin-top: 0">
                                    <thead
                                        ><tr><th>Code</th><th>Used By</th><th>Created</th><th>Actions</th></tr></thead
                                    >
                                    <tbody>
                                        ${!this._loading && !this._error && this._invites.length === 0
                                            ? html`<tr
                                                  ><td colspan="4" style="text-align: center"
                                                      >No invite codes generated.</td
                                                  ></tr
                                              >`
                                            : ""}
                                        ${this._invites.map(
                                            (i) => html`
                                                <tr>
                                                    <td class="code">${i.code}</td>
                                                    <td style="font-size: 0.85rem"
                                                        >${i.used_by_username ||
                                                        html`<span
                                                            style="color: var(--color-text-muted); font-style: italic"
                                                            >Unused</span
                                                        >`}</td
                                                    >
                                                    <td style="font-size: 0.8rem; color: var(--color-text-muted)"
                                                        >${new Date(i.created_at).toLocaleString()}</td
                                                    >
                                                    <td class="action-btns">
                                                        <button
                                                            class="popout-btn btn-fit btn-error"
                                                            @click=${() => this._deleteInvite(i.code)}
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

    _startEdit(u) {
        this._editingUserId = u.id;
        this._editUsername = u.username;
        this._editRole = u.role;
    }
    _cancelEdit() {
        this._editingUserId = null;
    }

    async _saveUser(id) {
        const data = { username: this._editUsername, role: this._editRole };
        try {
            const token = (await cookieStore.get("token"))?.value;
            const res = await fetch(`/api/v1/users/${id}`, {
                method: "PUT",
                headers: { Authorization: token, "Content-Type": "application/json" },
                body: JSON.stringify(data),
            });

            if (res.ok) {
                this.showStatus("User updated.", "success");
                this._editingUserId = null;
                this.fetchData();
            } else {
                const err = await res.json();
                this.showStatus(`Error: ${err.error}`, "error");
            }
        } catch {
            this.showStatus("Failed to update user", "error");
        }
    }

    async _generateInvite() {
        try {
            const token = (await cookieStore.get("token"))?.value;
            const res = await fetch("/api/v1/invites", {
                method: "POST",
                headers: { Authorization: token },
            });
            if (res.ok) {
                this.showStatus("Invite code generated.", "success");
                this.fetchData();
            } else {
                const err = await res.json();
                this.showStatus(`Error: ${err.error}`, "error");
            }
        } catch {
            this.showStatus("Failed to generate code", "error");
        }
    }

    async _deleteInvite(code) {
        if (!confirm(`Delete code ${code}?`)) return;
        try {
            const token = (await cookieStore.get("token"))?.value;
            const res = await fetch(`/api/v1/invites/${code}`, {
                method: "DELETE",
                headers: { Authorization: token },
            });
            if (res.ok) {
                this.showStatus("Invite code deleted.", "success");
                this.fetchData();
            } else {
                const err = await res.json();
                this.showStatus(`Error: ${err.error}`, "error");
            }
        } catch {
            this.showStatus("Deletion failed.", "error");
        }
    }
}

customElements.define("emunex-users-page", EmunexUsersPage);
