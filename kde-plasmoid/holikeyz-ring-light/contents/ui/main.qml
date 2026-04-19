import QtQuick
import QtQuick.Layouts
import org.kde.plasma.plasmoid
import org.kde.plasma.core as PlasmaCore
import org.kde.plasma.plasma5support as P5Support
import org.kde.kirigami as Kirigami

PlasmoidItem {
    id: root

    // ----- D-Bus constants -----
    readonly property string busName: "com.holikeyz.RingLight"
    readonly property string objectPath: "/com/holikeyz/RingLight"
    readonly property string iface: "com.holikeyz.RingLight.Control"
    readonly property int pollInterval: Plasmoid.configuration.pollInterval

    // ----- Observable state -----
    property bool isOn: false
    property int brightness: 50
    property int temperature: 5600          // Kelvin
    property string activeIp: ""
    property int activePort: 9123
    property bool connected: false          // talking to the service
    property bool lightReachable: false     // service talking to the light
    property bool busy: false
    property string errorMessage: ""

    property var discoveredLights: []       // array of {name, ip, port}
    property bool discovering: false

    readonly property var scenes: [
        { "id": "daylight", "name": "Daylight", "brightness": 80, "temperature": 5600 },
        { "id": "warm",     "name": "Warm",     "brightness": 60, "temperature": 3200 },
        { "id": "cool",     "name": "Cool",     "brightness": 70, "temperature": 6500 },
        { "id": "reading",  "name": "Reading",  "brightness": 90, "temperature": 4500 },
        { "id": "video",    "name": "Video",    "brightness": 75, "temperature": 5000 },
        { "id": "relax",    "name": "Relax",    "brightness": 40, "temperature": 2900 }
    ]

    // ----- Subprocess runner -----
    P5Support.DataSource {
        id: exec
        engine: "executable"
        connectedSources: []

        property var pending: ({})
        property int seq: 0

        onNewData: (sourceName, data) => {
            var cb = pending[sourceName]
            if (cb) {
                cb(data["exit code"] === 0, (data.stdout || "").toString(), (data.stderr || "").toString())
                delete pending[sourceName]
            }
            disconnectSource(sourceName)
        }

        // Run a shell command; each invocation is uniquely tagged so the
        // executable engine does not dedupe concurrent calls.
        function run(cmd, cb) {
            var tagged = cmd + " # s=" + (++seq)
            pending[tagged] = cb
            connectSource(tagged)
        }
    }

    function shQuote(s) {
        return "'" + String(s).replace(/'/g, "'\\''") + "'"
    }

    function qdbusCmd(method) {
        var cmd = "qdbus6 " + root.busName + " " + root.objectPath + " " + root.iface + "." + method
        for (var i = 1; i < arguments.length; i++) {
            var a = arguments[i]
            cmd += " " + (typeof a === 'string' ? shQuote(a) : String(a))
        }
        return cmd
    }

    function qdbus(method /*, args..., cb */) {
        var cb = arguments[arguments.length - 1]
        var args = Array.prototype.slice.call(arguments, 0, arguments.length - 1)
        var cmd = qdbusCmd.apply(null, args)
        exec.run(cmd, cb)
    }

    // ----- Service interactions -----
    function refreshStatus() {
        qdbus("GetStatus", function(ok, out, err) {
            if (!ok) {
                root.connected = false
                root.lightReachable = false
                root.errorMessage = (err || "service unreachable").trim()
                return
            }
            root.connected = true
            var lines = out.trim().split("\n")
            if (lines.length >= 3) {
                root.isOn = lines[0].trim() === "true"
                root.brightness = parseInt(lines[1]) || 0
                root.temperature = parseInt(lines[2]) || 0
                root.lightReachable = root.brightness > 0 || root.temperature > 0
                root.errorMessage = ""
            }
        })
        qdbus("GetActiveLight", function(ok, out) {
            if (!ok) return
            var lines = out.trim().split("\n")
            if (lines.length >= 2) {
                root.activeIp = lines[0].trim()
                root.activePort = parseInt(lines[1]) || 9123
            }
        })
    }

    function toggle() {
        root.busy = true
        qdbus("Toggle", function(ok) { root.busy = false; if (ok) refreshStatus() })
    }
    function setOn(on) {
        root.busy = true
        qdbus(on ? "TurnOn" : "TurnOff", function(ok) { root.busy = false; if (ok) refreshStatus() })
    }
    function setBrightness(b) {
        qdbus("SetBrightness", Math.round(b), function() {})
        root.brightness = Math.round(b)
    }
    function setTemperature(k) {
        qdbus("SetTemperature", Math.round(k), function() {})
        root.temperature = Math.round(k)
    }
    function applyScene(scene) {
        qdbus("ApplyScene", scene.id, function(ok) {
            if (ok) refreshStatus()
            else {
                // Fallback in case the scene id isn't recognised
                qdbus("TurnOn", function() {
                    qdbus("SetBrightness", scene.brightness, function() {})
                    qdbus("SetTemperature", scene.temperature, function() { refreshStatus() })
                })
            }
        })
    }

    // Returns JSON array of {name, ip, port}
    function discover(timeoutSecs, cb) {
        root.discovering = true
        qdbus("Discover", timeoutSecs || 4, function(ok, out, err) {
            root.discovering = false
            if (!ok) {
                cb(false, err || "discover failed", [])
                return
            }
            try {
                var arr = JSON.parse(out.trim())
                root.discoveredLights = arr
                cb(true, "", arr)
            } catch (e) {
                cb(false, "malformed response", [])
            }
        })
    }

    function selectLight(ip, port) {
        qdbus("SetActiveLight", String(ip), port || 9123, function(ok, out, err) {
            if (ok) refreshStatus()
            else root.errorMessage = (err || "SetActiveLight failed").trim()
        })
    }

    // ----- Plasmoid shell -----
    Plasmoid.icon: "im-light"
    Plasmoid.title: "Ring Light"
    toolTipMainText: "Ring Light"
    toolTipSubText: {
        if (!root.connected) return "Service not running"
        if (!root.activeIp) return "No active light"
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
        onTriggered: root.refreshStatus()
    }
}
