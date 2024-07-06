# shhh 
_simple tool to for adding shadows to an image._
##### Default behavior:

pipe an image to shhh and receive the image via stdout. 

specify --input & --output in order to override this behavior.

##### __Usage:__

```bash
grim -g"$(slurp)" - | shhh --offset=4,4 --radius=10 --spread_radius=10 --alpha=40 > image.png
```

_alternatively_, replace the image out with your clipboard of choice, like so:

```bash
grim -g"$(slurp)" - | shhh --offset=-60,0 --radius=20 --spread_radius=20 --alpha=60 | wl-copy --type image/png
```

