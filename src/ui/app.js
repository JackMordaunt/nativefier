// Actions which get sent to the backend.
var Action = (function () {
    "use strict";

    var send = function (action) {
        external.invoke(JSON.stringify(action));
    }

    return {
        build_app: function (name, url, directory) {
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
        dispatch: function (msg) {
            switch (msg.type) {
                case "DirectoryChosen":
                    Gui.set_directory(msg.path);
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

            $("#build-test").on("click", function (e) {
                e.preventDefault();
                Action.build_app("SoundCloud", "https://soundcloud.com/app", "C:\\Users\\Jack\\Desktop");
            })
        },
        set_directory: function (path) {
            var bits = folder.split(/:\\|\\/).map(function (x) {
                return document.createTextNode(x);
            });
            var end = bits.pop();

            var button = $("#directory");
            button.empty();
            bits.forEach(function (bit) {
                button.append(bit);
                button.append($("<span>❱</span>"));
            });
            button.append(end);
        },
    }
})();