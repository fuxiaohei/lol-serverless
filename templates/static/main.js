// htmx add post json via form
(() => {
    htmx.defineExtension('submitjson', {
        onEvent: function (name, evt) {
            if (name === "htmx:configRequest") {
                evt.detail.headers['Content-Type'] = "application/json"
            }
        },
        encodeParameters: function (xhr, parameters, elt) {
            xhr.overrideMimeType('text/json');
            let values = {};
            parameters.entries().forEach(function (entry) {
                if (!values[entry[0]]) values[entry[0]] = [];
                values[entry[0]].push(entry[1]);
            })
            return (JSON.stringify(values))
        }
    });
})();

document.addEventListener("DOMContentLoaded", function () {

    // bootstrap tooltip init
    (() => {
        const tooltipTriggerList = document.querySelectorAll('[data-bs-toggle="tooltip"]')
        const _tooltipList = [...tooltipTriggerList].map(tooltipTriggerEl => new bootstrap.Tooltip(tooltipTriggerEl));
    })();

    // htmx response error handle
    (() => {
        document.body.addEventListener('htmx:responseError', function (evt) {
            let xhr = evt.detail.xhr;
            let status = xhr.status;
            let message = evt.detail.error;
            if (status > 400) {
                message = xhr.responseText || message;
            }
            let message_html = `<div class="htmx-err-message">${message}<div>`;
            evt.detail.target.innerHTML = message_html;
            evt.detail.target.classList.add("htmx-err-show");
            setTimeout(() => {
                evt.detail.target.classList.remove("htmx-err-show");
            }, 2000);
        });
    })();

    // timeago change to human readable time
    (() => {
        let fn = function () {
            document.querySelectorAll(".time-ago").forEach((el) => {
                let timestamp = parseInt(el.getAttribute("data-x-timeago")) * 1000;
                let dt = new Date(timestamp);
                if (el.getAttribute("data-bs-toggle") == "tooltip") {
                    el.setAttribute("data-bs-title", dt.toLocaleString());
                    new bootstrap.Tooltip(el);
                }
                el.innerText = timeago.format(dt, "en_US");
            });
        };
        fn();
        setInterval(fn, 1000 * 30);
    })();

    // set copy clipboard
    (() => {
        var clipboard = new ClipboardJS('.btn-copy');
        clipboard.on('success', function (e) {
            const tooltip = bootstrap.Tooltip.getInstance(e.trigger);
            tooltip.show();
            setTimeout(() => {
                tooltip.hide();
            }, 1000);
        });
    })();
});


// theme dark mode
(() => {
    const changeTrafficPeriodBtn = theme => {
        // traffic period button changes 
        const traffic_period_btn = document.getElementById("traffic-period-btn");
        if (traffic_period_btn) {
            if (theme == "light") {
                traffic_period_btn.classList.remove("btn-dark");
                traffic_period_btn.classList.add("btn-secondary");
            } else {
                traffic_period_btn.classList.remove("btn-secondary");
                traffic_period_btn.classList.add("btn-dark");
            }
        }
    }

    const changeLinkTheme = theme => {
        // update highlightjs theme css style
        const links = document.querySelectorAll("link[title]");
        if (links.length > 0) {
            links.forEach((link) => {
                link.setAttribute('disabled', "disabled")
            });
            document.querySelector(`link[title="${theme}"]`).removeAttribute('disabled')
        }
    }

    const getStoredTheme = () => localStorage.getItem('runtime-land-theme')
    const setStoredTheme = theme => localStorage.setItem('runtime-land-theme', theme)
    const getPreferredTheme = () => {
        const storedTheme = getStoredTheme();
        return storedTheme ? storedTheme : window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
    }
    const setTheme = theme => {
        document.documentElement.setAttribute('data-bs-theme', theme);
        changeTrafficPeriodBtn(theme);
        changeLinkTheme(theme);
    }
    setTheme(getPreferredTheme());

    // handle media change
    window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', function (e) {
        let newTheme = e.matches ? 'dark' : 'light';
        setTheme(newTheme);
        setStoredTheme(newTheme);
    });

    // handle theme button click
    const themeSwitcher = document.getElementById("theme-switcher");
    if (!themeSwitcher) {
        return;
    }
    themeSwitcher.addEventListener('click', function () {
        let newTheme = getStoredTheme() === 'light' ? 'dark' : 'light';
        // <i class='bi bi-brightness-high'></i> // bi-moon-stars-fill
        setTheme(newTheme);
        setStoredTheme(newTheme);

        // swither icon changes
        const btn = document.getElementById("theme-switcher-icon");
        if (btn) {
            if (newTheme == "light") {
                btn.classList.remove("bi-brightness-high");
                btn.classList.add("bi-moon-stars-fill");
            } else {
                btn.classList.remove("bi-moon-stars-fill");
                btn.classList.add("bi-brightness-high");
            }
        }
    });
})();
