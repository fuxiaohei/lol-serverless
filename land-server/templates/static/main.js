/// friendly_bytesize convert bytes to human readable size
function friendly_bytesize(v, with_byte_unit) {
    if (v < 0.1) {
        return 0
    }
    let bytes_units = ['iB', 'KiB', 'MiB', 'GiB', 'TiB', 'PiB', 'EiB', 'ZiB', 'YiB'];
    let units = ['', 'K', 'M', 'G', 'T', 'P', 'E', 'Z', 'Y'];
    let i = 0;
    while (v > 1000) {
        v /= 1000;
        i++;
    }
    v = v.toFixed(2);
    let u = with_byte_unit ? bytes_units[i] : units[i];
    return `${v}${u}`
}

/// convert unixtimestamp to hour and minute, HH:MM
function unix2hour(v) {
    const dateObj = new Date(v)
    const hours = dateObj.getHours() >= 10 ? dateObj.getHours() : '0' + dateObj.getHours()
    const minutes = dateObj.getMinutes() < 10 ? dateObj.getMinutes() + '0' : dateObj.getMinutes()
    const UnixTimeToDate = hours + ':' + minutes
    if (window.traffic_period == "7d") {
        const month = dateObj.getMonth() + 1
        const days = dateObj.getDate() >= 10 ? dateObj.getDate() : '0' + dateObj.getDate()
        return month + '/' + days + ' ' + UnixTimeToDate
    }
    return UnixTimeToDate
}

document.addEventListener("DOMContentLoaded", function () {

    // bootstrap tooltip init
    (() => {
        const tooltipTriggerList = document.querySelectorAll('[data-bs-toggle="tooltip"]')
        const _tooltipList = [...tooltipTriggerList].map(tooltipTriggerEl => new bootstrap.Tooltip(tooltipTriggerEl));
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

    // htmx response error handle
    (() => {
        document.body.addEventListener('htmx:responseError', function (evt) {
            let message = `<div class="err-message">${evt.detail.error}/<div>`;
            evt.detail.target.innerHTML = message;
            evt.detail.target.classList.add("htmx-settling");
            setTimeout(() => {
                evt.detail.target.classList.remove("htmx-settling");
            }, 2000);
        });
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

    // traffic logic
    (() => {

        const traffic = document.querySelector("#traffic");
        if (!traffic) {
            return;
        }
        window.traffic_period = "1d";

        const pid = traffic.getAttribute("x-data-pid");
        async function request_traffic(pid) {
            let params = new URLSearchParams();
            params.append("period", window.traffic_period);
            if (pid) {
                params.append("pid", pid);
            }
            let resp = await fetch(`/traffic/requests`, {
                method: "POST",
                body: params,
            })
            let data = await resp.json();

            let c = document.getElementById('requests-chart');
            c.style.height = "200px";
            c.style.width = "100%";

            let loading = document.getElementById('requests-chart-loading');
            loading.style.display = "none";

            const total_div = document.getElementById("requests-total");
            total_div.classList.remove("d-none");
            document.getElementById('requests-total-success').innerText = data['typ-success']?.total || 0;
            document.getElementById('requests-total-error').innerText = data['typ-error']?.total || 0;

            let chart = echarts.init(c, null);
            let option = {
                title: {
                    show: false,
                },
                tooltip: {
                    trigger: "axis",
                },
                xAxis: {
                    type: 'time',
                    axisLabel: {
                        formatter: function (value, index) {
                            return unix2hour(parseInt(value)) // js use milliseconds
                        },
                    },
                    splitNumber: 3,
                },
                yAxis: {
                    show: false
                },
                grid: {
                    top: 0,
                    left: 0,
                    right: 0,
                    bottom: 20,
                },
                series: [
                    {
                        type: 'line',
                        data: data["typ-success"]?.values || [],
                        name: "success",
                        smooth: true,
                        symbol: "none",
                    },
                    {
                        type: 'line',
                        data: data['typ-error']?.values || [],
                        name: "error",
                        smooth: true,
                        symbol: "none",
                    },
                ],
                color: ['#5470c6', '#B03A5B'],
            }
            chart.setOption(option);
        }
        request_traffic(pid);

        async function flow_traffic(pid) {
            let params = new URLSearchParams();
            params.append("period", window.traffic_period);
            if (pid) {
                params.append("pid", pid);
            }
            let resp = await fetch(`/traffic/flows`, {
                method: "POST",
                body: params,
            })
            let data = await resp.json();

            let c = document.getElementById('flows-chart');
            c.style.height = "200px";
            c.style.width = "100%";

            let loading = document.getElementById('flows-chart-loading');
            loading.style.display = "none";

            const total_div = document.getElementById("flows-total");
            total_div.classList.remove("d-none");
            document.getElementById('flows-total-out').innerText = friendly_bytesize(data['typ-main_out_bytes']?.total || 0, true);
            document.getElementById('flows-total-in').innerText = friendly_bytesize(data['typ-main_in_bytes']?.total || 0, true);

            let chart = echarts.init(c, null);
            let option = {
                title: {
                    show: false,
                },
                tooltip: {
                    trigger: "axis",
                },
                xAxis: {
                    type: 'time',
                    axisLabel: {
                        formatter: function (value, index) {
                            return unix2hour(parseInt(value)) // js use milliseconds
                        },
                    },
                    splitNumber: 3,
                },
                yAxis: {
                    show: false
                },
                grid: {
                    top: 0,
                    left: 0,
                    right: 0,
                    bottom: 20,
                },
                series: [
                    {
                        type: 'line',
                        data: data["typ-main_out_bytes"]?.values || [],
                        name: "out",
                        smooth: true,
                        symbol: "none",
                    },
                    {
                        type: 'line',
                        data: data['typ-main_in_bytes']?.values || [],
                        name: "in",
                        smooth: true,
                        symbol: "none",
                    },
                ],
                color: ['#91cc75', '#fac858'],
            }
            chart.setOption(option);
        }
        flow_traffic(pid);

        document.querySelectorAll(".traffic-period-select").forEach((el) => {
            el.addEventListener("click", function () {
                let period = el.getAttribute("x-data-v");
                if (period == window.traffic_period) {
                    return;
                }

                let text = el.innerText;
                document.getElementById("traffic-period-btn").innerText = text;
                window.traffic_period = period;

                request_traffic(pid);
                flow_traffic(pid);
            })
        });
    })();
})