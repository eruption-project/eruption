# Table of Contents

- [Table of Contents](#table-of-contents)
  - [Adapting an existing Profile](#adapting-an-existing-profile)
    - [Caveats](#caveats)
    - [Copying the `.profile` file to a new location](#copying-the-profile-file-to-a-new-location)
    - [Changing the UUID, Name and Description](#changing-the-uuid-name-and-description)
    - [Changing some Parameters like e.g. colors](#changing-some-parameters-like-eg-colors)
      - [Example Colors](#example-colors)


## Adapting an existing Profile

In this HOWTO we will outline each step that is required to customize an existing profile, and adapt it to your needs.

### Caveats

Since Eruption `0.1.23` it is possible to override the values defined in a `.profile` file using a `.profile.state` file.
This file will be created if you customize a parameter using the `Eruption GUI` or by using the `eruptionctl param` command.

If such a file exists, all parameters will be set to the values from that `.profile.state` file. Values from the original
`.profile` file will only be used again if you delete the `.profile.state` file.

```sh
sudo rm /var/lib/eruption/profiles/myprofile.profile.state
```

### Copying the `.profile` file to a new location

Copy the existing profile to a new location, replace `myprofile` with the name of e.g. your game.

```sh
cp -v /var/lib/eruption/profiles/gaming.profile /var/lib/eruption/profiles/myprofile.profile
```

### Changing the UUID, Name and Description

Then you need to change the `UUID` to something unique. You should change the `name` and `description` fields too:

```sh
sudoedit /var/lib/eruption/profiles/myprofile.profile
```

Generate a new UUID:

```sh
uuidgen
```

### Changing some Parameters like e.g. colors

Every parameter exported by a Lua script file can be customized by a `.profile` file. Please use the following
command to find out, what parameters each referenced Lua script exports:

```sh
eruptionctl -v param
```

To get hex-code colors using a nice UI please visit this website:
[W3schools Color Picker](https://www.w3schools.com/colors/colors_picker.asp)

Use 0xFF\<the hex code from the color picker\> to define a fully opaque color. Eruption uses the `ARGB` format.

A: Alpha channel: 0-255, (0x00 - 0xFF)\
R: Red channel: 0-255, (0x00 - 0xFF)\
G: Green channel: 0-255, (0x00 - 0xFF)\
B: Blue channel: 0-255, (0x00 - 0xFF)

#### Example Colors

Red (fully opaque): 0xFFFF0000
Red (semi transparent): 0x40FF0000

Yellow (fully opaque): 0xFFFFFF00

...
