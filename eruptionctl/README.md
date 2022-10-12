## eruptionctl - A CLI control utility for the Eruption Linux user-mode driver

This is the command line interface to the Eruption daemon

### Example usage

```shell
$ eruptionctl switch profile swirl-perlin-blue-red-dim.profile 
```

```shell
$ eruptionctl switch slot 4
```

```shell
$ eruptionctl config brightness 100
```

```shell
$ eruptionctl status profile
Current profile: /var/lib/eruption/profiles/swirl-perlin-blue-red-dim.profile
```

```shell
$ eruptionctl devices debounce 1
Selected device: ROCCAT Kone Pure Ultra (1)
Debounce: true
```

```shell
$ eruptionctl devices brightness 1 20
Selected device: ROCCAT Kone Pure Ultra (1)
```

### eruptionctl

```shell
$ eruptionctl

Eruption is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

Eruption is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with Eruption.  If not, see <http://www.gnu.org/licenses/>.

Copyright (c) 2019-2022, The Eruption Development Team

 ********                          **   **                  
 /**/////                 ******   /**  //                   
 /**       ****** **   **/**///** ****** **  ******  ******* 
 /******* //**//*/**  /**/**  /**///**/ /** **////**//**///**
 /**////   /** / /**  /**/******   /**  /**/**   /** /**  /**
 /**       /**   /**  /**/**///    /**  /**/**   /** /**  /**
 /********/***   //******/**       //** /**//******  ***  /**
 //////// ///     ////// //         //  //  //////  ///   //

A CLI control utility for the Eruption Linux user-mode driver

Usage: eruptionctl [OPTIONS] <COMMAND>

Commands:
  config         Configuration related sub-commands
  color-schemes  Define, import or delete a named color scheme
  devices        Get or set some device specific configuration parameters
  status         Shows the currently active profile or slot
  switch         Switch to a different profile or slot
  profiles       Profile related sub-commands
  names          Naming related commands such as renaming of profile slots
  scripts        Script related sub-commands
  param          Get or set script parameters on the currently active profile
  completions    Generate shell completions
  help           Print this message or the help of the given subcommand(s)

Options:
  -v, --verbose...       Verbose mode (-v, -vv, -vvv, etc.)
  -r, --repeat           Repeat output until ctrl+c is pressed
  -c, --config <CONFIG>  Sets the configuration file to use
  -h, --help             Print help information
  -V, --version          Print version information

```
