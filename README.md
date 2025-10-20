# brightnessctl-rs

A basic CLI to manage brightness of the following classes

- `/sys/class/backlight/*`
- `/sys/class/leds/*`


## nixOS
Add the following to your nix configuration in order for the proper udev rules to be present

udev rules:
```nix
services.udev.extraRules = ''
  ACTION=="add", SUBSYSTEM=="backlight", RUN+="${pkgs.coreutils}/bin/chgrp video /sys/class/backlight/%k/brightness"
  ACTION=="add", SUBSYSTEM=="backlight", RUN+="${pkgs.coreutils}/bin/chmod g+w /sys/class/backlight/%k/brightness"
  ACTION=="add", SUBSYSTEM=="leds", RUN+="${pkgs.coreutils}/bin/chgrp input /sys/class/leds/%k/brightness"
  ACTION=="add", SUBSYSTEM=="leds", RUN+="${pkgs.coreutils}/bin/chmod g+w /sys/class/leds/%k/brightness"
'';
```

Also add your user to the `video` group

```nix
users.users.{username}= {
  extragroups = [ "video" ];
};

```
