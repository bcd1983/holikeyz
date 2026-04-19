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

    Layout.preferredWidth: Kirigami.Units.gridUnit * 18
    Layout.preferredHeight: Kirigami.Units.gridUnit * 28
    Layout.minimumWidth: Kirigami.Units.gridUnit * 15
    Layout.minimumHeight: Kirigami.Units.gridUnit * 26

    function sceneMatches(scene) {
        return root.isOn
            && Math.abs(root.brightness - scene.brightness) <= 1
            && Math.abs(root.temperature - scene.temperature) <= 50
    }

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: Kirigami.Units.largeSpacing
        spacing: Kirigami.Units.largeSpacing

        // ------- Header card: icon + title + on/off -------
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
                            if (!root.connected) return "Not connected"
                            if (!root.isOn) return "Off"
                            return root.brightness + "%  ·  " + root.temperature + "K"
                        }
                    }
                }

                QQC2.Switch {
                    checked: root.isOn
                    enabled: root.connected
                    onToggled: root.setOn(checked)
                }
            }
        }

        // ------- Brightness -------
        ColumnLayout {
            Layout.fillWidth: true
            spacing: Kirigami.Units.smallSpacing
            visible: root.connected
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

        // ------- Temperature -------
        ColumnLayout {
            Layout.fillWidth: true
            spacing: Kirigami.Units.smallSpacing
            visible: root.connected
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

        // ------- Scenes -------
        PlasmaComponents.Label {
            text: "Scenes"
            visible: root.connected
            Layout.topMargin: Kirigami.Units.smallSpacing
            opacity: 0.8
        }

        GridLayout {
            Layout.fillWidth: true
            Layout.fillHeight: true
            visible: root.connected
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

        // ------- Disconnected state -------
        ColumnLayout {
            Layout.fillWidth: true
            Layout.fillHeight: true
            visible: !root.connected
            spacing: Kirigami.Units.largeSpacing

            Item { Layout.fillHeight: true }

            Kirigami.Icon {
                Layout.alignment: Qt.AlignHCenter
                source: "network-disconnect"
                Layout.preferredWidth: Kirigami.Units.iconSizes.huge
                Layout.preferredHeight: Kirigami.Units.iconSizes.huge
                opacity: 0.6
            }
            PlasmaExtras.Heading {
                Layout.alignment: Qt.AlignHCenter
                level: 3
                text: "Can't reach light"
            }
            PlasmaComponents.Label {
                Layout.alignment: Qt.AlignHCenter
                text: root.ip + ":" + root.port + "  ·  " + (root.errorMessage || "unreachable")
                opacity: 0.6
            }
            PlasmaComponents.Button {
                Layout.alignment: Qt.AlignHCenter
                text: "Configure"
                icon.name: "configure"
                onClicked: Plasmoid.internalAction("configure").trigger()
            }

            Item { Layout.fillHeight: true }
        }
    }
}
