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
import "../libs/spark-md5.min.js";

class EmunexEmulatorPage extends LitElement {
    static properties = {
        _status: { type: String, state: true },
        _statusType: { type: String, state: true },
        _loading: { type: Boolean, state: true },
        _consoles: { type: Array, state: true },
        _selectedConsoles: { type: Array, state: true },
        _savePaths: { type: Array, state: true },
        _saveExtensions: { type: Array, state: true },
        _fileName: { type: String, state: true },
        _dragover: { type: Boolean, state: true },
        _extraFiles: { type: Array, state: true },
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
        this._savePaths = [];
        this._saveExtensions = [];
        this._fileName = "";
        this._dragover = false;
        this._extraFiles = [];
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
                                        style="display: grid; grid-template-columns: repeat(2, 1fr); gap: 4px; margin-top: 4px;">
                                        ${this._consoles.map(
            (c) => html`
                                                <label
                                                    class="checkbox-row"
                                                    style="margin: 0; align-items: center; gap: 4px;">
                                                    <input
                                                        type="checkbox"
                                                        .checked=${this._selectedConsoles.includes(c.name)}
                                                        @change=${(e) =>
                    this._toggleConsole(c.name, e.target.checked)} />
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
                                <label for="binary_name">Binary Name</label>
                                <input type="text" id="binary_name" name="binary_name" placeholder="snes9x.exe" />
                            </div>

                            <div class="form-group">
                                <label>
                                    Save Paths (press Enter to add, e.g.
                                    <code>/saves/</code>
                                    or
                                    <code>/states/</code>
                                    )
                                </label>
                                <div class="tag-system">
                                    ${this._savePaths.map(
            (t) => html`
                                            <div class="tag">
                                                ${t}
                                                <span class="tag-remove" @click=${() => this._removeSavePath(t)}>
                                                    ×
                                                </span>
                                            </div>
                                        `,
        )}
                                    <div class="tag-input-wrapper">
                                        <input
                                            type="text"
                                            placeholder="/saves/$rom_name.sav"
                                            @keydown=${(e) => this._handleSavePathKeydown(e)} />
                                    </div>
                                </div>
                            </div>

                            <div class="form-group">
                                <label for="input_config_file">Input Config File</label>
                                <input
                                    type="text"
                                    id="input_config_file"
                                    name="input_config_file"
                                    placeholder="retroarch.cfg" />
                            </div>

                            <div class="form-group">
                                <label for="input_mapper">Input Mapper Identifier</label>
                                <input type="text" id="input_mapper" name="input_mapper" placeholder="retroarch" />
                            </div>

                            <div class="form-group">
                                <label>
                                    Save File Extensions (press Enter to add, e.g.
                                    <code>.sra</code>
                                    ,
                                    <code>.srm</code>
                                    )
                                </label>
                                <div class="tag-system">
                                    ${this._saveExtensions.map(
            (t) => html`
                                            <div class="tag">
                                                ${t}
                                                <span class="tag-remove" @click=${() => this._removeSaveExt(t)}>×</span>
                                            </div>
                                        `,
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
                                    @drop=${this._drop}>
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
                                        @change=${this._fileChange} />
                                </label>
                            </div>

                            <div class="section-hint" style="margin-top: var(--spacing-lg)">Additional Files</div>

                            ${this._extraFiles.map(
            (f, i) => html`
                                    <div
                                        style="border: 1px solid var(--color-border); border-radius: 6px; padding: var(--spacing-md); margin-bottom: var(--spacing-md); position: relative; display: flex; flex-direction: column; gap: var(--spacing-md);">
                                        <button
                                            type="button"
                                            style="position: absolute; top: 8px; right: 8px; background: none; border: none; cursor: pointer; color: var(--color-error); font-size: 1rem; line-height: 1;"
                                            @click=${() => this._removeExtraFile(i)}
                                            title="Remove">
                                            x
                                        </button>
                                        <div class="form-group">
                                            <label>File</label>
                                            <label class="upload-zone">
                                                <div class="upload-icon">↑</div>
                                                <div class="upload-info">
                                                    <div class="upload-text">
                                                        ${f.file ? f.file.name : "Click to choose file…"}
                                                    </div>
                                                </div>
                                                <input
                                                    type="file"
                                                    style="display:none"
                                                    @change=${(e) =>
                    this._updateExtraFile(i, "file", e.target.files[0])} />
                                            </label>
                                        </div>
                                        <div class="form-group">
                                            <label>Install Path</label>
                                            <input
                                                type="text"
                                                placeholder="/opt/emu/bios.bin or C:\\Emu\\bios.bin"
                                                .value=${f.path || ""}
                                                @input=${(e) =>
                    this._updateExtraFile(i, "path", e.target.value)} />
                                        </div>
                                    </div>
                                `,
        )}

                            <button
                                type="button"
                                class="popout-btn btn-fit btn-secondary"
                                style="margin-bottom: var(--spacing-lg);"
                                @click=${this._addExtraFile}>
                                <span class="btn-edge"></span>
                                <span class="btn-front">+ Add File</span>
                            </button>

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
                        : "status-success"}">
                                      ${this._status}
                                  </div>
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

    _handleSavePathKeydown(e) {
        if (e.key === "Enter" && e.target.value.trim()) {
            e.preventDefault();
            const val = e.target.value.trim();
            if (!this._savePaths.includes(val)) this._savePaths = [...this._savePaths, val];
            e.target.value = "";
        }
    }

    _removeSavePath(t) {
        this._savePaths = this._savePaths.filter((path) => path !== t);
    }

    _addExtraFile() {
        this._extraFiles = [...this._extraFiles, { file: null, path: "" }];
    }

    _removeExtraFile(index) {
        this._extraFiles = this._extraFiles.filter((_, i) => i !== index);
    }

    _updateExtraFile(index, field, value) {
        this._extraFiles = this._extraFiles.map((f, i) => (i === index ? { ...f, [field]: value } : f));
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

        formData.delete("save_paths");
        this._savePaths.forEach((path) => formData.append("save_paths", path));

        formData.delete("consoles");
        this._selectedConsoles.forEach((c) => formData.append("consoles", c));

        const isZipped = form.querySelector("#zipped").checked;
        formData.set("zipped", isZipped);

        try {
            const token = (await cookieStore.get("token"))?.value;
            
            const binaryFile = formData.get("binary_file");
            if (!binaryFile || !binaryFile.size) throw new Error("Binary file is required");

            this._status = "Computing MD5... (This may take a moment for large binaries)";
            const md5 = await new Promise((resolve, reject) => {
                const chunkSize = 2097152;
                const chunks = Math.ceil(binaryFile.size / chunkSize);
                let currentChunk = 0;
                const spark = new window.SparkMD5.ArrayBuffer();
                const fileReader = new FileReader();
                fileReader.onload = (e) => {
                    spark.append(e.target.result);
                    currentChunk++;
                    if (currentChunk < chunks) loadNext();
                    else resolve(spark.end());
                };
                fileReader.onerror = () => reject("File reading failed");
                const loadNext = () => {
                    const start = currentChunk * chunkSize;
                    const end = start + chunkSize >= binaryFile.size ? binaryFile.size : start + chunkSize;
                    fileReader.readAsArrayBuffer(binaryFile.slice(start, end));
                };
                loadNext();
            });

            this._status = "Requesting secure upload URL...";
            let fileExt = binaryFile.name.split('.').pop().toLowerCase() || "bin";

            const signRes = await fetch("/api/v1/emulators/sign", {
                method: "POST",
                headers: { "Content-Type": "application/json", Authorization: token },
                body: JSON.stringify({
                    platform: formData.get("platform"),
                    consoles: this._selectedConsoles,
                    name: formData.get("name"),
                    file_extension: fileExt
                })
            });
            const signJson = await signRes.json();
            if (!signRes.ok) throw new Error(signJson.error || "Failed to sign upload URL");

            this._status = "Uploading binary directly to S3...";
            const putRes = await fetch(signJson.data.upload_url, {
                method: "PUT",
                body: binaryFile,
                headers: { "Content-Type": "application/octet-stream" }
            });
            if (!putRes.ok) throw new Error("S3 upload failed");

            this._status = "Finalizing emulator registration...";
            formData.delete("binary_file");
            formData.set("md5_hash", md5);
            formData.set("file_size_bytes", binaryFile.size.toString());
            formData.set("file_extension", fileExt);

            const response = await fetch("/api/v1/emulators/upload", {
                method: "POST",
                body: formData,
                headers: { Authorization: token },
            });

            const result = await response.json();

            if (!response.ok) {
                this._status = `Error: ${result.error || "Registration failed"}`;
                this._statusType = "error";
                return;
            }

            const emulatorId = typeof result.data === "object" ? result.data.id : result.data;

            for (const entry of this._extraFiles) {
                if (!entry.file) continue;

                const signRes = await fetch(`/api/v1/emulators/${emulatorId}/extra-file/sign`, {
                    method: "POST",
                    headers: { Authorization: token, "Content-Type": "application/json" },
                    body: JSON.stringify({
                        filename: entry.file.name,
                        path: entry.path,
                    }),
                });

                if (!signRes.ok) {
                    const err = await signRes.json();
                    this._status = `Extra file sign failed: ${err.error || signRes.statusText}`;
                    this._statusType = "error";
                    return;
                }
                const { data: signed } = await signRes.json();

                const putRes = await fetch(signed.upload_url, {
                    method: "PUT",
                    body: entry.file,
                    headers: { "Content-Type": "application/octet-stream" },
                });

                if (!putRes.ok) {
                    this._status = `Direct S3 upload failed for ${entry.file.name}`;
                    this._statusType = "error";
                    return;
                }

                await fetch(`/api/v1/emulators/${emulatorId}/extra-file/confirm`, {
                    method: "POST",
                    headers: { Authorization: token, "Content-Type": "application/json" },
                    body: JSON.stringify({
                        s3_path: signed.s3_path,
                        path: signed.path,
                    }),
                });
            }

            this._status = `Success! Emulator ID: ${emulatorId}`;
            this._statusType = "success";
            form.reset();
            this._savePaths = [];
            this._saveExtensions = [];
            this._selectedConsoles = [];
            this._fileName = "";
            this._extraFiles = [];
        } catch (err) {
            this._status = err.message || "A network error occurred.";
            this._statusType = "error";
        } finally {
            this._loading = false;
        }
    }
}

customElements.define("emunex-emulator-page", EmunexEmulatorPage);
