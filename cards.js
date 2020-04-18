var ws = new WebSocket("ws://127.0.0.1:7878/socketserver");

var name = undefined;

var colors = ["White", "Yellow", "Red", "Blue", "Green", "Rainbow"];
var values = ['One', 'Two', 'Three', 'Four', 'Five'];

ws.onmessage = function (event) {
    console.log(event.data);
    if (event.data.startsWith("ERROR")) {
        return;
    } else if (event.data.startsWith("Success")) {
        return;
    }

    FormatGame(JSON.parse(event.data));
}

function EnterRoom() {
    name = $("#name")["0"].value;

    document.getElementsByClassName('container')[0].innerHTML += `<div class="d-flex align-items-center">\n
        <strong>Loading...</strong>\n
        <div class="spinner-border ml-auto" role="status" aria-hidden="true"></div>\n
    </div>`;

    ws.send(`/join ${name}`);
}

function Play(index) {
    console.log(`Play ${index}\n`);
    ws.send(`/play ${index}`);
}

function Discard(index) {
    console.log(`Discard ${index}\n`);
    ws.send(`/discard ${index}`);
}

function GiveColorHint(color, player) {
    console.log(`GiveHint ${color} to ${player}\n`);
    ws.send(`/hint {"variant":"ColorHint","fields":["${color}"]} ${player}`);
}

function GiveValueHint(value, player) {
    console.log(`GiveHint ${value} to ${player}\n`);
    ws.send(`/hint {"variant":"ValueHint","fields":["${value}"]} ${player}`);
}

function CardToHTML(card, index, mine) {
    var strtonum = {
        'One': 1,
        'Two': 2,
        'Three': 3,
        'Four': 4,
        'Five': 5,
        '??': "??",
        'Empty': "Empty",
    };

    var props = "";
    if (index != -1) {
        props += `data-index=${index} `;
    }

    if (!mine) {
        props += "disabled";
    }

    var inner = `<span class="">${card.color || "Unknown"} ${strtonum[card.value || "??"]}</span>\n`;
    var outer = `<div style="float: left">\n<button type="button" class="btn hcard ${card.color || "Unknown"}" data-toggle="popover" ${props}>\n${inner}</button>\n</div>\n`;

    return outer;
}

function FormatCards(cards, includeIndex, mine) {
    var group = "";
    var index = -1;
    cards.forEach(function(x) {
        if (includeIndex) {
            index += 1;
        }

        group += CardToHTML(x, index, mine) + "\n";
    });

    return group;
}

function FormatPlayers(players, hints, turn) {
    var group = "";
    var player = 0;

    players.forEach(function(x) {
        var me = x.name == name;

        group += `<div class="card m-3">\n
        <div class="card-body">\n
            <h5 class="card-title"><span ${player==turn ? 'class="current-player"' : ""}>${x.name}</span>
                <button type="button" class="btn btn-secondary ml-5 give-hint $" data-toggle="popover" data-player=${x.name} ${(hints > 0 && !me) ? "" : "disabled"}>\n
                    <span>${me ? "Me!" : "Give Hint!"}</span>\n
                </button>\n
            </h5>\n`;
        group += FormatCards(x.cards, me, me);
        group += "</div>\n</div>\n";

        player += 1;
    });

    return group;
}

function FormatStats(hints, bombs) {
    var hintGroup = "";
    for (let i = 1; i <= 8; i++) {
        if (i <= hints) {
            hintGroup += '<button type="button" class="btn btn-primary btn-xs">\n<span>X</span>\n</button>\n';
        } else {
            hintGroup += '<button type="button" class="btn btn-primary btn-xs" disabled >\n<span>-</span>\n</button>\n';
        }
    }

    var bombGroup = "";
    for (let i = 1; i <= bombs; i++) {
        bombGroup += '<button type="button" class="btn btn-dark" disabled>\n<span>Boom!</span>\n</button>\n';
    }

    return `<div class="card m-3">\n
        <div class="card-body">\n
            <div id="hint-icons">\n
                <span>Hints: </span>\n
                ${hintGroup}
            </div>\n
            <div id="bomb-icons">\n
                <span>Bombs: </span>\n
                ${bombGroup}
            </div>\n
        </div>\n
    </div>\n`;
}

function FormatDiscarded(discarded) {
    return `<div class="card m-3">\n
        <div class="card-body">\n
            <h5 class="card-title">Discarded</h5>\n
            <div id="discard">\n
                ${FormatCards(discarded, false, false)}
            </div>\n
        </div>\n
    </div>\n`;
}

function FormatPiles(piles) {
    var pileSizeToStr = {
        1: 'One',
        2: 'Two',
        3: 'Three',
        4: 'Four',
        5: 'Five',
        0: "Empty",
    };
    var group = "";

    for (var i = 0; i < colors.length; i++) {
        group += CardToHTML({
            "color": colors[i], 
            "value": pileSizeToStr[piles[i]],
        }, false, false) + "\n";
    }

    return `<div class="card">
        <div class="card-body">
            <h5 class="card-title">Piles</h5>
            <div id="piles">
                ${group}
            </div>
        </div>
    </div>`;
}

function FormatGame(update) {
    var game = update.view;
    var update_message = update.action;
    var html ="";
    
    if (game["state"] == "Playing") {
        html = `
        <div class="container-fluid">\n
            ${update_message ? `<div class="jumbotron">\n
                <h1>Last Action</h1>\n
                <p>${update_message}</p>\n
            </div>\n` : ""}
            <div class="row">\n
                <div id="players" class="col-md-6">\n
                    ${FormatPlayers(game["players"], game["hints_left"], game["turn"])}
                </div>\n
                <div class="col-md-6">\n
                    ${FormatStats(game["hints_left"], game["bombs"])}
                    ${FormatDiscarded(game["discard"])}
                </div>\n
            </div>\n
            <div style="width: 100%">\n
                <centering>\n
                    <div class="card">\n
                        <div class="card-body mx-auto">\n
                            ${FormatPiles(game["piles"])}
                        </div>\n
                    </div>\n
                </centering>\n
            </div>\n
        </div>\n
        `
    } else {
        html = `
        <div class="container-fluid">\n
            <div class="jumbotron">\n
                <h1>You have ${game["state"].toLowerCase()} the game!</h1>\n
            </div>\n
        </div>\n
        `
    }

    console.log(html)

    $('body')[0].innerHTML = html;

    $(function () { 
        $('.hcard').popover({
            placement: 'auto',
            html: true,
            sanitize: false,
            title: 'Actions',
            content: function () {
                return `<div class="btn-group"><button type="button" class="btn btn-success" onclick="Play(${this.dataset.index});"><span>Play</span></button><button type="button" class="btn btn-dark" onclick="Discard(${this.dataset.index});"><span>Discard</span></button></div>`;
            },
            container: 'body',
            trigger: 'focus'
        });
    });

    $(function () { 
        $('.give-hint').popover({
            placement: 'auto',
            html: true,
            sanitize: false,
            title: 'Actions',
            content: function () {
                console.log(this)
                var name = this.dataset.player;
                var group = '<div class="btn-group">\n';
                group += '<button type="button" class="btn btn-primary dropdown-toggle" data-toggle="dropdown">Color</button>\n';
                group += '<div class="dropdown-menu">\n';

                colors.forEach(function (color) {
                    group += `<button type="button" class="dropdown-item ${color} hint" onclick="GiveColorHint('${color}', '${name}');">\n
                        <span>${color}</span>\n
                    </button>\n`;
                });

                group += '</div>\n</div>\n'

                group += '<div class="btn-group">\n';
                group += '<button type="button" class="btn btn-primary dropdown-toggle" data-toggle="dropdown">Value</button>\n';
                group += '<div class="dropdown-menu">';

                values.forEach(function (value) {
                    group += `<button type="button" class="dropdown-item unknown hint" onclick="GiveValueHint('${value}', '${name}');">\n
                        <span>${value}</span>\n
                    </button>\n`;
                });

                group += '</div>\n</div>\n'

                return group;
            },
            container: 'body',
            focus: 'trigger',
        });
    });
}