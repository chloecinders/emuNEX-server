import { css, html, LitElement } from "lit";
import {
    Camera,
    Check,
    ChevronLeft,
    ChevronRight,
    Clock,
    createIcons,
    ExternalLink,
    Eye,
    EyeOff,
    Gamepad2,
    Lock,
    LogOut,
    Pipette,
    Save,
    Settings,
    Shield,
    User,
    X,
} from "lucide";
import "../components/navbar.js";
import {
    authTokens,
    baseTokens,
    buttonStyles,
    cardStyles,
    formStyles,
    pageHostStyles,
    statusStyles,
    uploadZoneStyles,
} from "../components/shared-styles.js";
import "../theme-manager.js";

class EmunexSettingsPage extends LitElement {
    static properties = {
        username: { type: String },
        domain: { type: String },
        _userData: { state: true },
        _status: { state: true },
        _statusType: { state: true },
        _editUsername: { state: true },
        _currentPassword: { state: true },
        _newPassword: { state: true },
        _confirmPassword: { state: true },
        _showCurrentPw: { state: true },
        _showNewPw: { state: true },
        _avatarPreview: { state: true },
        _profileColor: { state: true },
        _avatarFile: { state: true },
        _avatarUploading: { state: true },
        _discordSyncing: { state: true },
    };

    static styles = [
        baseTokens,
        authTokens,
        pageHostStyles,
        cardStyles,
        formStyles,
        buttonStyles,
        statusStyles,
        uploadZoneStyles,
        css`
            .auth-container {
                max-width: 600px;
                margin-bottom: 80px;
            }

            .color-grid {
                display: flex;
                flex-wrap: wrap;
                gap: var(--spacing-md);
                padding: 4px;
                margin-bottom: var(--spacing-lg);
            }

            .color-swatch {
                width: 42px;
                height: 42px;
                border-radius: var(--radius-sm);
                cursor: pointer;
                border: 1px solid var(--color-border);
                transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
                position: relative;
            }

            .color-swatch.active {
                box-shadow:
                    0 0 0 2px var(--color-surface),
                    0 0 0 4px var(--color-primary);
                border-color: transparent;
                z-index: 5;
            }

            .custom-swatch {
                display: flex;
                align-items: center;
                justify-content: center;
                background: var(--color-surface-variant);
            }

            .custom-swatch i {
                width: 18px;
                height: 18px;
                color: var(--color-text-muted);
                pointer-events: none;
            }

            .custom-swatch.active i {
                color: white;
                filter: drop-shadow(0 2px 4px rgba(0, 0, 0, 0.4));
            }

            .custom-swatch input[type="color"] {
                position: absolute;
                inset: 0;
                opacity: 0;
                cursor: pointer;
                width: 100%;
                height: 100%;
            }

            .avatar-upload-area {
                display: flex;
                align-items: center;
                gap: var(--spacing-lg);
                padding: var(--spacing-md);
                background: var(--color-surface-variant);
                border-radius: var(--radius-md);
                border: 1px dashed var(--color-border);
                margin-bottom: var(--spacing-md);
            }

            .avatar-preview {
                width: 64px;
                height: 64px;
                border-radius: var(--radius-full);
                background: var(--color-primary);
                color: white;
                display: flex;
                align-items: center;
                justify-content: center;
                font-weight: 900;
                font-size: 1.5rem;
                overflow: hidden;
                flex-shrink: 0;
            }

            .avatar-preview img {
                width: 100%;
                height: 100%;
                object-fit: cover;
            }

            .avatar-upload-info {
                flex: 1;
            }

            .pw-field {
                position: relative;
                display: flex;
                align-items: center;
            }

            .pw-toggle {
                position: absolute;
                right: 12px;
                background: transparent;
                border: none;
                color: var(--color-text-muted);
                cursor: pointer;
                display: flex;
                align-items: center;
                z-index: 5;
            }

            .btn-front-flex {
                display: flex;
                align-items: center;
                justify-content: center;
                gap: 8px;
                line-height: 1;
            }

            .btn-front-flex i {
                width: 16px;
                height: 16px;
                margin-top: -1px;
            }

            .section-hint {
                margin-top: var(--spacing-xl);
                margin-bottom: var(--spacing-md);
            }
            .section-hint:first-child {
                margin-top: 0;
            }
        `,
    ];

    constructor() {
        super();
        this.username = "";
        this.domain = "";
        this._userData = null;
        this._status = "";
        this._statusType = "";
        this._editUsername = "";
        this._profileColor = "#6b5cb1";
        this._currentPassword = "";
        this._newPassword = "";
        this._confirmPassword = "";
        this._showCurrentPw = false;
        this._showNewPw = false;
        this._avatarPreview = null;
        this._avatarFile = null;
        this._avatarUploading = false;
        this._discordSyncing = false;
    }

    async connectedCallback() {
        super.connectedCallback();
        this._editUsername = this.username;
        await this._fetchUserData();
    }

    async _getToken() {
        return (await cookieStore.get("token"))?.value;
    }

    async _fetchUserData() {
        try {
            const token = await this._getToken();
            const res = await fetch("/api/v1/users/@me", { headers: { Authorization: token } });
            if (res.ok) {
                const json = await res.json();
                this._userData = json.data;
                this._profileColor = this._userData.profile_color || "#6b5cb1";
                this._editUsername = this._userData.username || this.username;
                this.style.setProperty("--profile-color", this._profileColor);
            }
        } catch (e) {
            console.error("Fetch user data failed", e);
        }
    }

    showStatus(msg, type) {
        this._status = msg;
        this._statusType = type;
        setTimeout(() => (this._status = ""), 4000);
    }

    async _updateProfileColor(color) {
        this._profileColor = color;
        this.style.setProperty("--profile-color", color);
        try {
            const token = await this._getToken();
            const res = await fetch("/api/v1/users/@me/profile-color", {
                method: "PUT",
                headers: { Authorization: token, "Content-Type": "application/json" },
                body: JSON.stringify({ color }),
            });
            if (res.ok) {
                this.showStatus("Theme color updated!", "success");
            }
        } catch (e) {
            console.error("Update color failed", e);
        }
    }

    async _updateUsername(e) {
        e.preventDefault();
        try {
            const token = await this._getToken();
            const res = await fetch("/api/v1/users/@me/username", {
                method: "PUT",
                headers: { Authorization: token, "Content-Type": "application/json" },
                body: JSON.stringify({ username: this._editUsername }),
            });
            if (res.ok) {
                this.username = this._editUsername;
                this.showStatus("Username updated!", "success");
            } else {
                const err = await res.json();
                this.showStatus(err.error || "Update failed", "error");
            }
        } catch (e) {
            this.showStatus("Network error", "error");
        }
    }

    async _updatePassword(e) {
        e.preventDefault();
        if (this._newPassword !== this._confirmPassword) {
            this.showStatus("New passwords don't match", "error");
            return;
        }
        try {
            const token = await this._getToken();
            const res = await fetch("/api/v1/users/@me/password", {
                method: "PUT",
                headers: { Authorization: token, "Content-Type": "application/json" },
                body: JSON.stringify(body),
            });
            if (res.ok) {
                this._currentPassword = "";
                this._newPassword = "";
                this._confirmPassword = "";
                if (this._userData) {
                    this._userData.has_password = true;
                    this.requestUpdate();
                }
                this.showStatus("Password updated successfully", "success");
            } else {
                const err = await res.json();
                this.showStatus(err.error || "Update failed", "error");
            }
        } catch (e) {
            this.showStatus("Network error", "error");
        }
    }

    _onAvatarFileChange(e) {
        const file = e.target.files?.[0];
        if (!file) return;
        if (file.size > 10 * 1024 * 1024) {
            this.showStatus("Avatar file must be under 10MB", "error");
            e.target.value = "";
            return;
        }
        this._avatarFile = file;
        const reader = new FileReader();
        reader.onload = (ev) => {
            this._avatarPreview = ev.target.result;
        };
        reader.readAsDataURL(file);
    }

    async _uploadAvatar() {
        if (!this._avatarFile) return;
        this._avatarUploading = true;
        try {
            const reader = new FileReader();
            const base64 = await new Promise((res) => {
                reader.onload = (e) => res(e.target.result.split(",")[1]);
                reader.readAsDataURL(this._avatarFile);
            });
            const token = await this._getToken();
            const result = await fetch("/api/v1/users/@me/avatar", {
                method: "PUT",
                headers: { Authorization: token, "Content-Type": "application/json" },
                body: JSON.stringify({ content: base64 }),
            });
            if (result.ok) {
                this.showStatus("Avatar updated!", "success");
                this._avatarFile = null;
                await this._fetchUserData();
            }
        } catch (e) {
            this.showStatus("Upload failed", "error");
        } finally {
            this._avatarUploading = false;
        }
    }

    async _syncDiscordWidget() {
        if (this._discordSyncing) return;
        this._discordSyncing = true;
        try {
            const token = await this._getToken();
            const res = await fetch("/api/v1/users/@me/sync-discord-widget", {
                method: "POST",
                headers: { Authorization: token },
            });
            if (res.ok) {
                this.showStatus("Discord profile synced!", "success");
            } else {
                const err = await res.json().catch(() => ({}));
                this.showStatus(err.error || "Sync failed", "error");
            }
        } catch (e) {
            this.showStatus("Network error", "error");
        } finally {
            this._discordSyncing = false;
        }
    }

    async _logout() {
        if (!confirm("Are you sure you want to log out?")) return;
        try {
            const token = await this._getToken();
            await fetch("/api/v1/logout", { method: "POST", headers: { Authorization: token } });
        } catch (e) {}
        await cookieStore.delete("token");
        window.location.href = "/";
    }

    _getInitials() {
        return (this.username || "?").substring(0, 1).toUpperCase();
    }

    _renderStatusBox() {
        if (!this._status) return "";
        return html`<div class="status-box ${this._statusType === "error" ? "status-error" : "status-success"}"
            >${this._status}</div
        >`;
    }

    render() {
        const avatar =
            this._avatarPreview || (this._userData?.avatar_path ? `/storage${this._userData.avatar_path}` : null);
        const presetColors = ["#6b5cb1", "#e66b1d", "#2da44e", "#cf222e", "#0969da", "#bf3989", "#d4a72c", "#2ea043"];
        const isCustom = !presetColors.includes(this._profileColor);

        return html`
            <emunex-navbar></emunex-navbar>

            <div class="auth-container">
                <div class="auth-card">
                    <header class="card-header">
                        <h1>Settings</h1>
                    </header>

                    <div class="content">
                        ${this._renderStatusBox()}

                        <div class="section-hint">Appearance</div>
                        <div
                            style="font-size: 0.75rem; font-weight: 800; color: var(--color-text-muted); text-transform: uppercase; margin-bottom: var(--spacing-sm);"
                            >Theme Color</div
                        >
                        <div class="color-grid">
                            ${presetColors.map(
                                (c) => html`
                                    <div
                                        class="color-swatch ${this._profileColor === c ? "active" : ""}"
                                        style="background: ${c}"
                                        @click=${() => this._updateProfileColor(c)}
                                    ></div>
                                `,
                            )}
                            <div
                                class="color-swatch custom-swatch ${isCustom ? "active" : ""}"
                                style="background: ${isCustom ? this._profileColor : "var(--color-surface-variant)"}"
                            >
                                <i data-lucide="pipette"></i>
                                <input
                                    type="color"
                                    .value=${this._profileColor || "#ffffff"}
                                    @input=${(e) => this._updateProfileColor(e.target.value)}
                                />
                            </div>
                        </div>

                        <div
                            style="font-size: 0.75rem; font-weight: 800; color: var(--color-text-muted); text-transform: uppercase; margin-bottom: var(--spacing-sm);"
                            >Avatar</div
                        >
                        <div class="avatar-upload-area">
                            <div class="avatar-preview">
                                ${avatar ? html`<img src="${avatar}" />` : this._getInitials()}
                            </div>
                            <div class="avatar-upload-info">
                                <div style="font-size: 0.85rem; font-weight: 800; color: var(--color-text);"
                                    >New Avatar</div
                                >
                            </div>
                            <label
                                for="avatar-input"
                                class="popout-btn btn-secondary btn-fit"
                                style="cursor:pointer; margin-top:0;"
                            >
                                <span class="btn-edge"></span>
                                <span class="btn-front btn-front-flex" style="padding: 8px 16px;">Choose</span>
                            </label>
                            <input
                                id="avatar-input"
                                type="file"
                                accept="image/*"
                                style="display:none"
                                @change=${this._onAvatarFileChange}
                            />
                        </div>
                        ${this._avatarFile
                            ? html`
                                  <button
                                      class="popout-btn"
                                      style="width:100%; margin-bottom: var(--spacing-lg);"
                                      @click=${this._uploadAvatar}
                                  >
                                      <span class="btn-edge"></span>
                                      <span class="btn-front btn-front-flex" style="padding: 10px;">
                                          ${this._avatarUploading ? "Uploading..." : "Save Avatar"}
                                      </span>
                                  </button>
                              `
                            : ""}

                        <div class="section-hint">Identity</div>
                        <form @submit=${this._updateUsername}>
                            <div class="form-group">
                                <label>Username</label>
                                <input
                                    type="text"
                                    .value=${this._editUsername}
                                    @input=${(e) => (this._editUsername = e.target.value)}
                                    required
                                />
                            </div>
                            <button type="submit" class="popout-btn" style="width: 100%;">
                                <span class="btn-edge"></span>
                                <span class="btn-front" style="padding: 10px;">Update Username</span>
                            </button>
                        </form>

                        <div class="section-hint">Security</div>
                        <form
                            @submit=${this._updatePassword}
                            style="display:flex; flex-direction:column; gap:var(--spacing-md);"
                        >
                            ${this._userData?.has_password !== false
                                ? html`
                                      <div class="form-group">
                                          <label>Current Password</label>
                                          <div class="pw-field">
                                              <input
                                                  type=${this._showCurrentPw ? "text" : "password"}
                                                  .value=${this._currentPassword}
                                                  @input=${(e) => (this._currentPassword = e.target.value)}
                                                  required
                                              />
                                              <button
                                                  type="button"
                                                  class="pw-toggle"
                                                  @click=${() => (this._showCurrentPw = !this._showCurrentPw)}
                                                  ><i data-lucide="${this._showCurrentPw ? "eye-off" : "eye"}"></i
                                              ></button>
                                          </div>
                                      </div>
                                  `
                                : ""}
                            <div class="form-group">
                                <label>New Password</label>
                                <div class="pw-field">
                                    <input
                                        type=${this._showNewPw ? "text" : "password"}
                                        .value=${this._newPassword}
                                        @input=${(e) => (this._newPassword = e.target.value)}
                                        required
                                        minlength="6"
                                    />
                                    <button
                                        type="button"
                                        class="pw-toggle"
                                        @click=${() => (this._showNewPw = !this._showNewPw)}
                                        ><i data-lucide="${this._showNewPw ? "eye-off" : "eye"}"></i
                                    ></button>
                                </div>
                            </div>
                            <div class="form-group">
                                <label>Confirm New Password</label>
                                <input
                                    type="password"
                                    .value=${this._confirmPassword}
                                    @input=${(e) => (this._confirmPassword = e.target.value)}
                                    required
                                    minlength="6"
                                />
                            </div>
                            <button type="submit" class="popout-btn" style="width: 100%;">
                                <span class="btn-edge"></span>
                                <span class="btn-front" style="padding: 10px;"
                                    >${this._userData?.has_password === false
                                        ? "Set Password"
                                        : "Change Password"}</span
                                >
                            </button>
                        </form>

                        <div class="section-hint">Connections</div>
                        <div
                            style="font-size: 0.85rem; font-weight: 700; color: var(--color-text-muted); line-height:1.5; margin-bottom: var(--spacing-md);"
                        >
                            ${this._userData?.discord_id
                                ? "Your account is linked to Discord."
                                : "Link your Discord account to log in without a password."}
                        </div>
                        ${this._userData?.discord_id
                            ? html`
                                  <div
                                      style="display: flex; align-items: center; justify-content: space-between; background: var(--color-surface-variant); padding: var(--spacing-md); border-radius: var(--radius-md); border: 1px solid var(--color-border); margin-bottom: var(--spacing-md);"
                                  >
                                      <div style="display: flex; align-items: center; gap: var(--spacing-sm);">
                                          <i data-lucide="gamepad-2" style="color: #5865F2;"></i>
                                          <span style="font-weight: 800;">${this._userData.discord_id}</span>
                                      </div>
                                      <div
                                          style="font-size: 0.75rem; color: var(--color-success, #2da44e); font-weight: 800; text-transform: uppercase;"
                                          >Linked</div
                                      >
                                  </div>
                                  <button
                                      class="popout-btn"
                                      style="--btn-color-primary: #5865F2; --btn-color-dark: #4752C4; width: 100%; margin-top: 0; margin-bottom: var(--spacing-md);"
                                      ?disabled=${this._discordSyncing}
                                      @click=${this._syncDiscordWidget}
                                  >
                                      <span class="btn-edge"></span>
                                      <span class="btn-front btn-front-flex" style="padding: 10px; font-weight: 900;">
                                          <i data-lucide="gamepad-2"></i>
                                          ${this._discordSyncing ? "Syncing..." : "SYNC DISCORD PROFILE"}
                                      </span>
                                  </button>
                              `
                            : html`
                                  <button
                                      class="popout-btn"
                                      style="--btn-color-primary: #5865F2; --btn-color-dark: #4752C4; width: 100%; margin-top: 0; margin-bottom: var(--spacing-md);"
                                      @click=${() => {
                                          window.location.href = "/auth/discord/authorize?action=link";
                                      }}
                                  >
                                      <span class="btn-edge"></span>
                                      <span class="btn-front btn-front-flex" style="padding: 10px; font-weight: 900;">
                                          <i data-lucide="gamepad-2"></i>
                                          LINK DISCORD ACCOUNT
                                      </span>
                                  </button>
                              `}

                        <div class="section-hint">Connectivity</div>
                        <div
                            style="font-size: 0.85rem; font-weight: 700; color: var(--color-text-muted); line-height:1.5; margin-bottom: var(--spacing-md);"
                        >
                            Connect your emuNEX client to the server instantly.
                        </div>
                        <button
                            class="popout-btn"
                            style="width: 100%; margin-top: 0;"
                            @click=${() => {
                                this._getToken().then((t) => {
                                    window.location.href = `emunex://login?token=\${t}&domain=\${encodeURIComponent(this.domain)}&storage_path=/storage`;
                                });
                            }}
                        >
                            <span class="btn-edge"></span>
                            <span class="btn-front" style="padding: 10px; font-weight: 900;">LAUNCH CLIENT</span>
                        </button>

                        <div class="section-hint" style="color: var(--color-error);">Sessions</div>
                        <button class="popout-btn btn-error" style="width: 100%;" @click=${this._logout}>
                            <span class="btn-edge"></span>
                            <span class="btn-front" style="padding: 10px; font-weight: 900;">LOG OUT</span>
                        </button>
                    </div>
                </div>
            </div>
        `;
    }

    updated() {
        this._updateIcons();
    }
    _updateIcons() {
        createIcons({
            icons: {
                Settings,
                User,
                Shield,
                Save,
                ExternalLink,
                Camera,
                Check,
                X,
                Eye,
                EyeOff,
                LogOut,
                Lock,
                Gamepad2,
                ChevronRight,
                ChevronLeft,
                Clock,
                Pipette,
            },
            nameAttr: "data-lucide",
            root: this.shadowRoot,
        });
    }
}

customElements.define("emunex-settings-page", EmunexSettingsPage);
