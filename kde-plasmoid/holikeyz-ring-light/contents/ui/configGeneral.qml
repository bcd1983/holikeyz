import QtQuick
import QtQuick.Layouts
import QtQuick.Controls as QQC2
import org.kde.kirigami as Kirigami

Kirigami.FormLayout {
    id: generalPage

    property alias cfg_pollInterval: pollField.value

    QQC2.SpinBox {
        id: pollField
        Kirigami.FormData.label: i18n("Poll interval (ms):")
        from: 500
        to: 60000
        stepSize: 500
    }

    QQC2.Label {
        Kirigami.FormData.label: ""
        text: i18n("The active light is managed by the Rust D-Bus service (com.holikeyz.RingLight). " +
                   "Use the Discover button in the popup header to scan for lights and pick one.")
        wrapMode: Text.WordWrap
        Layout.preferredWidth: Kirigami.Units.gridUnit * 20
        opacity: 0.7
    }
}
