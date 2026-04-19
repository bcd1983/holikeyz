import QtQuick
import QtQuick.Layouts
import QtQuick.Controls as QQC2
import org.kde.plasma.core as PlasmaCore
import org.kde.plasma.components as PlasmaComponents
import org.kde.kirigami as Kirigami

Rectangle {
    id: cardRoot

    property string sceneId: ""
    property string sceneName: ""
    property bool active: false
    signal activated()

    radius: Kirigami.Units.largeSpacing
    color: "transparent"
    border.width: active ? 2 : 1
    border.color: active ? Kirigami.Theme.highlightColor
                         : Qt.rgba(1, 1, 1, 0.1)
    clip: true

    Behavior on border.width { NumberAnimation { duration: 120 } }

    Image {
        id: thumbnail
        anchors.fill: parent
        source: Qt.resolvedUrl("../../images/" + cardRoot.sceneId + ".jpg")
        fillMode: Image.PreserveAspectCrop
        asynchronous: true
        smooth: true
        opacity: mouse.pressed ? 0.75 : 1.0
        Behavior on opacity { NumberAnimation { duration: 100 } }
    }

    Rectangle {
        anchors.fill: parent
        gradient: Gradient {
            GradientStop { position: 0.45; color: "transparent" }
            GradientStop { position: 1.0;  color: "#c0000000" }
        }
    }

    PlasmaComponents.Label {
        anchors.bottom: parent.bottom
        anchors.horizontalCenter: parent.horizontalCenter
        anchors.bottomMargin: Kirigami.Units.smallSpacing
        text: cardRoot.sceneName
        color: "white"
        font.bold: true
        font.pixelSize: Kirigami.Theme.defaultFont.pixelSize
        style: Text.Raised
        styleColor: "#80000000"
    }

    Rectangle {
        visible: mouse.containsMouse && !mouse.pressed
        anchors.fill: parent
        color: "white"
        opacity: 0.08
        radius: cardRoot.radius
    }

    MouseArea {
        id: mouse
        anchors.fill: parent
        hoverEnabled: true
        cursorShape: Qt.PointingHandCursor
        onClicked: cardRoot.activated()
    }
}
