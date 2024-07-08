# shhh 
_simple tool to for adding shadows and border radius to an image._

![mew](https://github.com/entailz/shhh/blob/main/assets/shhh_aft.png?raw=true)

##### Default behavior:

pipe an image to shhh and receive the image data via stdout. 

specify --input and/or --output in order to override this behavior.

##### __Usage:__

```bash
grim -g"$(slurp)" - | shhh > image.png
```

_alternatively_, replace the image out with your clipboard of choice, like so:

```bash
grim -g"$(slurp)" - | shhh -e -20,-20 -s 20 -a 120| wl-copy --type image/png
```

