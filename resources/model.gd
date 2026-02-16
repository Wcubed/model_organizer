class_name Model
extends Resource

## A single model can contain multiple stl files.

const CONFIG_FILE_NAME := "config.moa"

@export var directory: String
@export var name: String

## Relative path of all printable files found in the directory.
@export var printable_files: Array[String]
## Relative path of all non-model files found in the directory.
## Excludes images that have the same file name as a 3d model,
## Those are in `rendered_files`
@export var misc_files: Array[String]
## When an image has the same directory/name as a renderable file, it is
## put in this list.
@export var rendered_files: Array[String]

## Relative path to the cover image.
@export var cover_image_path: String = ""
## Cover image, if available
@export var cover_image: ImageTexture = null

## --- User config that is saved in the directory on-change ---

## Which way "up" the model should be displayed by default.
var default_orientation: Utils.ModelOrientation = Utils.ModelOrientation.Z_UP:
	set(new_value):
		default_orientation = new_value
		_save_config()

var supported_image_extensions := [".jpg", ".jpeg", ".png", ".webp"]
## These extensions can be put on the 3d printer. Others can't.
var printable_file_extensions := [".stl", ".3mf"]

func _init(p_directory = ""):
	directory = p_directory
	name = directory.split("/")[-1].split(".")[0]

## Scans the model directory for relevant info.
## Call this once after creating the resource.
func scan_directory():
	printable_files = []
	misc_files = []
	rendered_files = []
	cover_image_path = ""
	cover_image = null
	
	_load_config_or_default()
	
	_scan_subdirectory(directory, "")
	
	printable_files.sort_custom(Utils.sort_string_natural_order)
	misc_files.sort_custom(Utils.sort_string_natural_order)
	rendered_files.sort_custom(Utils.sort_string_natural_order)
	
	_find_and_load_cover_image()

func _load_config_or_default():
	var config = ConfigFile.new()
	# We do not care if the config loads correctly or not.
	# Because if there is no config, we want to fall back to defaults anyway.
	config.load("%s/%s" % [directory, CONFIG_FILE_NAME])
	
	default_orientation = config.get_value("main", "default_orientation", Utils.ModelOrientation.Z_UP)

## Call this only when the config is changed.
func _save_config():
	var config = ConfigFile.new()
	
	config.set_value("main", "default_orientation", default_orientation)
	
	config.save("%s/%s" % [directory, CONFIG_FILE_NAME])

## Returns true if the model matches the search parameters.
func matches_search(search_text: String) -> bool:
	if Utils.string_matches_search_pattern(name, search_text) || Utils.string_matches_search_pattern(directory, search_text):
		return true
	
	for file in printable_files:
		if Utils.string_matches_search_pattern(file, search_text):
			return true
	
	return false

func _find_and_load_cover_image():
	# See if we can find a cover image.
	var potential_covers := []
	potential_covers.append_array(misc_files)
	# If there is no cover in the standard images, we fall back to rendered ones.
	potential_covers.append_array(rendered_files)
	
	for file in potential_covers:
		for extension in supported_image_extensions:
			if file.ends_with(extension):
				cover_image_path = file
				
				if _load_cover_image() == OK:
					# Cover image loaded ok. Else we try the next one.
					return
	
	# No cover image.
	cover_image_path = ""

## Returns OK if the load was successful.
func _load_cover_image() -> int:
	if cover_image_path.is_empty():
		return FAILED
	
	var absolute_image_path := "%s/%s" % [directory, cover_image_path]
	var cached_cover_path = "%s/%s.png" % [Utils.cover_image_cache_dir, Utils.hash_string(absolute_image_path)]
	
	# The modified time will be "0" if the image does not exist, so it works even if the cache does not exist.
	var image_is_newer_than_cache := FileAccess.get_modified_time(absolute_image_path) > FileAccess.get_modified_time(cached_cover_path)
	
	var image = Image.new()
	
	var err := ERR_QUERY_FAILED
	if !image_is_newer_than_cache:
		# If the cache is up-to-date, try to load it.
		err = image.load(cached_cover_path)
	
	if err == OK:
		# Cached image loaded
		cover_image = ImageTexture.create_from_image(image)
	else:
		# No cached image, load the original.
		err = image.load(absolute_image_path)
	
		if err == OK:
			Utils.fit_image_proportional(image)
			
			# Cache the smaller version of the cover image.
			image.save_png(cached_cover_path)
			cover_image = ImageTexture.create_from_image(image)
	
	return err

func _scan_subdirectory(base_path: String, subdir: String):
	var dir = DirAccess.open("%s/%s" % [base_path, subdir])
	if !dir:
		return
	
	# Directory exists. Scan it.
	var new_files = Array(dir.get_files())
	while !new_files.is_empty():
		var next_file: String = new_files.pop_front()
		
		# This is protection against thingyverse wierdness, where sometimes
		# there are broken stl files in the images directory.
		if subdir.split("/")[-1] == "images" && next_file.ends_with(".stl"):
			# Do nothing
			continue
		
		if next_file == CONFIG_FILE_NAME:
			# Do not list the config in the list of files.
			continue
		
		var file_path := "%s/%s" % [subdir, next_file]
		
		if _is_file_printable(file_path):
			printable_files.append(file_path)
			
			# Check if there is a png with the same name.
			var rendered_png_file := Utils.strip_extension(next_file) + ".png"
			var position := new_files.find(rendered_png_file)
			
			var relative_path := "%s/%s" % [subdir, rendered_png_file]
			if position != -1:
				# There _is_ a rendered version of this renderable file.
				rendered_files.append(relative_path)
				# That file is now processed, skip it.
				new_files.remove_at(position)
			else:
				# Maybe we have already processed the file?
				position = misc_files.find(relative_path)
				if position != -1:
					misc_files.remove_at(position)
					rendered_files.append(relative_path)
		else:
			misc_files.append(file_path)
	
	# Scan subdirectories
	var subdirs = dir.get_directories()
	for new_dir in subdirs:
		var new_subdir = "%s/%s" % [subdir, new_dir]
		_scan_subdirectory(base_path, new_subdir)


func _is_file_printable(file: String) -> bool:
	for extension in printable_file_extensions:
		if file.to_lower().ends_with(extension):
			return true
	
	return false
