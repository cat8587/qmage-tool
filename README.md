# qmage
A library and command-line tool for working with Samsung's proprietary .qmg file format.

# Using it for custom boot images
Right now, animations are not supported for now, but for static images, use this:

```bash
./qmage-tool encode-boot-image <regular image> <bootsamsung.qmg output path>
```

To make it into a magisk module, make a new directory and write a module.prop like this:
```ini
id=<module id>
name=<name>
version=v1.0
versionCode=10
author=<your name>
description=<description>
```
As for the module id, the first character must be a letter and every other character must be a letter, a number, a dash (-) or an underscore (_).

Now in that same folder create a folder named system, then one inside system named media.
Copy your bootsamsung.qmg to media and copy and paste it again as bootsamsungloop.qmg.
Now place a shutdown.qmg which can be a different image or the same.
Compress the system folder and module.prop together as a .zip file.
Finally, install the module, reboot and enjoy!

Note: if you get an error saying RecordTooBig, then it means the image is too detailed for the maximum the file format can handle, this will improve if I end up improving the compression algorithm in the future.
