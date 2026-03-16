document.addEventListener('DOMContentLoaded', () => {
    const errorBubble = document.querySelector(".error-bubble");
    const form = document.querySelector("#form");

    if (!form) return;

    form.addEventListener("submit", async (e) => {
        e.preventDefault();
        hideError();

        const submitBtn = form.querySelector('button[type="submit"]');
        const btnFront = submitBtn.querySelector('.btn-front');
        const originalBtnText = btnFront.innerText;
        
        const username = document.querySelector("#username").value;
        const password = document.querySelector("#password").value;
        let invite_code = "";

        if (authType === "register") {
            const confirm = document.querySelector("#confirm").value;
            invite_code = document.querySelector("#invite_code").value;
            if (password !== confirm) {
                showError("Passwords do not match");
                return;
            }
        }

        try {
            submitBtn.disabled = true;
            btnFront.innerText = authType === "login" ? "CONNECTING..." : "CREATING...";

            const res = await fetch(`/api/v1/${authType}`, {
                method: "POST",
                headers: { "Content-Type": "application/json" },
                body: JSON.stringify({ username, password, invite_code }),
            });

            const json = await res.json();

            if (!res.ok || !json.success) {
                throw new Error(json.message || json.error || `Server returned ${res.status}`);
            }

            console.log("Authenticated!", json.data.token);
            localStorage.setItem("token", json.data.token);

            if (window.cookieStore) {
                await cookieStore.set({
                    name: "token",
                    value: json.data.token,
                    expires: Date.now() + 86400000,
                    path: '/',
                });
            } else {
                document.cookie = `token=${json.data.token}; path=/; max-age=86400; SameSite=Lax`;
            }

            setTimeout(() => {
                window.location.href = `emunex://login?token=${json.data.token}&domain=${encodeURIComponent(domain)}&storage_path=/storage`;
            }, 100);

        } catch (err) {
            showError(`${err.message}`);
            submitBtn.disabled = false;
            submitBtn.innerText = originalBtnText;
        }
    });

    function showError(msg) {
        errorBubble.innerText = msg;
        errorBubble.style.display = "block";
    }

    function hideError() {
        errorBubble.style.display = "none";
    }
});
