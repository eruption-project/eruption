# Table of Contents

- [Table of Contents](#table-of-contents)
  - [Support for Audio Playback and Capture](#support-for-audio-playback-and-capture)
    - [For PipeWire enabled Distros](#for-pipewire-enabled-distros)
      - [Configure routing](#configure-routing)
    - [For PulseAudio enabled Distros](#for-pulseaudio-enabled-distros)

## Support for Audio Playback and Capture

If you want Eruption to be able to play back sound effects, or use one of the
audio visualizer Lua scripts, then you have to perform a few additional steps.
The following steps will allow the Eruption daemon to access the PipeWire/PulseAudio
server of the current user, for playback and for capturing of audio signals as well.

Eruption currently has built-in support for the following audio APIs:

* PipeWire (via the PulseAudio interface of PipeWire)
* PulseAudio

### For PipeWire enabled Distros

If your distribution uses PipeWire, please create the configuration directory and edit
the client configuration file in `/root/.config/pulse/client.conf` for the user that
Eruption runs as (default: root)

```sh
 $ sudo mkdir -p /root/.config/pulse/
 $ EDITOR=nano sudoedit /root/.config/pulse/client.conf
```

and then add the following lines:

```ini
autospawn = no
default-server = unix:/run/user/1000/pulse/native
enable-memfd = yes
```

This will enable Eruption to process the audio streams of the user with the ID `1000`.

#### Configure routing

Now we need to set up routing. First we need to determine the sink that Eruption should listen on:

```sh
 $ pactl list short sinks
```

```sh
44	alsa_output.pci-0000_01_00.1.hdmi-stereo	PipeWire	s32le 2ch 48000Hz	SUSPENDED
46	alsa_output.pci-0000_00_1f.3.analog-stereo	PipeWire	s32le 2ch 48000Hz	RUNNING
```

On this system, the name of the correct sink would be `alsa_output.pci-0000_00_1f.3.analog-stereo`.
So let's route the sink to the `Eruption Audio Grabber`.

```sh
 $ pactl list source-outputs
```

Find the ID of the `Eruption Audio Grabber`, in this case this would be `Source Output #88`.

```sh
Source Output #88
	Driver: PipeWire
	Owner Module: n/a
	Client: 91
	Source: 65582
	Sample Specification: s16le 2ch 44100Hz
	Channel Map: front-left,front-right
	Format: pcm
	Corked: no
	Mute: no
	Volume: front-left: 65536 / 100% / 0,00 dB,   front-right: 65536 / 100% / 0,00 dB
	        balance 0,00
	Buffer Latency: 0 usec
	Source Latency: 0 usec
	Resample method: PipeWire
	Properties:
		client.api = "pipewire-pulse"
		application.name = "eruption"
		application.process.id = "97789"
		application.process.user = "root"
		application.process.host = "<redacted>"
		application.process.binary = "eruption"
		application.language = "C"
		window.x11.display = ":1"
		application.process.machine_id = "<redacted>"
		media.name = "Audio Grabber"
		stream.is-live = "true"
		node.name = "eruption"
		node.autoconnect = "true"
		media.class = "Stream/Input/Audio"
		adapt.follower.node = ""
		factory.id = "6"
		audio.adapt.follower = ""
		factory.mode = "merge"
		library.name = "audioconvert/libspa-audioconvert"
		object.id = "88"
		client.id = "91"
		node.latency = "88200/44100"
		pulse.attr.maxlength = "4194304"
		pulse.attr.fragsize = "352800"
		module-stream-restore.id = "source-output-by-application-name:eruption"
```

Route the previously determined sink to the `Eruption Audio Grabber` and set a reasonable volume:

```sh
 $ pactl move-source-output 88 alsa_output.pci-0000_00_1f.3.analog-stereo.monitor
 $ pactl set-source-output-volume 88 '85%'
```

Finally, restart PipeWire and Eruption for the changes to take effect:

```sh
 $ systemctl --user restart pipewire-pulse.service
 $ systemctl --user restart pipewire.service
 $ sudo systemctl restart eruption.service
```

### For PulseAudio enabled Distros

Create the config directory and edit the server configuration file
for your user account:

```sh
 $ mkdir -p ~/.config/pulse/
 $ cp /etc/pulse/default.pa ~/.config/pulse/default.pa
 $ nano ~/.config/pulse/default.pa
```

then add the following line at the end of the file:

```conf
load-module module-native-protocol-unix auth-group=root socket=/tmp/pulse-server
```

Create the PulseAudio configuration directory and edit the client configuration
file in `/root/.config/pulse/client.conf` for the user that Eruption runs as
(default: root)

```sh
 $ sudo mkdir -p /root/.config/pulse/
 $ EDITOR=nano sudoedit /root/.config/pulse/client.conf
```

and then add the following lines:

```ini
autospawn = no
default-server = unix:/tmp/pulse-server
enable-memfd = yes
```

To configure routing of sinks to the `Eruption Audio Grabber`, please see
`Configure routing` in the section above.

Finally, restart PulseAudio and Eruption for the changes to take effect:

```sh
 $ systemctl --user restart pulseaudio.service
 $ sudo systemctl restart eruption.service
```
