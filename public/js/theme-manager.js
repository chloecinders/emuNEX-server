
const THEME_KEY = "emunex-theme";

export class ThemeManager {
    static #listeners = new Set();

    static get current() {
        return localStorage.getItem(THEME_KEY) || "light";
    }

    static async set(theme, sync = true) {
        const resolved = theme === "dark" ? "dark" : "light";

        localStorage.setItem(THEME_KEY, resolved);
        ThemeManager.#apply(resolved);

        window.dispatchEvent(new CustomEvent("theme-changed", { detail: { theme: resolved } }));

        if (sync) {
            const token = (await cookieStore.get("token"))?.value;
            if (token) {
                try {
                    await fetch("/api/v1/users/@me/preferences", {
                        method: "PUT",
                        headers: {
                            Authorization: token,
                            "Content-Type": "application/json",
                        },
                        body: JSON.stringify({ theme: resolved }),
                    });
                } catch (_) { }
            }
        }
    }

    static toggle() {
        return ThemeManager.set(ThemeManager.current === "dark" ? "light" : "dark");
    }

    static async init() {
        ThemeManager.#apply(ThemeManager.current);

        const token = (await cookieStore.get("token"))?.value;

        if (token) {
            try {
                const res = await fetch("/api/v1/users/@me/preferences", {
                    headers: { Authorization: token },
                });
                if (res.ok) {
                    const json = await res.json();
                    const serverTheme = json?.data?.theme;

                    if (serverTheme && serverTheme !== ThemeManager.current) {
                        await ThemeManager.set(serverTheme, false);
                    }
                }
            } catch (_) { }
        }
    }

    static #apply(theme) {
        document.documentElement.setAttribute("data-theme", theme);
    }
}

ThemeManager.init();
