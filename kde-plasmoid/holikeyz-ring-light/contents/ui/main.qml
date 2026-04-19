import QtQuick
import QtQuick.Layouts
import org.kde.plasma.plasmoid
import org.kde.plasma.core as PlasmaCore
import org.kde.kirigami as Kirigami

PlasmoidItem {
    id: root

    readonly property string ip: Plasmoid.configuration.ip
    readonly property int port: Plasmoid.configuration.port
    readonly property int pollInterval: Plasmoid.configuration.pollInterval
    readonly property string baseUrl: "http://" + ip + ":" + port + "/elgato"

    property bool isOn: false
    property int brightness: 50
    property int temperature: 5600
    property bool connected: false
    property bool busy: false
    property string errorMessage: ""

    readonly property var scenes: [
        { "id": "daylight", "name": "Daylight", "brightness": 80, "temperature": 5600 },
        { "id": "warm",     "name": "Warm",     "brightness": 60, "temperature": 3200 },
        { "id": "cool",     "name": "Cool",     "brightness": 70, "temperature": 6500 },
        { "id": "reading",  "name": "Reading",  "brightness": 90, "temperature": 4500 },
        { "id": "video",    "name": "Video",    "brightness": 75, "temperature": 5000 },
        { "id": "relax",    "name": "Relax",    "brightness": 40, "temperature": 2900 }
    ]

    function kelvinToApi(kelvin) {
        if (kelvin >= 7000) return 143
        if (kelvin <= 2900) return 344
        var normalized = (7000 - kelvin) / (7000 - 2900)
        return Math.round(143 + normalized * (344 - 143))
    }

    function apiToKelvin(value) {
        if (value <= 143) return 7000
        if (value >= 344) return 2900
        var normalized = (value - 143) / (344 - 143)
        return Math.round(7000 - normalized * (7000 - 2900))
    }

    function getState() {
        var xhr = new XMLHttpRequest()
        xhr.timeout = 2000
        xhr.onreadystatechange = function() {
            if (xhr.readyState !== XMLHttpRequest.DONE) return
            if (xhr.status === 200) {
                try {
                    var resp = JSON.parse(xhr.responseText)
                    if (resp.lights && resp.lights.length > 0) {
                        var light = resp.lights[0]
                        root.isOn = light.on === 1
                        root.brightness = light.brightness
                        root.temperature = root.apiToKelvin(light.temperature)
                        root.connected = true
                        root.errorMessage = ""
                    }
                } catch (e) {
                    root.connected = false
                    root.errorMessage = "Malformed response"
                }
            } else {
                root.connected = false
                root.errorMessage = xhr.status === 0 ? "No response" : ("HTTP " + xhr.status)
            }
        }
        xhr.ontimeout = function() {
            root.connected = false
            root.errorMessage = "Timeout"
        }
        try {
            xhr.open("GET", root.baseUrl + "/lights")
            xhr.send()
        } catch (e) {
            root.connected = false
            root.errorMessage = "Invalid URL"
        }
    }

    function setState(patch) {
        if (root.busy) return
        root.busy = true

        var light = {
            "on": (patch.on !== undefined ? patch.on : root.isOn) ? 1 : 0,
            "brightness": patch.brightness !== undefined ? patch.brightness : root.brightness,
            "temperature": patch.temperature !== undefined
                ? root.kelvinToApi(patch.temperature)
                : root.kelvinToApi(root.temperature)
        }
        var body = JSON.stringify({ "numberOfLights": 1, "lights": [light] })

        var xhr = new XMLHttpRequest()
        xhr.timeout = 2000
        xhr.onreadystatechange = function() {
            if (xhr.readyState !== XMLHttpRequest.DONE) return
            root.busy = false
            if (xhr.status === 200) {
                try {
                    var resp = JSON.parse(xhr.responseText)
                    if (resp.lights && resp.lights.length > 0) {
                        var l = resp.lights[0]
                        root.isOn = l.on === 1
                        root.brightness = l.brightness
                        root.temperature = root.apiToKelvin(l.temperature)
                        root.connected = true
                        root.errorMessage = ""
                    }
                } catch (e) {
                    root.errorMessage = "Malformed response"
                }
            } else {
                root.errorMessage = xhr.status === 0 ? "No response" : ("HTTP " + xhr.status)
            }
        }
        xhr.ontimeout = function() {
            root.busy = false
            root.errorMessage = "Timeout"
        }
        try {
            xhr.open("PUT", root.baseUrl + "/lights")
            xhr.setRequestHeader("Content-Type", "application/json")
            xhr.send(body)
        } catch (e) {
            root.busy = false
            root.errorMessage = "Invalid URL"
        }
    }

    function toggle() { setState({ "on": !root.isOn }) }
    function setOn(on) { setState({ "on": on }) }
    function setBrightness(b) { setState({ "on": true, "brightness": Math.round(b) }) }
    function setTemperature(k) { setState({ "on": true, "temperature": Math.round(k) }) }
    function applyScene(scene) {
        setState({ "on": true, "brightness": scene.brightness, "temperature": scene.temperature })
    }

    Plasmoid.icon: "im-light"
    Plasmoid.title: "Ring Light"
    toolTipMainText: "Ring Light"
    toolTipSubText: {
        if (!root.connected) return "Not connected to " + root.ip
        return root.isOn ? (root.brightness + "% · " + root.temperature + "K") : "Off"
    }

    preferredRepresentation: compactRepresentation
    compactRepresentation: CompactRepresentation {}
    fullRepresentation: FullRepresentation {}

    Timer {
        id: poller
        interval: root.pollInterval
        running: true
        repeat: true
        triggeredOnStart: true
        onTriggered: root.getState()
    }

    Connections {
        target: Plasmoid.configuration
        function onIpChanged() { root.getState() }
        function onPortChanged() { root.getState() }
    }
}
