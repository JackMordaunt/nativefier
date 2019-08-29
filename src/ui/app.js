var Config = {
    platform: null,
};

// Actions which get sent to the backend.
var Action = (function () {
    "use strict";

    var send = function (action) {
        external.invoke(JSON.stringify(action));
    }

    return {
        load_config: function () {
            send({ type: "LoadConfig" });
        },
        build_app: function (name, url, directory) {
            console.log({ type: "Build", name, url, directory });
            send({ type: "Build", name, url, directory });
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
                case "ConfigLoaded":
                    Config = event;
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
        boot: function () {
            // initialise event handlers 
            // connects ui events to Actions
            $("#directory").on("click", function (e) {
                e.preventDefault();
                Action.choose_directory();
            });

            $("#build").on("click", function (e) {
                e.preventDefault();
                Action.build_app($("#name").text(), $("#url").text(), $("#directory").path || "");
            })

            Action.load_config();
        },
        set_directory: function (path) {
            var pattern;
            if (Config.platform == "windows") {
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
            button.path = path;
        },
        build_complete: function () {
            $("#build-status").append($("done"));
        },
    }
})();

$(document).ready(Gui.boot);