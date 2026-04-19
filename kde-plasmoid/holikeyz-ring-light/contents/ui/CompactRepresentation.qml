import QtQuick
import QtQuick.Layouts
import org.kde.plasma.core as PlasmaCore
import org.kde.kirigami as Kirigami

MouseArea {
    id: compactRoot

    hoverEnabled: true
    acceptedButtons: Qt.LeftButton | Qt.MiddleButton

    onClicked: (mouse) => {
        if (mouse.button === Qt.MiddleButton) {
            root.toggle()
        } else {
            root.expanded = !root.expanded
        }
    }

    Kirigami.Icon {
        anchors.fill: parent
        source: "im-light"
        opacity: !root.connected ? 0.35
               : root.isOn ? 1.0
               : 0.55
        Behavior on opacity { NumberAnimation { duration: 150 } }
    }

    Rectangle {
        anchors.right: parent.right
        anchors.bottom: parent.bottom
        width: Math.max(6, parent.width * 0.25)
        height: width
        radius: width / 2
        color: root.connected ? (root.isOn ? Kirigami.Theme.positiveTextColor : Kirigami.Theme.disabledTextColor)
                              : Kirigami.Theme.negativeTextColor
        border.color: Kirigami.Theme.backgroundColor
        border.width: 1
        visible: compactRoot.containsMouse || !root.connected
    }
}
