// Actions which get sent to the backend.
var Action = (function () {
    "use strict";

    var send = function (action) {
        external.invoke(JSON.stringify(action));
    }

    return {
        boot: function () {
            send({ type: "Initialize" });
        },
        build_app: function (name, url, directory) {
            send({ type: "Build", name: name, url: url, directory: directory });
        },
        choose_directory: function () {
            send({ type: "ChooseDirectory" });
        },
    }
})();

// Events coming from the backend.
var Event = (function () {
    "use strict";

    return {
        dispatch: function (event) {
            switch (event.type) {
                case "Initialized":
                    var state = Gui.state(event.config);
                    Gui.set_directory(state.default_path);
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

    return {
        _state: {
            platform: null,
            default_path: null,
        },
        _clone: function (obj) {
            return JSON.parse(JSON.stringify(obj));
        },
        // State merges any provided object and returns a read-only copy.
        state: function (new_state) {
            return this._clone(Object.assign(this._state, new_state));
        },
        boot: function () {
            // initialise event handlers 
            // connects ui events to Actions
            $("#directory").on("click", function (e) {
                e.preventDefault();
                Action.choose_directory();
            });

            $("#build").on("click", function (e) {
                e.preventDefault();
                Action.build_app($("#name").val(), $("#url").val(), $("#directory").data().path);
            })

            Action.boot();
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
    }
})();

$(document).ready(Gui.boot);