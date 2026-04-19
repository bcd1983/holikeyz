import QtQuick
import QtQuick.Layouts
import QtQuick.Controls as QQC2
import org.kde.kirigami as Kirigami

Kirigami.FormLayout {
    id: generalPage

    property alias cfg_ip: ipField.text
    property alias cfg_port: portField.value
    property alias cfg_pollInterval: pollField.value

    QQC2.TextField {
        id: ipField
        Kirigami.FormData.label: i18n("Light IP:")
        placeholderText: "192.168.6.80"
        Layout.preferredWidth: Kirigami.Units.gridUnit * 12
    }

    QQC2.SpinBox {
        id: portField
        Kirigami.FormData.label: i18n("Port:")
        from: 1
        to: 65535
    }

    QQC2.SpinBox {
        id: pollField
        Kirigami.FormData.label: i18n("Poll interval (ms):")
        from: 1000
        to: 60000
        stepSize: 1000
    }

    QQC2.Label {
        Kirigami.FormData.label: ""
        text: i18n("Use `holikeyz-cli discover` to find the light's IP on your network.")
        wrapMode: Text.WordWrap
        Layout.preferredWidth: Kirigami.Units.gridUnit * 18
        opacity: 0.7
    }
}
