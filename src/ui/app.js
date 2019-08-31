// Actions which get sent to the backend.
var Action = (function () {
    "use strict";

    var send = function (action) {
        external.invoke(JSON.stringify(action));
    }

    return {
        boot: function () {
            this.log("Action.boot");
            // Fixme: window.onerror not in MSHTML webview!
            window.onerror = function (msg, uri, line) {
                this.error(msg, uri, line);
            };
            send({ type: "Initialize" });
        },
        build_app: function (name, url, directory) {
            send({ type: "Build", name: name, url: url, directory: directory });
        },
        choose_directory: function () {
            send({ type: "ChooseDirectory" });
        },
        log: function () {
            var args = [];
            for (var ii = 0; ii < arguments.length; ii++) {
                args.push(arguments[ii]);
            }
            send({ type: "Log", msg: args.join(" ") });
        },
        error: function (msg, uri, line) {
            send({ type: "Error", msg: msg, uri: uri, line: line });
        },
    }
})();

// Events coming from the backend.
var Event = (function () {
    "use strict";

    return {
        dispatch: function (event) {
            Action.log("event:", JSON.stringify(event));
            switch (event.type) {
                case "Initialized":
                    var dir = Gui.state(event).default_path;
                    Gui.set_directory(dir);
                    break;
                case "Error":
                    Gui.show_error(event);
                    break;
                case "DirectoryChosen":
                    Gui.set_directory(event.path);
                    break;
                case "BuildComplete":
                    Gui.build_complete();
                    break;
            }
        }
    }
})();

// GUI mutations.
var Gui = (function () {
    "use strict";

    var state = {
        platform: "",
        default_path: "",
    };

    var clone = function (obj) {
        return JSON.parse(JSON.stringify(obj));
    };

    return {
        // state merges any provided object and returns a read-only copy.
        state: function (new_state) {
            return clone($.extend(state, new_state));
        },
        boot: function () {
            $("#directory").on("click", function (e) {
                e.preventDefault();
                Action.choose_directory();
            });

            $("#build").on("click", function (e) {
                e.preventDefault();
                Action.build_app($("#name").val(), $("#url").val(), $("#directory").data().path);
            })

            $("#error-message").toggle(false);
        },
        set_directory: function (path) {
            var pattern;
            if (this.state().platform == "windows") {
                pattern = /:\\|\\/;
            } else {
                pattern = "/";
            }
            var bits = path.split(pattern).map(function (x) {
                return document.createTextNode(x);
            });
            var end = bits.pop();

            var button = $("#directory");
            button.empty();
            bits.forEach(function (bit) {
                button.append(bit);
                button.append($("<span> > </span>"));
            });
            button.append(end);
            button.data({ path: path });
        },
        build_complete: function () {
            $("#build-status").append("<span>done</span>");
        },
        show_error: function (err) {
            $("#error-message .body").text(err.msg);
            $("#error-message").toggle(true);
        }
    }
})();

$(document).ready(function () {
    Action.boot();
    Gui.boot();
});