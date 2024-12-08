let roomListDiv = document.getElementById("room-list");
let messagesDiv = document.getElementById("messages");
let newMessageForm = document.getElementById("new-message");
let newRoomForm = document.getElementById("new-room");
let statusDiv = document.getElementById("status");

let roomTemplate = document.getElementById("room");
let messageTemplate = document.getElementById("message");

let messageField = newMessageForm.querySelector("#message");
let usernameField = document.getElementById("username");
let roomNameField = newRoomForm.querySelector("#name");
let fileField = document.getElementById("file")


var STATE = {
    room: "lobby",
    rooms: {},
    connected: false,
};

document.addEventListener('DOMContentLoaded', async () => {
    const token = localStorage.getItem('jwt_token');

    console.log(token)

    if (!token) {
        window.location.href = "/login";
        return;
    }


    const response = await fetch('/messages', {
        method: 'GET',
        headers: {
            'Authorization': `Bearer ${token}`,
        }
    });

    if (!response.ok) {
        window.location.href = "/login";
    }

    const messages = await response.json();

    const messagesContainer = document.getElementById('messages');
    const messageTemplate = document.getElementById('message');

    messages.forEach(msg => {
        const messageElement = messageTemplate.content.cloneNode(true);
        messageElement.querySelector('.username').textContent = msg.username;
        messageElement.querySelector('.text').textContent = msg.message;
        const fileContainer = messageElement.querySelector('#file-container');

        if (msg.file) {
            const decodedFilePath = msg.file;

            const fileLink = document.createElement('a');
            fileLink.classList.add('file-link');
            fileLink.href = "\"C:\\Users\\User\\Desktop\\" + decodedFilePath;
            fileLink.target = '_blank';
            fileLink.textContent = 'Download file';

            fileContainer.appendChild(fileLink);
        }
        messagesContainer.appendChild(messageElement);

    });
});

function hashColor(str) {
    let hash = 0;
    for (var i = 0; i < str.length; i++) {
        hash = str.charCodeAt(i) + ((hash << 5) - hash);
        hash = hash & hash;
    }

    return `hsl(${hash % 360}, 100%, 70%)`;
}


function addRoom(name) {
    if (STATE[name]) {
        changeRoom(name);
        return false;
    }

    var node = roomTemplate.content.cloneNode(true);
    var room = node.querySelector(".room");
    room.addEventListener("click", () => changeRoom(name));
    room.textContent = name;
    room.dataset.name = name;
    roomListDiv.appendChild(node);

    STATE[name] = [];
    changeRoom(name);
    return true;
}


function addMessage(room, username, message, push = false, filePath = null) {
    if (push) {
        STATE[room].push({username, message, filePath});
    }

    if (STATE.room == room) {
        var node = messageTemplate.content.cloneNode(true);
        node.querySelector(".message .username").textContent = username;
        node.querySelector(".message .username").style.color = hashColor(username);
        node.querySelector(".message .text").textContent = message;

        if (filePath) {
            const fileContainer = document.createElement("div");
            fileContainer.classList.add("file-container");

            if (filePath.startsWith('data:image') || filePath.endsWith('.jpg') || filePath.endsWith('.jpeg') || filePath.endsWith('.png')) {
                const img = document.createElement("img");
                img.src = "\"C:\\Users\\User\\Desktop\\" +`${filePath}`;
                img.alt = "Sent image";
                img.classList.add("file-image");

                fileContainer.appendChild(img);
            } else {
                const link = document.createElement("a");
                link.href = "\"C:\\Users\\User\\Desktop\\" + filePath;
                link.textContent = "Download file";
                link.target = "_blank";
                link.classList.add("file-link");

                fileContainer.appendChild(link);
            }

            node.querySelector(".message").appendChild(fileContainer);
        }

        messagesDiv.appendChild(node);
    }
}


function subscribe(uri) {
    let ws;
    let retryTime = 1;

    function connect(uri) {
        ws = new WebSocket(uri);

        ws.onopen = () => {
            setConnectedStatus(true);
            console.log(`connected to websocket at ${uri}`);
            retryTime = 1;
        };

        ws.onmessage = (ev) => {
            const msg = JSON.parse(ev.data);
            if (!msg.message || !msg.room || !msg.username) return;
            addMessage(msg.room, msg.username, msg.message, msg.file, true);
        };

        ws.onerror = () => {
            setConnectedStatus(false);
            ws.close();

            let timeout = retryTime;
            retryTime = Math.min(64, retryTime * 2);
            console.log(`connection lost. attempting to reconnect in ${timeout}s`);
            setTimeout(() => connect(uri), timeout * 1000);
        };

        ws.onclose = () => {
            setConnectedStatus(false);
            console.log('WebSocket closed');
        };
    }

    connect(uri);
}

function setConnectedStatus(status) {
    STATE.connected = status;
    statusDiv.className = status ? "connected" : "reconnecting";
}

async function init() {
    addRoom("lobby");
    const token = localStorage.getItem('jwt_token');
    const responseUser = await fetch('/user', {
        method: 'GET',
        headers: {
            'Authorization': `Bearer ${token}`,
        }
    });
    const user = await responseUser.json();
    usernameField.value = user.username;

    newMessageForm.addEventListener("submit", (e) => {
        e.preventDefault();

        const room = STATE.room;
        const message = messageField.value;
        const username = usernameField.value || "guest";
        const file = fileField.files[0];

        if (!message || !username) return;

        const formData = new FormData();
        formData.append('room', room);
        formData.append('username', username);
        formData.append('message', message);

        if (file) {
            const reader = new FileReader();
            reader.onloadend = function() {
                const encodedFilePath = file.name;
                formData.append('file', encodedFilePath);


                if (STATE.connected) {
                    fetch("/message", {
                        method: "POST",
                        body: formData,
                    }).then((response) => {
                        if (response.ok) messageField.value = "";
                    });
                }
            };
            reader.readAsDataURL(file);
        } else {
            if (STATE.connected) {
                fetch("/message", {
                    method: "POST",
                    body: formData,
                }).then((response) => {
                    if (response.ok) messageField.value = "";
                });
            }
        }
    });



    newRoomForm.addEventListener("submit", (e) => {
        e.preventDefault();

        const room = roomNameField.value;
        if (!room) return;

        roomNameField.value = "";
        if (!addRoom(room)) return;

        addMessage(room, "Rocket", `Look, your own "${room}" room! Nice.`, true);
    });

    subscribe("ws://your-server-url/events");
}

init();
