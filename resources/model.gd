class_name Model
extends Resource

## A single model can contain multiple stl files.

@export var directory: String
@export var name: String
## Absolute path of all files found in the directory.
@export var files: Array[String]
## Absolute path to the cover image.
@export var cover_image_path: String = ""
## Cover image, if available
@export var cover_image: ImageTexture = null

var supported_image_extensions := [".jpg", ".jpeg", ".png", ".webp"]

func _init(p_directory = ""):
	directory = p_directory
	name = directory.split("/")[-1].split(".")[0]

## Scans the model directory for relevant info.
## Call this once after creating the resource.
func scan_directory():
	files = []
	cover_image_path = ""
	cover_image = null
	
	_scan_subdirectory(directory)
	
	_find_cover_image()
	_load_cover_image()

func _find_cover_image():
	# See if we can find a cover image.
	for file in files:
		for extension in supported_image_extensions:
			if file.ends_with(extension):
				cover_image_path = file
				return
	
	# No cover image.
	cover_image_path = ""

func _load_cover_image():
	if cover_image_path.is_empty():
		return
	
	var image = Image.new()
	var err := image.load(cover_image_path)
	
	if err == OK:
		# Scale the image to fit to the expected size
		var container_size = 300.0
		var new_height = container_size
		var new_width = container_size
		if image.get_height() > image.get_width():
			var new_scale = container_size / image.get_height() as float
			new_width = image.get_width() * new_scale
		else:
			var new_scale = container_size / image.get_width() as float
			new_height = image.get_height() * new_scale
		
		image.resize(new_width, new_height, Image.INTERPOLATE_BILINEAR)
		
		cover_image = ImageTexture.create_from_image(image)

func _scan_subdirectory(path: String):
	var dir = DirAccess.open(path)
	if !dir:
		return
	
	# Directory exists. Scan it.
	var new_files = Array(dir.get_files())
	for file in new_files:
		files.append("%s/%s" % [path, file])
	
	# Scan subdirectories
	var subdirs = dir.get_directories()
	for subdir in subdirs:
		var new_path = "%s/%s" % [path, subdir]
		_scan_subdirectory(new_path)
