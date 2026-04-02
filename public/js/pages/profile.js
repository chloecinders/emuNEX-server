import { css, html, LitElement } from "lit";
import {
    Camera,
    Check,
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
import { authTokens, baseTokens, buttonStyles, cardStyles, pageHostStyles } from "../components/shared-styles.js";
import "../theme-manager.js";

class EmunexProfilePage extends LitElement {
    static properties = {
        username: { type: String },
        domain: { type: String },
        _userData: { state: true },
        _library: { state: true },
        _topGames: { state: true },
        _loading: { state: true },
        _profileColor: { state: true },
    };

    static styles = [
        baseTokens,
        authTokens,
        pageHostStyles,
        cardStyles,
        buttonStyles,
        css`
            :host {
                display: flex;
                flex-direction: column;
                align-items: stretch;
                justify-content: flex-start;
                --profile-color: #6b5cb1;
                padding-top: 56px;
            }

            .profile-hero {
                width: 100%;
                background-color: var(--profile-color);
                padding: var(--spacing-xxl) 0;
                position: relative;
                overflow: hidden;
                min-height: 180px;
                display: flex;
                align-items: center;
            }

            .hero-container {
                max-width: 1100px;
                width: 100%;
                margin: 0 auto;
                padding: 0 var(--spacing-xl);
                display: flex;
                align-items: center;
                gap: var(--spacing-xl);
                position: relative;
                z-index: 1;
            }

            .avatar-wrap {
                position: relative;
                flex-shrink: 0;
            }

            .avatar {
                width: 128px;
                height: 128px;
                border-radius: var(--radius-full);
                background: rgba(0, 0, 0, 0.2);
                overflow: hidden;
                display: flex;
                align-items: center;
                justify-content: center;
                font-size: 3.5rem;
                font-weight: 900;
                color: white;
                letter-spacing: -2px;
                position: relative;
                backdrop-filter: blur(4px);
            }

            .avatar img {
                width: 100%;
                height: 100%;
                object-fit: cover;
                display: block;
            }

            .profile-info {
                flex: 1;
                min-width: 0;
                display: flex;
                flex-direction: column;
                justify-content: center;
                align-items: flex-start;
            }

            .profile-name-row {
                display: flex;
                align-items: center;
                gap: var(--spacing-md);
            }

            .profile-username {
                font-size: 3.2rem;
                font-weight: 900;
                color: white;
                margin: 0;
                letter-spacing: -2px;
                word-break: break-word;
            }

            .profile-body {
                max-width: 1100px;
                width: 100%;
                margin: 0 auto;
                padding: var(--spacing-xl);
            }

            @media (max-width: 768px) {
                .hero-container {
                    flex-direction: column;
                    text-align: center;
                    gap: var(--spacing-lg);
                }
                .profile-info {
                    align-items: center;
                }
                .profile-name-row {
                    justify-content: center;
                }
                .avatar {
                    width: 100px;
                    height: 100px;
                    font-size: 2.5rem;
                }
                .profile-username {
                    font-size: 2.4rem;
                }
            }

            .covers-grid {
                display: grid;
                grid-template-columns: repeat(5, 1fr);
                gap: var(--spacing-lg);
                margin-top: var(--spacing-xl);
            }

            @media (max-width: 1000px) {
                .covers-grid {
                    grid-template-columns: repeat(3, 1fr);
                }
            }
            @media (max-width: 600px) {
                .covers-grid {
                    grid-template-columns: repeat(2, 1fr);
                }
            }

            .cover-card {
                aspect-ratio: 2 / 3;
                background: var(--color-surface-variant);
                border-radius: var(--radius-md);
                overflow: hidden;
                position: relative;
                border: 1px solid var(--color-border);
                cursor: pointer;
                text-decoration: none;
            }

            .cover-card img {
                width: 100%;
                height: 100%;
                object-fit: cover;
            }

            .cover-overlay {
                position: absolute;
                inset: 0;
                display: flex;
                flex-direction: column;
                justify-content: flex-end;
                padding: var(--spacing-md);
            }

            .cover-title {
                color: white;
                font-weight: 800;
                font-size: 0.9rem;
                margin: 0;
                line-height: 1.2;
                display: -webkit-box;
                -webkit-line-clamp: 2;
                -webkit-box-orient: vertical;
                overflow: hidden;
            }

            .cover-play-badge {
                display: inline-flex;
                align-items: center;
                gap: 4px;
                background: var(--profile-color);
                color: white;
                font-size: 0.65rem;
                font-weight: 900;
                padding: 4px 8px;
                border-radius: var(--radius-sm);
                text-transform: uppercase;
                margin-top: var(--spacing-xs);
                align-self: flex-start;
            }

            .section-title {
                display: flex;
                align-items: center;
                gap: var(--spacing-sm);
                margin-bottom: var(--spacing-md);
            }

            .section-title .lucide {
                width: 20px;
                height: 20px;
                color: var(--profile-color);
            }

            .section-title h2 {
                margin: 0;
                font-size: 0.9rem;
                font-weight: 900;
                text-transform: uppercase;
                letter-spacing: 2px;
                color: var(--color-text-muted);
            }
        `,
    ];

    constructor() {
        super();
        this.username = "";
        this.domain = "";
        this._userData = null;
        this._library = [];
        this._topGames = [];
        this._loading = true;
        this._profileColor = "#6b5cb1";
    }

    async connectedCallback() {
        super.connectedCallback();
        await this._fetchInitialData();
    }

    async _fetchInitialData() {
        this._loading = true;
        await Promise.all([this._fetchUserData(), this._fetchLibrary()]);
        this._loading = false;
        if (this._userData?.profile_color) {
            this._profileColor = this._userData.profile_color;
            this.style.setProperty("--profile-color", this._profileColor);
        }
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
                this.style.setProperty("--profile-color", this._profileColor);
            }
        } catch (e) {
            console.error("Fetch profile failed", e);
        }
    }

    async _fetchLibrary() {
        try {
            const token = await this._getToken();
            const res = await fetch("/api/v1/library", { headers: { Authorization: token } });
            if (!res.ok) return;
            const json = await res.json();
            this._library = json.data || [];
            this._topGames = [...this._library].sort((a, b) => b.play_count - a.play_count).slice(0, 5);
        } catch (e) {
            console.error("Fetch library failed", e);
        }
    }

    _getInitials() {
        return (this.username || "?").slice(0, 1).toUpperCase();
    }

    render() {
        if (this._loading)
            return html`<div style="padding: 100px; text-align: center; color: var(--color-text-muted);"
                >Loading...</div
            >`;

        const avatar = this._userData?.avatar_path ? `/storage${this._userData.avatar_path}` : null;

        return html`
            <emunex-navbar></emunex-navbar>

            <div class="profile-hero">
                <div class="hero-container">
                    <div class="avatar-wrap">
                        <div class="avatar"> ${avatar ? html`<img src="${avatar}" />` : this._getInitials()} </div>
                    </div>

                    <div class="profile-info">
                        <div class="profile-name-row">
                            <h1 class="profile-username">${this.username}</h1>
                        </div>
                    </div>
                </div>
            </div>

            <div class="profile-body">
                <div class="section-title">
                    <h2>Most Played</h2>
                </div>

                ${this._topGames.length === 0
                ? html`
                          <div class="empty-state" style="padding: 100px; text-align: center; opacity: 0.5;">
                              <i data-lucide="gamepad-2" style="width: 48px; height: 48px; margin-bottom: 16px;"></i>
                              <p>No games played yet. Start your journey!</p>
                          </div>
                      `
                : html`
                          <div class="covers-grid">
                              ${this._topGames.map(
                    (game) => html`
                                      <a class="cover-card" href="/saves?rom_id=${game.rom_id}" title="${game.title}">
                                          ${game.image_path
                            ? html`<img src="/storage${game.image_path}" />`
                            : html`<div
                                                    style="flex:1; display:flex; align-items:center; justify-content:center; background:#1a1a1a;"
                                                    ><i data-lucide="gamepad-2" style="opacity:0.2;"></i
                                                ></div>`}
                                          <div class="cover-overlay">
                                              <div class="cover-play-badge">
                                                  <i data-lucide="clock" style="width:10px; height:10px;"></i>
                                                  ${game.play_count} Plays
                                              </div>
                                          </div>
                                      </a>
                                  `,
                )}
                          </div>
                      `}
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
                Clock,
                Pipette,
            },
            nameAttr: "data-lucide",
            root: this.shadowRoot,
        });
    }
}

customElements.define("emunex-profile-page", EmunexProfilePage);
