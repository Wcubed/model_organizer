extends Node

## Size of cover images when scaled down from the original.
const ICON_SIZE := 300.0


@onready var cover_image_cache_dir := "%s/%s" % [OS.get_user_data_dir(), "cover_images"]


## Fits the image into a square of ICON_SIZE.
func fit_image_proportional(image: Image):
	# Scale the image to fit to the expected size
	var new_height = Utils.ICON_SIZE
	var new_width = Utils.ICON_SIZE
	if image.get_height() > image.get_width():
		var new_scale = Utils.ICON_SIZE / image.get_height() as float
		new_width = image.get_width() * new_scale
	else:
		var new_scale = Utils.ICON_SIZE / image.get_width() as float
		new_height = image.get_height() * new_scale
	
	image.resize(new_width, new_height, Image.INTERPOLATE_BILINEAR)


## Returns the hexadecimal representation of the hash of the given string.
func hash_string(string: String) -> String:
	var ctx = HashingContext.new()
	ctx.start(HashingContext.HASH_MD5)
	ctx.update(string.to_multibyte_char_buffer())
	return ctx.finish().hex_encode()
