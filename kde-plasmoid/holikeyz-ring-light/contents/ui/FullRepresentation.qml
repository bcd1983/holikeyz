import QtQuick
import QtQuick.Layouts
import QtQuick.Controls as QQC2
import org.kde.plasma.core as PlasmaCore
import org.kde.plasma.components as PlasmaComponents
import org.kde.plasma.extras as PlasmaExtras
import org.kde.plasma.plasmoid
import org.kde.kirigami as Kirigami

Item {
    id: fullRoot

    Layout.preferredWidth: Kirigami.Units.gridUnit * 19
    Layout.preferredHeight: Kirigami.Units.gridUnit * 30
    Layout.minimumWidth: Kirigami.Units.gridUnit * 16
    Layout.minimumHeight: Kirigami.Units.gridUnit * 27

    // When true, the middle section shows the discover list instead of scenes.
    property bool showDiscover: false

    function sceneMatches(scene) {
        return root.isOn
            && Math.abs(root.brightness - scene.brightness) <= 1
            && Math.abs(root.temperature - scene.temperature) <= 50
    }

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: Kirigami.Units.largeSpacing
        spacing: Kirigami.Units.largeSpacing

        // ---------- Header ----------
        Rectangle {
            Layout.fillWidth: true
            Layout.preferredHeight: Kirigami.Units.gridUnit * 5
            radius: Kirigami.Units.largeSpacing
            color: root.isOn ? Qt.rgba(Kirigami.Theme.highlightColor.r,
                                       Kirigami.Theme.highlightColor.g,
                                       Kirigami.Theme.highlightColor.b, 0.10)
                             : Qt.rgba(0, 0, 0, 0.05)
            border.width: 1
            border.color: Qt.rgba(1, 1, 1, 0.08)
            Behavior on color { ColorAnimation { duration: 180 } }

            RowLayout {
                anchors.fill: parent
                anchors.leftMargin: Kirigami.Units.largeSpacing
                anchors.rightMargin: Kirigami.Units.largeSpacing
                spacing: Kirigami.Units.largeSpacing

                Rectangle {
                    Layout.preferredWidth: Kirigami.Units.iconSizes.large + Kirigami.Units.largeSpacing
                    Layout.preferredHeight: Kirigami.Units.iconSizes.large + Kirigami.Units.largeSpacing
                    radius: width / 2
                    color: root.isOn ? Kirigami.Theme.highlightColor
                                     : Qt.rgba(0.4, 0.4, 0.4, 0.25)
                    Behavior on color { ColorAnimation { duration: 180 } }
                    Kirigami.Icon {
                        anchors.centerIn: parent
                        width: Kirigami.Units.iconSizes.large
                        height: width
                        source: "im-light"
                        color: root.isOn ? "white" : Kirigami.Theme.textColor
                        isMask: true
                    }
                }

                ColumnLayout {
                    Layout.fillWidth: true
                    spacing: 2
                    PlasmaExtras.Heading {
                        level: 2
                        text: "Ring Light"
                        Layout.fillWidth: true
                        elide: Text.ElideRight
                    }
                    PlasmaComponents.Label {
                        Layout.fillWidth: true
                        opacity: 0.7
                        elide: Text.ElideRight
                        text: {
                            if (!root.connected) return "Service offline"
                            if (!root.activeIp) return "No light selected"
                            if (!root.isOn) return "Off  ·  " + root.activeIp
                            return root.brightness + "% · " + root.temperature + "K  ·  " + root.activeIp
                        }
                    }
                }

                QQC2.ToolButton {
                    icon.name: fullRoot.showDiscover ? "go-previous" : "network-wireless"
                    display: QQC2.AbstractButton.IconOnly
                    enabled: root.connected
                    QQC2.ToolTip.visible: hovered
                    QQC2.ToolTip.text: fullRoot.showDiscover ? i18n("Back") : i18n("Discover lights")
                    onClicked: {
                        if (!fullRoot.showDiscover) {
                            root.discover(4, function(ok) {})
                        }
                        fullRoot.showDiscover = !fullRoot.showDiscover
                    }
                }

                QQC2.Switch {
                    checked: root.isOn
                    enabled: root.connected && root.activeIp.length > 0
                    onToggled: root.setOn(checked)
                }
            }
        }

        // ---------- Brightness ----------
        ColumnLayout {
            Layout.fillWidth: true
            spacing: Kirigami.Units.smallSpacing
            visible: root.connected && root.activeIp.length > 0 && !fullRoot.showDiscover
            opacity: root.isOn ? 1.0 : 0.5
            Behavior on opacity { NumberAnimation { duration: 150 } }

            RowLayout {
                Layout.fillWidth: true
                Kirigami.Icon {
                    source: "brightness-low"
                    Layout.preferredWidth: Kirigami.Units.iconSizes.smallMedium
                    Layout.preferredHeight: Kirigami.Units.iconSizes.smallMedium
                    opacity: 0.7
                }
                PlasmaComponents.Label { text: "Brightness" }
                Item { Layout.fillWidth: true }
                PlasmaComponents.Label {
                    text: root.brightness + "%"
                    opacity: 0.7
                }
            }
            QQC2.Slider {
                id: brightnessSlider
                Layout.fillWidth: true
                from: 1; to: 100; stepSize: 1
                value: root.brightness
                enabled: root.isOn && root.connected
                onMoved: debounceBright.restart()
                Timer {
                    id: debounceBright
                    interval: 80
                    onTriggered: root.setBrightness(brightnessSlider.value)
                }
            }
        }

        // ---------- Temperature ----------
        ColumnLayout {
            Layout.fillWidth: true
            spacing: Kirigami.Units.smallSpacing
            visible: root.connected && root.activeIp.length > 0 && !fullRoot.showDiscover
            opacity: root.isOn ? 1.0 : 0.5
            Behavior on opacity { NumberAnimation { duration: 150 } }

            RowLayout {
                Layout.fillWidth: true
                Kirigami.Icon {
                    source: "color-picker-white"
                    Layout.preferredWidth: Kirigami.Units.iconSizes.smallMedium
                    Layout.preferredHeight: Kirigami.Units.iconSizes.smallMedium
                    opacity: 0.7
                }
                PlasmaComponents.Label { text: "Temperature" }
                Item { Layout.fillWidth: true }
                PlasmaComponents.Label {
                    text: root.temperature + "K"
                    opacity: 0.7
                }
            }
            Item {
                Layout.fillWidth: true
                Layout.preferredHeight: tempSlider.height
                Rectangle {
                    anchors.left: parent.left
                    anchors.right: parent.right
                    anchors.verticalCenter: parent.verticalCenter
                    height: 4
                    radius: 2
                    gradient: Gradient {
                        orientation: Gradient.Horizontal
                        GradientStop { position: 0.0; color: "#ffb46b" }
                        GradientStop { position: 0.5; color: "#fff3d6" }
                        GradientStop { position: 1.0; color: "#bcd6ff" }
                    }
                    opacity: tempSlider.enabled ? 0.9 : 0.35
                }
                QQC2.Slider {
                    id: tempSlider
                    anchors.fill: parent
                    from: 2900; to: 7000; stepSize: 50
                    value: root.temperature
                    enabled: root.isOn && root.connected
                    background: Item {}
                    onMoved: debounceTemp.restart()
                    Timer {
                        id: debounceTemp
                        interval: 80
                        onTriggered: root.setTemperature(tempSlider.value)
                    }
                }
            }
        }

        // ---------- Scene grid (normal view) ----------
        PlasmaComponents.Label {
            text: "Scenes"
            visible: root.connected && root.activeIp.length > 0 && !fullRoot.showDiscover
            Layout.topMargin: Kirigami.Units.smallSpacing
            opacity: 0.8
        }

        GridLayout {
            Layout.fillWidth: true
            Layout.fillHeight: true
            visible: root.connected && root.activeIp.length > 0 && !fullRoot.showDiscover
            columns: 3
            rowSpacing: Kirigami.Units.smallSpacing
            columnSpacing: Kirigami.Units.smallSpacing

            Repeater {
                model: root.scenes
                delegate: SceneCard {
                    Layout.fillWidth: true
                    Layout.fillHeight: true
                    Layout.minimumHeight: Kirigami.Units.gridUnit * 4
                    sceneId: modelData.id
                    sceneName: modelData.name
                    active: fullRoot.sceneMatches(modelData)
                    onActivated: root.applyScene(modelData)
                }
            }
        }

        // ---------- Discover view ----------
        ColumnLayout {
            Layout.fillWidth: true
            Layout.fillHeight: true
            visible: fullRoot.showDiscover
            spacing: Kirigami.Units.smallSpacing

            RowLayout {
                Layout.fillWidth: true
                PlasmaExtras.Heading {
                    level: 4
                    text: root.discovering ? i18n("Scanning network…") : i18n("Discovered lights")
                    Layout.fillWidth: true
                }
                QQC2.BusyIndicator {
                    running: root.discovering
                    Layout.preferredWidth: Kirigami.Units.iconSizes.medium
                    Layout.preferredHeight: Kirigami.Units.iconSizes.medium
                }
                QQC2.ToolButton {
                    icon.name: "view-refresh"
                    enabled: !root.discovering && root.connected
                    onClicked: root.discover(4, function(ok) {})
                    QQC2.ToolTip.visible: hovered
                    QQC2.ToolTip.text: i18n("Re-scan")
                }
            }

            QQC2.ScrollView {
                Layout.fillWidth: true
                Layout.fillHeight: true
                clip: true
                contentWidth: availableWidth

                ColumnLayout {
                    width: parent.width
                    spacing: Kirigami.Units.smallSpacing

                    Repeater {
                        model: root.discoveredLights
                        delegate: Rectangle {
                            Layout.fillWidth: true
                            Layout.preferredHeight: Kirigami.Units.gridUnit * 3
                            radius: Kirigami.Units.smallSpacing
                            border.width: isActive ? 2 : 1
                            border.color: isActive ? Kirigami.Theme.highlightColor
                                                   : Qt.rgba(1, 1, 1, 0.08)
                            color: ma.containsMouse ? Qt.rgba(1, 1, 1, 0.06) : "transparent"
                            Behavior on color { ColorAnimation { duration: 80 } }

                            property bool isActive: modelData.ip === root.activeIp
                                                  && modelData.port === root.activePort

                            RowLayout {
                                anchors.fill: parent
                                anchors.leftMargin: Kirigami.Units.largeSpacing
                                anchors.rightMargin: Kirigami.Units.largeSpacing
                                spacing: Kirigami.Units.largeSpacing

                                Kirigami.Icon {
                                    source: "im-light"
                                    Layout.preferredWidth: Kirigami.Units.iconSizes.medium
                                    Layout.preferredHeight: Kirigami.Units.iconSizes.medium
                                    opacity: 0.8
                                }
                                ColumnLayout {
                                    Layout.fillWidth: true
                                    spacing: 2
                                    PlasmaComponents.Label {
                                        Layout.fillWidth: true
                                        text: (modelData.name || "").split(".")[0] || "Ring Light"
                                        elide: Text.ElideRight
                                        font.bold: true
                                    }
                                    PlasmaComponents.Label {
                                        Layout.fillWidth: true
                                        text: modelData.ip + ":" + modelData.port
                                        elide: Text.ElideRight
                                        opacity: 0.7
                                    }
                                }
                                Kirigami.Icon {
                                    visible: parent.parent.isActive
                                    source: "emblem-ok-symbolic"
                                    Layout.preferredWidth: Kirigami.Units.iconSizes.small
                                    Layout.preferredHeight: Kirigami.Units.iconSizes.small
                                    color: Kirigami.Theme.highlightColor
                                    isMask: true
                                }
                            }

                            MouseArea {
                                id: ma
                                anchors.fill: parent
                                hoverEnabled: true
                                cursorShape: Qt.PointingHandCursor
                                onClicked: {
                                    root.selectLight(modelData.ip, modelData.port)
                                    fullRoot.showDiscover = false
                                }
                            }
                        }
                    }

                    PlasmaComponents.Label {
                        visible: !root.discovering && root.discoveredLights.length === 0
                        text: i18n("No lights found. Make sure the light is powered on and on the same network.")
                        wrapMode: Text.WordWrap
                        Layout.fillWidth: true
                        opacity: 0.6
                        Layout.topMargin: Kirigami.Units.largeSpacing
                        horizontalAlignment: Text.AlignHCenter
                    }
                }
            }
        }

        // ---------- Service-offline state ----------
        ColumnLayout {
            Layout.fillWidth: true
            Layout.fillHeight: true
            visible: !root.connected && !fullRoot.showDiscover
            spacing: Kirigami.Units.largeSpacing

            Item { Layout.fillHeight: true }

            Kirigami.Icon {
                Layout.alignment: Qt.AlignHCenter
                source: "network-offline"
                Layout.preferredWidth: Kirigami.Units.iconSizes.huge
                Layout.preferredHeight: Kirigami.Units.iconSizes.huge
                opacity: 0.6
            }
            PlasmaExtras.Heading {
                Layout.alignment: Qt.AlignHCenter
                level: 3
                text: i18n("Service offline")
            }
            PlasmaComponents.Label {
                Layout.alignment: Qt.AlignHCenter
                Layout.fillWidth: true
                text: i18n("Install the holikeyz D-Bus service, then click Retry.")
                wrapMode: Text.WordWrap
                horizontalAlignment: Text.AlignHCenter
                opacity: 0.7
            }
            PlasmaComponents.Button {
                Layout.alignment: Qt.AlignHCenter
                text: i18n("Retry")
                icon.name: "view-refresh"
                onClicked: root.refreshStatus()
            }

            Item { Layout.fillHeight: true }
        }
    }
}
