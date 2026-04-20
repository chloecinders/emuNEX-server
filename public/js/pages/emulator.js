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
    tagStyles,
    uploadZoneStyles,
} from "../components/shared-styles.js";
import "../theme-manager.js";

class EmunexEmulatorPage extends LitElement {
    static properties = {
        _status: { type: String, state: true },
        _statusType: { type: String, state: true },
        _loading: { type: Boolean, state: true },
        _consoles: { type: Array, state: true },
        _selectedConsoles: { type: Array, state: true },
        _saveExtensions: { type: Array, state: true },
        _fileName: { type: String, state: true },
        _dragover: { type: Boolean, state: true },
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
        tagStyles,
        css`
            .grid-2 {
                display: grid;
                grid-template-columns: 1fr 1fr;
                gap: var(--spacing-md);
            }
            .checkbox-row {
                display: flex;
                align-items: center;
                gap: var(--spacing-sm);
                margin: var(--spacing-sm) 0;
            }
            .checkbox-row label {
                margin-bottom: 0;
            }
        `,
    ];

    constructor() {
        super();
        this._status = "";
        this._statusType = "";
        this._loading = false;
        this._consoles = [];
        this._selectedConsoles = [];
        this._saveExtensions = [];
        this._fileName = "";
        this._dragover = false;
    }

    connectedCallback() {
        super.connectedCallback();
        this.fetchConsoles();
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

    render() {
        return html`
            <emunex-navbar></emunex-navbar>
            <div class="auth-container">
                <div class="auth-card">
                    <header class="card-header">
                        <a href="/emulators" class="back-link">Back</a>
                        <h1>emuNEX</h1>
                    </header>

                    <div class="content">
                        <form id="emulatorForm" @submit=${this._handleSubmit}>
                            <div class="section-hint">Emulator Details</div>

                            <div class="grid-2">
                                <div class="form-group">
                                    <label for="name">Name</label>
                                    <input type="text" id="name" name="name" placeholder="VBA-M" required />
                                </div>
                                <div class="form-group">
                                    <label for="version">Version</label>
                                    <input type="text" id="version" name="version" placeholder="1.0.0" />
                                </div>
                                <div class="form-group">
                                    <label>Consoles (select at least one)</label>
                                    <div
                                        class="checkbox-grid"
                                        style="display: grid; grid-template-columns: repeat(2, 1fr); gap: 4px; margin-top: 4px;"
                                    >
                                        ${this._consoles.map(
                                            (c) => html`
                                                <label
                                                    class="checkbox-row"
                                                    style="margin: 0; align-items: center; gap: 4px;"
                                                >
                                                    <input
                                                        type="checkbox"
                                                        .checked=${this._selectedConsoles.includes(c.name)}
                                                        @change=${(e) => this._toggleConsole(c.name, e.target.checked)}
                                                    />
                                                    ${c.name.toUpperCase()}
                                                </label>
                                            `,
                                        )}
                                    </div>
                                </div>
                            </div>

                            <div class="form-group">
                                <label for="platform">Platform</label>
                                <select id="platform" name="platform" required>
                                    <option value="linux">Linux</option>
                                    <option value="windows">Windows</option>
                                    <option value="darwin">macOS</option>
                                </select>
                            </div>

                            <div class="form-group">
                                <label for="run_command">Run Command</label>
                                <input type="text" id="run_command" name="run_command" placeholder="--game $rom" />
                            </div>

                            <div class="form-group">
                                <label for="binary_name">Binary Name (e.g. snes9x-x64.exe)</label>
                                <input type="text" id="binary_name" name="binary_name" placeholder="snes9x.exe" />
                            </div>

                            <div class="form-group">
                                <label for="save_path">Save Path (optional)</label>
                                <input type="text" id="save_path" name="save_path" placeholder="/saves/$rom_name.sav" />
                            </div>

                            <div class="form-group">
                                <label for="input_config_file">Input Config File (e.g. retroarch.cfg)</label>
                                <input
                                    type="text"
                                    id="input_config_file"
                                    name="input_config_file"
                                    placeholder="retroarch.cfg"
                                />
                            </div>

                            <div class="form-group">
                                <label for="input_mapper">Input Mapper Identifier</label>
                                <input type="text" id="input_mapper" name="input_mapper" placeholder="retroarch" />
                            </div>

                            <div class="form-group">
                                <label
                                    >Save File Extensions (press Enter to add, e.g. <code>.sra</code>,
                                    <code>.srm</code>)</label
                                >
                                <div class="tag-system">
                                    ${this._saveExtensions.map(
                                        (t) =>
                                            html`<div class="tag"
                                                >${t}
                                                <span class="tag-remove" @click=${() => this._removeSaveExt(t)}
                                                    >×</span
                                                ></div
                                            >`,
                                    )}
                                    <div class="tag-input-wrapper">
                                        <input type="text" placeholder=".sra" @keydown=${this._handleSaveExtKeydown} />
                                    </div>
                                </div>
                            </div>

                            <div class="checkbox-row">
                                <input type="checkbox" id="zipped" name="zipped" />
                                <label for="zipped">Is Zipped / Bundle</label>
                            </div>

                            <div class="section-hint">Files</div>

                            <div class="form-group">
                                <label>Executable / Archive</label>
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
                                        <div class="file-name">${this._fileName}</div>
                                    </div>
                                    <input
                                        type="file"
                                        id="binary_file"
                                        name="binary_file"
                                        required
                                        style="display: none"
                                        @change=${this._fileChange}
                                    />
                                </label>
                            </div>

                            <button type="submit" class="popout-btn" ?disabled=${this._loading}>
                                <span class="btn-edge"></span>
                                <span class="btn-front">${this._loading ? "Processing…" : "Upload Emulator"}</span>
                            </button>
                        </form>

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

    _handleSaveExtKeydown(e) {
        if (e.key === "Enter" && e.target.value.trim()) {
            e.preventDefault();
            const val = e.target.value.trim();
            if (!this._saveExtensions.includes(val)) this._saveExtensions = [...this._saveExtensions, val];
            e.target.value = "";
        }
    }

    _removeSaveExt(t) {
        this._saveExtensions = this._saveExtensions.filter((ext) => ext !== t);
    }

    _toggleConsole(name, checked) {
        if (checked) {
            if (!this._selectedConsoles.includes(name)) this._selectedConsoles = [...this._selectedConsoles, name];
        } else {
            this._selectedConsoles = this._selectedConsoles.filter((c) => c !== name);
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
        const input = this.renderRoot.querySelector("#binary_file");
        input.files = e.dataTransfer.files;
        if (input.files.length) this._fileName = input.files[0].name;
    }

    _fileChange(e) {
        if (e.target.files.length) this._fileName = e.target.files[0].name;
    }

    async _handleSubmit(e) {
        e.preventDefault();
        this._loading = true;
        this._status = "Processing registration…";
        this._statusType = "success";

        const form = this.renderRoot.querySelector("#emulatorForm");
        const formData = new FormData(form);

        formData.delete("save_extensions");
        this._saveExtensions.forEach((ext) => formData.append("save_extensions", ext));

        formData.delete("consoles");
        this._selectedConsoles.forEach((c) => formData.append("consoles", c));

        const isZipped = form.querySelector("#zipped").checked;
        formData.set("zipped", isZipped);

        try {
            const token = (await cookieStore.get("token"))?.value;
            const response = await fetch("/api/v1/emulators/upload", {
                method: "POST",
                body: formData,
                headers: { Authorization: token },
            });

            const result = await response.json();

            if (response.ok) {
                this._status = `Success! Emulator ID: ${result.data}`;
                this._statusType = "success";
                form.reset();
                this._saveExtensions = [];
                this._selectedConsoles = [];
                this._fileName = "";
            } else {
                this._status = `Error: ${result.error || "Registration failed"}`;
                this._statusType = "error";
            }
        } catch (err) {
            this._status = "A network error occurred.";
            this._statusType = "error";
        } finally {
            this._loading = false;
        }
    }
}

customElements.define("emunex-emulator-page", EmunexEmulatorPage);
