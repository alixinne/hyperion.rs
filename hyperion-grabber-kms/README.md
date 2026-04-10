# hyperion-grabber-kms

hyperion-grabber-kms is a companion binary to the hyperion.rs program. It's
intended to run on the host producing video output, just like the Android TV
grabber app or the X11/Wayland grabbers.

The main difference of this grabber compared to the X11/Wayland grabbers is
that this one uses direct access to the output framebuffer (via _DMA BUF_),
which works even with no X11 or Wayland desktop environment running - such
as when running a login session under Gamescope rather than a traditional
desktop environment.

## How to use

This currently relies on access to the framebuffer, which is a privileged
operation. This binary must either run as root, or at the very least have
`CAP_SYS_ADMIN` to allow the capture to succeed.

```
Usage: hyperion-grabber-kms [OPTIONS] --card <CARD> --target-host <TARGET_HOST>

Options:
      --card <CARD>
          Path to the DRI card device node to use
      --tone-mapping-offset <TONE_MAPPING_OFFSET>
          Offset for the linear tone mapping curve [default: 0.0]
      --tone-mapping-scaling <TONE_MAPPING_SCALING>
          Scaling factor for the linear tone mapping curve [default: 1.0]
      --target-host <TARGET_HOST>
          Target host (hyperion, protobuf server)
      --image-width <IMAGE_WIDTH>
          Buffer image width [default: 180]
  -v, --verbose...
          Increase logging verbosity
  -q, --quiet...
          Decrease logging verbosity
      --fps <FPS>
          Capture FPS [default: 30]
  -h, --help
          Print help
  -V, --version
          Print version
```

For example, assuming a Raspberry Pi connected to a LED strip:

```bash
hyperion-grabber-kms --card /dev/dri/card1 --target-host raspi.lan:19445
```

## Status

This is currently experimental. The following are expected to be broken
on many setups:

- HDR to SDR tone mapping is very crude: actual transfer functions should be
  used to perform proper tone mapping rather than just offset/scaling.
- Various framebuffer configurations: only 1-buffer and 3-buffer configurations
  are supported when setting up the capture pipeline. Other configurations will
  fail.
- Only tested on recent AMD GPUs.
