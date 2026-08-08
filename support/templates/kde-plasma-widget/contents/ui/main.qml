import QtQuick
import QtQuick.Layouts
import QtQuick.Controls

Item {
    id: root
    width: 320
    height: 180

    property string activeProfile: "Unknown"
    property int activeSlot: -1
    property int brightness: 0

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: 12
        spacing: 8

        Label {
            text: "Eruption"
            font.bold: true
        }

        Label {
            text: "Profile: " + root.activeProfile
        }

        Label {
            text: "Slot: " + (root.activeSlot >= 0 ? root.activeSlot : "N/A")
        }

        Slider {
            from: 0
            to: 100
            value: root.brightness
            onMoved: {
                // TODO: connect to D-Bus call for global brightness
            }
        }

        RowLayout {
            Button {
                text: "Refresh"
                onClicked: {
                    // TODO: fetch current state from D-Bus
                }
            }

            Button {
                text: "Open UI"
                onClicked: {
                    // TODO: launch pyroclasm if available
                }
            }
        }
    }
}
