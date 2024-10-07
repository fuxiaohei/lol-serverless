
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
})