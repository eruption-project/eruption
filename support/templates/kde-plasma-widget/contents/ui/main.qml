import QtQuick
import QtQuick.Layouts
import QtQuick.Controls
import org.kde.plasma.plasma5support as Plasma5Support

Item {
    id: root
    width: 320
    height: 180

    property string activeProfile: "unavailable"
    property int activeSlot: 0
    property int brightness: 0
    property string statusText: "ready"

    function backendCommand(args) {
        return "~/.local/bin/eruption-plasma-backend " + args
    }

    function refreshState() {
        executableSource.connectSource(backendCommand("status"))
    }

    Plasma5Support.DataSource {
        id: executableSource
        engine: "executable"

        onNewData: function(sourceName, data) {
            var out = data["stdout"] || ""
            var err = data["stderr"] || ""
            var code = data["exit code"]
            disconnectSource(sourceName)

            if (sourceName.indexOf(" status") >= 0) {
                if (code !== 0) {
                    root.statusText = err.length > 0 ? err : "backend unavailable"
                    return
                }

                try {
                    var s = JSON.parse(out)
                    root.activeProfile = s.profile
                    root.activeSlot = s.slot
                    root.brightness = s.brightness
                    root.statusText = "ok"
                } catch (e) {
                    root.statusText = "parse error"
                }
            } else if (code !== 0) {
                root.statusText = err.length > 0 ? err : "command failed"
            } else {
                root.statusText = "ok"
                refreshState()
            }
        }
    }

    Timer {
        interval: 10000
        running: true
        repeat: true
        onTriggered: refreshState()
    }

    Component.onCompleted: refreshState()

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
            elide: Text.ElideRight
            Layout.fillWidth: true
        }

        Label {
            text: "Slot: " + (root.activeSlot > 0 ? root.activeSlot : "N/A")
        }

        Slider {
            id: brightnessSlider
            from: 0
            to: 100
            value: root.brightness
            onMoved: {
                executableSource.connectSource(backendCommand("set-brightness " + Math.round(value)))
            }
        }

        RowLayout {
            Repeater {
                model: [1, 2, 3, 4]
                delegate: Button {
                    text: "S" + modelData
                    onClicked: executableSource.connectSource(backendCommand("set-slot " + modelData))
                }
            }
        }

        RowLayout {
            Button {
                text: "Refresh"
                onClicked: {
                    refreshState()
                }
            }

            Button {
                text: "Open UI"
                onClicked: {
                    executableSource.connectSource(backendCommand("open-ui"))
                }
            }
        }

        Label {
            text: "Status: " + root.statusText
            color: root.statusText === "ok" ? "#3a7" : "#c84"
        }
    }
}
