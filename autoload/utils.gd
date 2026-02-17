extends Node

## Size of cover images when scaled down from the original.
const ICON_SIZE := 300.0

## Which axis is "up" for a model.
## Z is most common, with Y second common.
enum ModelOrientation {Z_UP = 0, Y_UP = 1}

enum StringMatchResult {SUCCESS, FAIL_MISSING_POSITIVE, FAIL_CONTAINS_NEGATIVE}

# If we want to "open folder and select file" on linux, we need to fall back to doing it manually.
# This is true if dolphin is available on the system.
var dolphin_is_installed := false

@onready var cover_image_cache_dir := "%s/%s" % [OS.get_user_data_dir(), "cover_images"]


func _ready() -> void:
	if OS.get_name() == "Linux":
		# See if dolphin is installed.
		var result = OS.execute("dolphin", ["-v"])
		dolphin_is_installed = result == 0


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


func strip_extension(path: String) -> String:
	var extension_location := path.rfind(".")
	return path.substr(0, extension_location)


## Returns true if a < b
func sort_string_natural_order(a: String, b: String) -> bool:
	return a.naturalnocasecmp_to(b) == -1


func select_file_in_file_manager(path: String):
	if dolphin_is_installed:
		# The window by default does not come to the foreground due to focus-stealing prevention.
		# To fix this, on dolphin:
		# - go to "more actions -> Configure special window settings..."
		# - Add "Focus stealing prevention": "Force", "None"
		# Now dolphin will come to the foreground when activated.
		OS.create_process("dolphin", ["--select", path])
	else:
		OS.shell_show_in_file_manager(path, false)

func open_with_default_program(path: String):
	OS.shell_open(path)


## Returns true if the given string matches the given search pattern.
func string_matches_search_pattern(string: String, pattern: String) -> StringMatchResult:
	var items := pattern.to_lower().split(" ")
	var lower_string = string.to_lower()
	
	var result := StringMatchResult.SUCCESS
	
	for item in items:
		# An item prefixed with "!" is a negative search term.
		if item.begins_with("!"):
			item = item.trim_prefix("!")
			if lower_string.contains(item):
				# Not allowed to contain this item.
				result = StringMatchResult.FAIL_CONTAINS_NEGATIVE
				# A negative fail has priority over other fails.
				break
		else:
			if !lower_string.contains(item):
				# The string needs to contain all the items to match.
				if result != StringMatchResult.FAIL_CONTAINS_NEGATIVE:
					result = StringMatchResult.FAIL_MISSING_POSITIVE
	
	return result
